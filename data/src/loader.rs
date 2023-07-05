/// 定义了数据加载器

/// 本地数据加载器
pub mod local {
    use std::path::PathBuf;

    use anyhow::Context;

    use crate::stock::GetSymbolCode;
    use crate::{Stock, Stocks, StocksLoader};

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
        Ok(Stocks::new(stocks))
    }

    #[derive(Debug, Clone)]
    pub struct LocalLoader {
        base_dir: PathBuf,
    }

    impl LocalLoader {
        fn default() -> anyhow::Result<Self> {
            data_dir().and_then(|path| LocalLoader::new(path))
        }

        pub fn new(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
            let path = path.into();
            let test_writeable = path.join("test");
            std::fs::write(&test_writeable, "").context("Failed to write local folder")?;
            std::fs::remove_file(&test_writeable).context("Failed to remove local folder")?;

            Ok(Self { base_dir: path.into() })
        }

        pub fn stock_path(&self, symbol: impl GetSymbolCode) -> anyhow::Result<PathBuf> {
            let symbol = symbol.symbol();
            let (s1, s2) = (&symbol[..2], &symbol[2..4]);
            self.storage(format!("stocks/{}/{}/{}.csv", s1, s2, symbol))
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        #[ignore]
        fn show_dirs() {
            let path = data_dir().unwrap();
            println!("data_dir: {:?}", path);

            let loader = LocalLoader::default().unwrap();
            let path = loader.stock_path("600444").unwrap();
            println!("stock: {:?}", path);

            let path = loader.stocks_path().unwrap();
            println!("stocks: {:?}", path);
        }

        #[tokio::test]
        #[ignore]
        async fn load_stocks() {
            let loader = LocalLoader::default().unwrap();
            let stocks = loader.stocks().await;
            assert!(stocks.is_ok(), "load chart error");
            let stocks = stocks.unwrap();
            assert!(stocks.len() > 0, "load chart error");
        }
    }
}

/// 远程加载器
pub mod remote {
    use std::path::PathBuf;

    use anyhow::Context;
    use reqwest::{header, Method, RequestBuilder, Response};
    use serde::{Deserialize, Serialize};

    use crate::{Stock, Stocks};

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
    }
}
