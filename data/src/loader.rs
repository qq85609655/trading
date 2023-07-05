/// 定义了数据加载器

/// 本地数据加载器
pub mod local {
    use std::path::PathBuf;

    use anyhow::Context;

    use crate::{Stock, Stocks, StocksLoader};
    use crate::stock::GetSymbolCode;

    pub fn data_dir() -> anyhow::Result<PathBuf> {
        let dir = dirs::data_local_dir().ok_or(anyhow::anyhow!("data local dir not found"))?;
        let dir = dir.join("la.renzhen.trading");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).context("Failed to create data local dir")?;
        }
        Ok(dir)
    }

    #[allow(dead_code)]
    fn stock(symbol: impl GetSymbolCode) -> anyhow::Result<PathBuf> {
        let symbol = symbol.symbol();
        let (s1, s2) = (&symbol[..2], &symbol[2..4]);
        storage(format!("stocks/{}/{}/{}.csv", s1, s2, symbol).as_str())
    }

    fn stocks() -> anyhow::Result<PathBuf> {
        storage("stocks.csv")
    }

    pub fn storage(filename: &str) -> anyhow::Result<PathBuf> {
        let mut path = data_dir()?;
        path.push(filename);
        Ok(path)
    }

    pub(crate) fn parse_stocks_data(content: String) -> anyhow::Result<Stocks> {
        let mut stocks = vec![];
        for line in content.lines().skip(1) {
            let mut it = line.split("\t");
            let symbol = it.next().context("can't found stock symbol")?;
            let name = it.next().context("can't found stock name")?;
            stocks.push(Stock::new(name, symbol))
        }
        Ok(Stocks::new(stocks))
    }

    #[derive(Debug, Clone)]
    pub struct StocksLocalLoader;

    #[async_trait::async_trait]
    impl StocksLoader for StocksLocalLoader {
        async fn stocks(&self) -> anyhow::Result<Stocks> {
            let path = stocks()?;
            let content = tokio::fs::read_to_string(path).await.context("Failed to read stock file")?;
            parse_stocks_data(content)
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

            let path = stock("600444").unwrap();
            println!("stock: {:?}", path);

            let path = stocks().unwrap();
            println!("stocks: {:?}", path);
        }

        #[tokio::test]
        #[ignore]
        async fn load_stocks() {
            let loader = StocksLocalLoader;
            let stocks = loader.stocks().await;
            assert!(stocks.is_ok(), "load chart error");
            let stocks = stocks.unwrap();
            assert!(stocks.len() > 0, "load chart error");
        }
    }
}

/// 远程加载器
pub mod remote {
    use reqwest::{header, Method, RequestBuilder, Response};
    use serde::{Deserialize, Serialize};

    use crate::{Stock, Stocks};

    pub mod headers {
        pub const NAME: &str = "X-Trading-Name";
        pub const VERSION: &str = "X-Trading-Version";
        pub const PLATFORM: &str = "X-Trading-Platform";
        pub const AK: &str = "X-Trading-AccessKey";
        pub const SIGN: &str = "X-Trading-Sign";
        pub const TIMESTAMP: &str = "X-Trading-Timestamp";
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
    pub struct StocksRemoteLoader {
        pub host: String,
        pub credential: Option<Credential>,
        pub timeout: Option<std::time::Duration>,
    }

    impl Default for StocksRemoteLoader {
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

    impl StocksRemoteLoader {
        pub fn new<P: CredentialProvider>(host: &str, timeout: Option<std::time::Duration>, provider: P) -> anyhow::Result<Self> {
            Ok(Self {
                host: host.to_string(),
                credential: provider.credential()?,
                timeout,
            })
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

        /// 添加验证信息
        fn sign(&self, mut req: RequestBuilder, data: &str) -> RequestBuilder {
            let timestamp = chrono::Local::now().timestamp_millis();
            req = req.header(headers::TIMESTAMP, timestamp);
            if let Some(credential) = &self.credential {
                let sign = sign(env!("CARGO_PKG_VERSION"), &credential.access_key, data, &timestamp.to_string());
                req = req.header(headers::AK, credential.access_key.as_str());
                req = req.header(headers::SIGN, sign);
            }
            req
        }
    }

    #[async_trait::async_trait]
    impl crate::StocksLoader for StocksRemoteLoader {
        async fn stocks(&self) -> anyhow::Result<crate::stock::Stocks> {
            let req = self.request(Method::GET, "/stocks");
            let req = self.sign(req, "");

            let resp = req.send().await?;
            if self.is_json_response(&resp) {
                let items = resp.json::<Vec<Stock>>().await?;
                return Ok(Stocks::new(items));
            } else {
                let content = resp.text().await?;
                super::local::parse_stocks_data(content)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::StocksLoader;

        use super::*;

        #[tokio::test]
        #[ignore]
        async fn load_stocks() {
            let loader = StocksRemoteLoader::default();
            // let stocks = loader.stocks().await;
            // assert!(stocks.is_err(), "load chart error");

            let stocks = loader.stocks().await;
            assert!(stocks.is_ok(), "load chart error");
            let stocks = stocks.unwrap();
            assert!(stocks.len() > 0, "load chart error");
        }
    }
}
