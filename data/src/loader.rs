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
            let content = tokio::fs::read_to_string(path).await
                .context("Failed to read stock file")?;
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
    use reqwest::header;

    use crate::{Stock, Stocks};

    #[derive(Debug, Clone)]
    pub struct StocksRemoteLoader {
        pub host: String,
        pub token: Option<String>,
        pub timeout: Option<std::time::Duration>,
    }

    impl Default for StocksRemoteLoader {
        fn default() -> Self {
            Self {
                host: "http://127.0.0.1:18686".to_string(),
                token: None,
                timeout: Some(std::time::Duration::from_secs(3)),
            }
        }
    }

    #[async_trait::async_trait]
    impl crate::StocksLoader for StocksRemoteLoader {
        async fn stocks(&self) -> anyhow::Result<crate::stock::Stocks> {
            let url = format!("{}/data/stocks", self.host);
            let mut req = reqwest::Client::new().get(&url);
            if let Some(token) = &self.token {
                req = req.bearer_auth(token);
            }
            if let Some(timeout) = &self.timeout {
                req = req.timeout(*timeout);
            }
            let resp = req.send().await?;

            if let Some(content_type) = resp.headers().get(header::CONTENT_TYPE) {
                if content_type.to_str().unwrap().contains("application/json") {
                    let items = resp.json::<Vec<Stock>>().await?;
                    return Ok(Stocks::new(items));
                }
            }

            let content = resp.text().await?;
            super::local::parse_stocks_data(content)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::StocksLoader;

        use super::*;

        #[tokio::test]
        #[ignore]
        async fn load_stocks() {
            let mut loader = StocksRemoteLoader::default();
            let stocks = loader.stocks().await;
            assert!(stocks.is_err(), "load chart error");

            loader.token = Some(String::from("haiker"));
            let stocks = loader.stocks().await;
            assert!(stocks.is_ok(), "load chart error");
            let stocks = stocks.unwrap();
            assert!(stocks.len() > 0, "load chart error");
        }
    }
}
