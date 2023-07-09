/// 定义了数据加载器

/// 本地数据加载器
pub mod local {
    use std::collections::HashMap;
    use std::fmt::Write;
    use std::ops::{Add};
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    use anyhow::{bail, Context};
    use tracing::log::debug;
    use tracing::trace;

    use crate::{Bar, BarLoader, Chart, ChartLoader, ChartParamter, MarketCurrentLoader, Period, Stock, Stocks, StocksLoader, TradingDay};
    use crate::stock::GetSymbolCode;

    pub fn data_dir() -> anyhow::Result<PathBuf> {
        let dir = dirs::data_local_dir().ok_or(anyhow::anyhow!("data local dir not found"))?;
        let dir = dir.join("la.renzhen.trading");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).context("Failed to create data local dir")?;
        }
        Ok(dir)
    }

    pub fn config_dir() -> anyhow::Result<PathBuf> {
        let dir = dirs::home_dir().ok_or(anyhow::anyhow!("config dir not found"))?;
        let dir = dir.join(".config/la.renzhen.trading");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).context("Failed to create config dir")?;
        }
        Ok(dir)
    }

    pub(crate) fn parse_stocks_data(content: String) -> anyhow::Result<Stocks> {
        let mut stocks = vec![];
        for line in content.lines().skip(1) {
            let mut it = line.split("\t");
            let symbol = it.next().context("can't found stock symbol")?;
            let name = it.next().context("can't found stock name")?;
            stocks.push(Stock::new(name, symbol))
        }
        Ok(Stocks::new(stocks).sorted())
    }

    pub fn write_stocks_data<P: AsRef<Path>>(path: P, stocks: &Stocks) -> anyhow::Result<()> {
        let mut content = String::from("股票代码,股票名称");
        for stock in stocks.iter() {
            content.write_str(format!("\n{},{}", stock.symbol, stock.name).as_str())?;
        }
        std::fs::write(path, content).context("write file")?;
        Ok(())
    }

    #[derive(Debug, Clone)]
    pub struct LocalLoader {
        base_dir: PathBuf,
    }

    impl LocalLoader {
        pub fn base() -> anyhow::Result<Self> {
            data_dir().and_then(|path| LocalLoader::new(path))
        }

        pub fn new(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
            let path = path.into();
            let test_writeable = path.join("test");
            std::fs::write(&test_writeable, "").context("Failed to write local folder")?;
            std::fs::remove_file(&test_writeable).context("Failed to remove local folder")?;

            Ok(Self { base_dir: path.into() })
        }

        pub fn day_chart_path(&self, symbol: impl GetSymbolCode) -> anyhow::Result<PathBuf> {
            let symbol = symbol.symbol();
            let (s1, s2) = (&symbol[..2], &symbol[2..4]);
            self.storage(format!("stocks/day/{}/{}/{}.csv", s1, s2, symbol))
        }

        pub fn minutes_chart_dir(&self, symbol: impl GetSymbolCode) -> anyhow::Result<PathBuf> {
            let symbol = symbol.symbol();
            let (s1, s2) = (&symbol[..2], &symbol[2..4]);
            self.storage(format!("stocks/minutes/{}/{}/{}", s1, s2, symbol))
        }

        pub fn stocks_path(&self) -> anyhow::Result<PathBuf> {
            self.storage("stocks.csv")
        }

        pub fn storage(&self, append: impl AsRef<str>) -> anyhow::Result<PathBuf> {
            Ok(self.base_dir.join(append.as_ref()))
        }
    }

    #[async_trait::async_trait]
    impl StocksLoader for LocalLoader {
        async fn stocks(&self) -> anyhow::Result<Stocks> {
            let path = self.stocks_path()?;
            let content = tokio::fs::read_to_string(path).await.context("Failed to read stock file")?;
            parse_stocks_data(content)
        }
    }

    impl LocalLoader {
        fn parse_chart(&self, content: String) -> anyhow::Result<Chart> {
            let lines = content.lines().skip(1);
            let mut chart = Chart::default();
            let mut yesterday = 0.0;
            for line in lines.into_iter() {
                if line.is_empty() {
                    continue;
                }
                let mut fields = line.split(',');

                let date = fields.next().context("not found date")?;
                let mut bar = Bar::new(date);

                bar.open = fields.next().context("not found open")?.parse::<f64>().context("parse open")?;
                bar.high = fields.next().context("not found high")?.parse::<f64>().context("parse high")?;
                bar.low = fields.next().context("not found low")?.parse::<f64>().context("parse low")?;
                bar.close = fields.next().context("not found close")?.parse::<f64>().context("parse close")?;
                bar.volume = fields.next().context("not found volume")?.parse::<f64>().context("parse volume")?;
                if !bar.is_ok() {
                    continue;
                }
                bar.yesterday = yesterday;
                yesterday = bar.close;
                chart.push(bar);
            }
            Ok(chart)
        }

        async fn day_chart(&self, param: ChartParamter) -> anyhow::Result<Chart> {
            let path = self.day_chart_path(&param.symbol)?;
            let length = param.limit.unwrap_or(usize::MAX);

            let err = format!("[{}] read stock chart file: {}", param.symbol, path.display());
            let content = tokio::fs::read_to_string(&path).await.context(err)?;

            let mut chart = self.parse_chart(content)?;

            chart.length(length);

            Ok(chart)
        }

        async fn week_chart(&self, mut param: ChartParamter) -> anyhow::Result<Chart> {
            if let Some(end) = &param.end {
                param.end = Some(TradingDay::from_str(&end)?.week_end_day().to_string());
            }
            let limit = std::mem::replace(&mut param.limit, None);

            let output = self.day_chart(param).await?.value();
            let output = output.into_iter()
                .map(|mut v| {
                    v.date = TradingDay::from_str(&v.date).unwrap().week_start_day().to_string();
                    v
                })
                .fold(Vec::default(), |mut acc: Vec<Bar>, bar| {
                    match acc.last_mut() {
                        Some(last) if last.date.eq(&bar.date) => {
                            last.merge(bar);
                        }
                        _ => acc.push(bar),
                    }
                    acc
                });
            let mut chart = Chart::new(output);
            if let Some(limit) = limit {
                chart.length(limit);
            }
            Ok(chart)
        }

        async fn minutes_chart(&self, param: ChartParamter) -> anyhow::Result<Chart> {
            let path = self.minutes_chart_dir(&param.symbol)?;
            if !path.exists() {
                return Ok(Chart::default());
            }

            let Period::Minute(minutes) = param.period.clone() else {
                unreachable!("!!");
            };

            let end = TradingDay::trading(param.end.clone())?;
            let limit = param.limit.unwrap_or((60 / minutes) * 4 * 5);
            let limit = (limit as f64/ ((60 / minutes) * 4) as f64).ceil() as usize;
            let mut start = end.clone() - (limit - 1);
            debug!("start: {}, end: {}", start, end);

            let mut items = vec![];

            while start.le(&end) {
                let file = path.join(format!("{}.csv",start.to_string()));
                start = start.add(1);

                if !file.exists() {
                    if items.len() > 0 {
                        bail!("invalid minutes day");
                    }
                    continue;
                }
                trace!("load {:?}", file);

                if let Ok(content) = tokio::fs::read_to_string(file).await {
                    let mut chart = self.parse_chart(content)?.value();
                    items.append(&mut chart);
                }
            }

            Ok(Chart::with_period(items,Period::Minute(minutes)))
        }
    }

    #[async_trait::async_trait]
    impl ChartLoader for LocalLoader {
        async fn chart(&self, param: impl Into<ChartParamter> + Send) -> anyhow::Result<Chart> {
            let param = param.into();
            match &param.period {
                Period::Day => self.day_chart(param).await,
                Period::Week => self.week_chart(param).await,
                Period::Minute(_) => self.minutes_chart(param).await,
            }
        }
    }

    #[async_trait::async_trait]
    impl MarketCurrentLoader for LocalLoader {
        async fn market(&self) -> anyhow::Result<HashMap<String, Bar>> {
            anyhow::bail!("not support")
        }
    }

    #[async_trait::async_trait]
    impl BarLoader for LocalLoader {
        async fn current(&self, _symbol: impl GetSymbolCode + Send) -> anyhow::Result<Bar> {
            anyhow::bail!("not support")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        #[ignore]
        fn show_dirs() {
            let path = data_dir().unwrap();
            println!("data_dir: {:?}", path);

            let loader = LocalLoader::base().unwrap();
            let path = loader.day_chart_path("600444").unwrap();
            println!("stock: {:?}", path);

            let path = loader.stocks_path().unwrap();
            println!("stocks: {:?}", path);
        }

        #[tokio::test]
        #[ignore]
        async fn load_stocks() {
            let loader = LocalLoader::base().unwrap();
            let stocks = loader.stocks().await;
            assert!(stocks.is_ok(), "load chart error");
            let stocks = stocks.unwrap();
            assert!(stocks.len() > 0, "load chart error");
        }

        #[tokio::test]
        #[ignore]
        async fn load_chart() {
            let loader = LocalLoader::base().unwrap();
            {
                let param = ChartParamter::day("601888")
                    .limit(2).end("2023-07-12");
                let chart = loader.chart(param).await.unwrap();
                for bar in chart.iter() {
                    println!("{}, {},{},{},{}, {}", bar.date, bar.open, bar.high, bar.low, bar.close, bar.volume);
                }
            }

            {
                let param = ChartParamter::new("601888", Period::Week).limit(2);
                let chart = loader.chart(param).await.unwrap();
                for bar in chart.iter() {
                    println!("{}, {},{},{},{}, {}", bar.date, bar.open, bar.high, bar.low, bar.close, bar.volume);
                }
            }
        }

        #[tokio::test]
        #[ignore]
        async fn load_minutes_chart() {
            let loader = LocalLoader::base().unwrap();
            let param = ChartParamter::new("601888", Period::Minute(30)).end("2023-07-12");
            let chart = loader.chart(param).await.unwrap();
            dbg!(&chart);
            for bar in chart.iter() {
                println!("{}, {},{},{},{}, {}", bar.date, bar.open, bar.high, bar.low, bar.close, bar.volume);
            }
        }
    }
}

/// 远程加载器
pub mod remote {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use anyhow::Context;
    use reqwest::{header, Method, RequestBuilder, Response};
    use serde::{Deserialize, Serialize};

    use crate::{Bar, BarLoader, Chart, ChartLoader, ChartParamter, MarketCurrentLoader, Stock, Stocks};
    use crate::stock::GetSymbolCode;

    pub mod headers {
        pub const NAME: &str = "x-trading-name";
        pub const VERSION: &str = "x-trading-version";
        pub const PLATFORM: &str = "x-trading-platform";
        pub const AK: &str = "x-trading-access-key";
        pub const SIGN: &str = "x-trading-sign";
        pub const TIMESTAMP: &str = "x-trading-timestamp";
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Credential {
        pub access_key: String,
        pub secret_key: String,
    }

    pub trait CredentialProvider {
        fn credential(&self) -> anyhow::Result<Option<Credential>>;
    }

    #[derive(Debug, Clone)]
    pub struct RemoteLoader {
        pub host: String,
        pub credential: Option<Credential>,
        pub timeout: Option<std::time::Duration>,
    }

    impl Default for RemoteLoader {
        fn default() -> Self {
            Self {
                host: "http://127.0.0.1:18686/api/data".to_string(),
                credential: None,
                timeout: Some(std::time::Duration::from_secs(3)),
            }
        }
    }

    pub fn sign(_version: &str, secret_key: &str, data: &str, timestamp: &str) -> String {
        let data = md5::compute(format!("{}:{}:{}", secret_key, data, timestamp));
        let data = md5::compute(format!("{:x}", data));
        format!("{:x}", data)
    }

    impl RemoteLoader {
        pub fn new<P: CredentialProvider>(
            host: &str,
            timeout: Option<std::time::Duration>,
            provider: P,
        ) -> anyhow::Result<Self> {
            Ok(Self { host: host.to_string(), credential: provider.credential()?, timeout })
        }

        pub fn with_host<T: AsRef<str>>(mut self, host: T) -> Self {
            self.host = host.as_ref().to_string();
            self
        }
        pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
            self.timeout = Some(timeout);
            self
        }

        #[allow(dead_code)]
        pub fn with_provider<T: CredentialProvider>(mut self, provider: T) -> anyhow::Result<Self> {
            self.credential = provider.credential()?;
            Ok(self)
        }

        pub fn with_credential(mut self, credential: Credential) -> Self {
            self.credential = Some(credential);
            self
        }

        fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
            let url = format!("{}{}", self.host, path);
            let mut req = reqwest::Client::new().request(method, url);
            if let Some(timeout) = &self.timeout {
                req = req.timeout(*timeout);
            }
            req = req.header(headers::NAME, "la.renzhen.trading");
            req = req.header(headers::VERSION, env!("CARGO_PKG_VERSION"));

            req = if cfg!(target_os = "windows") {
                req.header(headers::PLATFORM, "windows")
            } else if cfg!(target_os = "macos") {
                req.header(headers::PLATFORM, "macos")
            } else if cfg!(target_os = "linux") {
                req.header(headers::PLATFORM, "linux")
            } else {
                req.header(headers::PLATFORM, "unknown")
            };
            req
        }

        fn is_json_response(&self, resp: &Response) -> bool {
            if let Some(content_type) = resp.headers().get(header::CONTENT_TYPE) {
                if content_type.to_str().unwrap().contains("application/json") {
                    return true;
                }
            }
            false
        }

        fn is_ok(&self, resp: &Response) -> anyhow::Result<()> {
            if resp.status().is_success() {
                return Ok(());
            }
            anyhow::bail!("http error: {}", resp.status().to_string())
        }

        /// 添加验证信息
        fn sign(&self, mut req: RequestBuilder, data: &str) -> RequestBuilder {
            let timestamp = chrono::Local::now().timestamp_millis();
            req = req.header(headers::TIMESTAMP, timestamp);
            if let Some(credential) = &self.credential {
                let sign = sign(env!("CARGO_PKG_VERSION"), &credential.secret_key, data, &timestamp.to_string());
                req = req.header(headers::AK, credential.access_key.as_str());
                req = req.header(headers::SIGN, sign);
            }
            req
        }
    }

    /// 静态凭证提供者
    pub struct StaticCredentialProvider(Credential);

    impl CredentialProvider for StaticCredentialProvider {
        fn credential(&self) -> anyhow::Result<Option<Credential>> {
            Ok(Some(self.0.clone()))
        }
    }

    pub struct EnvCredentialProvider {
        access_key_env: String,
        secret_key_env: String,
    }

    impl EnvCredentialProvider {
        pub fn new(access_key_env: &str, secret_key_env: &str) -> Self {
            Self {
                access_key_env: access_key_env.to_string(),
                secret_key_env: secret_key_env.to_string(),
            }
        }
    }

    impl Default for EnvCredentialProvider {
        fn default() -> Self {
            Self::new("TRADING_ACCESS_KEY", "TRADING_SECRET_KEY")
        }
    }

    impl CredentialProvider for EnvCredentialProvider {
        fn credential(&self) -> anyhow::Result<Option<Credential>> {
            tracing::debug!("load credential from env");
            let access_key = std::env::var(&self.access_key_env)
                .context(format!("read access_key from env: {}", self.access_key_env))?;
            let secret_key = std::env::var(&self.secret_key_env)
                .context(format!("read secret_key from env: {}", self.secret_key_env))?;
            Ok(Some(Credential { access_key, secret_key }))
        }
    }

    pub struct FileCredentialProvider {
        path: PathBuf,
    }

    impl FileCredentialProvider {
        pub fn new(path: impl Into<PathBuf>) -> Self {
            Self { path: path.into() }
        }
    }

    impl Default for FileCredentialProvider {
        fn default() -> Self {
            let path = super::local::config_dir().unwrap().join("credential.json");
            Self::new(path)
        }
    }

    impl CredentialProvider for FileCredentialProvider {
        fn credential(&self) -> anyhow::Result<Option<Credential>> {
            tracing::debug!("load credential from {:?}", self.path);
            let content = std::fs::read_to_string(&self.path)?;
            let credential = serde_json::from_str::<Credential>(&content).context("deserialize credential file")?;
            Ok(Some(credential))
        }
    }

    pub struct MultiCredentialProvider {
        providers: Vec<Box<dyn CredentialProvider>>,
        must_found: bool,
    }

    impl Default for MultiCredentialProvider {
        fn default() -> Self {
            Self {
                providers: vec![
                    Box::new(EnvCredentialProvider::default()),
                    Box::new(FileCredentialProvider::default()),
                ],
                must_found: false,
            }
        }
    }

    impl MultiCredentialProvider {
        pub fn new(providers: Vec<Box<dyn CredentialProvider>>, must_found: bool) -> Self {
            Self { providers, must_found }
        }
    }

    impl CredentialProvider for MultiCredentialProvider {
        fn credential(&self) -> anyhow::Result<Option<Credential>> {
            for provider in &self.providers {
                match provider.credential() {
                    Err(err) => {
                        tracing::warn!("load credential failed: {}", err);
                    }
                    Ok(None) => {}
                    Ok(Some(credential)) => return Ok(Some(credential)),
                }
            }

            if self.must_found {
                anyhow::bail!("credential not found");
            }

            Ok(None)
        }
    }

    #[async_trait::async_trait]
    impl crate::StocksLoader for RemoteLoader {
        async fn stocks(&self) -> anyhow::Result<crate::stock::Stocks> {
            let req = self.request(Method::GET, "/stocks");
            let req = self.sign(req, "/stocks");

            let resp = req.send().await?;
            self.is_ok(&resp)?;
            if self.is_json_response(&resp) {
                let items = resp.json::<Vec<Stock>>().await?;
                return Ok(Stocks::new(items));
            } else {
                let content = resp.text().await?;
                super::local::parse_stocks_data(content)
            }
        }
    }

    #[async_trait::async_trait]
    impl MarketCurrentLoader for RemoteLoader {
        async fn market(&self) -> anyhow::Result<HashMap<String, Bar>> {
            let req = self.request(Method::GET, "/market");
            let req = self.sign(req, "/market");

            let resp = req.send().await?;
            self.is_ok(&resp)?;
            if self.is_json_response(&resp) {
                let output = resp.json::<HashMap<String, Bar>>().await?;
                return Ok(output);
            }
            let mut outputs = HashMap::new();

            let content = resp.text().await?;

            let lines = content.lines().skip(1);
            for line in lines.into_iter() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let mut fields = line.split(',');
                let name = fields.next().context("not found name")?;

                let mut bar = Bar::new(fields.next().context("not found date")?);
                bar.open = fields.next().context("not found open")?.parse::<f64>().context("parse open")?;
                bar.high = fields.next().context("not found high")?.parse::<f64>().context("parse high")?;
                bar.low = fields.next().context("not found low")?.parse::<f64>().context("parse low")?;
                bar.close = fields
                    .next()
                    .context("not found close")?
                    .parse::<f64>()
                    .context("parse close")?;
                bar.volume = fields
                    .next()
                    .context("not found volume")?
                    .parse::<f64>()
                    .context("parse volume")?;
                if !bar.is_ok() {
                    continue;
                }
                outputs.insert(name.to_string(), bar);
            }
            Ok(outputs)
        }
    }

    #[async_trait::async_trait]
    impl BarLoader for RemoteLoader {
        async fn current(&self, symbol: impl GetSymbolCode + Send) -> anyhow::Result<Bar> {
            let uri = format!("/current/{}", symbol.symbol());
            let req = self.request(Method::GET, &uri);
            let req = self.sign(req, &uri);
            let resp = req.send().await?;
            self.is_ok(&resp)?;
            if self.is_json_response(&resp) {
                let output = resp.json::<Bar>().await?;
                return Ok(output);
            }
            let content = resp.text().await?;
            let Some(line) = content.lines().skip(1).next() else {
                anyhow::bail!("not found");
            };

            if line.is_empty() {
                anyhow::bail!("closed");
            }

            let mut fields = line.split(',');
            let _name = fields.next().context("not found name")?;
            let mut bar = Bar::new(fields.next().context("not found date")?);
            bar.open = fields.next().context("not found open")?.parse::<f64>().context("parse open")?;
            bar.high = fields.next().context("not found high")?.parse::<f64>().context("parse high")?;
            bar.low = fields.next().context("not found low")?.parse::<f64>().context("parse low")?;
            bar.close = fields
                .next()
                .context("not found close")?
                .parse::<f64>()
                .context("parse close")?;
            bar.volume = fields
                .next()
                .context("not found volume")?
                .parse::<f64>()
                .context("parse volume")?;
            if !bar.is_ok() {
                anyhow::bail!("invailed data");
            }
            Ok(bar)
        }
    }

    #[async_trait::async_trait]
    impl ChartLoader for RemoteLoader {
        async fn chart(&self, param: impl Into<ChartParamter> + Send) -> anyhow::Result<Chart> {
            let param = param.into();
            let uri = format!("/chart/{}/{}", param.period.to_string(), &param.symbol);

            let mut params = HashMap::new();
            if let Some(limit) = &param.limit {
                params.insert("limit", limit.to_string());
            }
            if let Some(end) = &param.end {
                params.insert("end", end.to_string());
            }

            let req = self.request(Method::GET, &uri).query(&params);
            let req = self.sign(req, &uri);

            let resp = req.send().await?;
            self.is_ok(&resp)?;
            if self.is_json_response(&resp) {
                let output = resp.json::<Vec<Bar>>().await?;
                return Ok(Chart::new(output));
            }
            let mut items = vec![];
            let content = resp.text().await?;
            let lines = content.lines().skip(1);
            for line in lines.into_iter() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let mut fields = line.split(',');
                let mut bar = Bar::new(fields.next().context("not found date")?);
                bar.open = fields.next().context("not found open")?.parse::<f64>().context("parse open")?;
                bar.high = fields.next().context("not found high")?.parse::<f64>().context("parse high")?;
                bar.low = fields.next().context("not found low")?.parse::<f64>().context("parse low")?;
                bar.close = fields
                    .next()
                    .context("not found close")?
                    .parse::<f64>()
                    .context("parse close")?;
                bar.volume = fields
                    .next()
                    .context("not found volume")?
                    .parse::<f64>()
                    .context("parse volume")?;
                if !bar.is_ok() {
                    continue;
                }
                items.push(bar);
            }
            Ok(Chart::new(items))
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::StocksLoader;

        use super::*;

        #[tokio::test]
        #[ignore]
        async fn load_stocks() {
            let mut loader = RemoteLoader::default();
            let stocks = loader.stocks().await;
            assert!(stocks.is_err(), "load chart error");

            let provider = MultiCredentialProvider::default();
            loader.credential = provider.credential().unwrap();

            let stocks = loader.stocks().await;
            assert!(stocks.is_ok(), "load chart error");
            let stocks = stocks.unwrap();
            assert!(stocks.len() > 0, "load chart error");
        }

        #[tokio::test]
        #[ignore]
        async fn load_market() {
            let loader = RemoteLoader::default()
                .with_provider(MultiCredentialProvider::default())
                .unwrap();

            let market = loader.market().await;
            assert!(market.is_ok(), "load market error");
            let market = market.unwrap();
            assert!(market.len() > 0, "load market length is 0");

            let current = loader.current("601888").await;
            dbg!(&current);
            assert!(current.is_ok(), "load current error");
        }

        #[tokio::test]
        #[ignore]
        async fn load_chart() {
            let loader = RemoteLoader::default()
                .with_provider(MultiCredentialProvider::default())
                .unwrap();
            let param = ChartParamter::day("601888");
            let chart = loader.chart(param).await;
            assert!(chart.is_ok(), "load chart error");
        }
    }
}
