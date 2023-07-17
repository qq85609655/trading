use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::deref;

pub trait GetSymbolCode {
    fn symbol(&self) -> &str;
}

impl GetSymbolCode for Stock {
    fn symbol(&self) -> &str {
        &self.symbol
    }
}

impl GetSymbolCode for String {
    fn symbol(&self) -> &str {
        &self
    }
}

impl GetSymbolCode for &str {
    fn symbol(&self) -> &str {
        &self
    }
}

impl<T: GetSymbolCode> GetSymbolCode for &T {
    fn symbol(&self) -> &str {
        (*self).symbol()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Stock {
    pub symbol: String,
    pub name: String,
}

impl Stock {
    pub fn new<T>(name: T, symbol: T) -> Self
    where
        T: ToString,
    {
        Self { symbol: symbol.to_string(), name: name.to_string() }
    }

    pub fn test() -> Self {
        Self::new("国机通用", "600444")
    }

    pub fn petty_display_name(&self) -> String {
        match self.name.len() {
            10 /*深圳吧A*/ | 11 /*三个AB*/ => format!(" {}", self.name),
            9 /*三个字*/ => format!("{}　", self.name),
            7 => format!("{}  ", self.name),
            6 => format!("{}　　", self.name),
            _ => format!("{}", self.name),
        }
    }
}

impl Display for Stock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("[{}]{}", self.symbol, self.petty_display_name()))
    }
}

#[async_trait::async_trait]
pub trait StocksLoader {
    async fn stocks(&self) -> anyhow::Result<Stocks>;
}

#[async_trait::async_trait]
impl<T: StocksLoader + std::marker::Sync> StocksLoader for &T {
    async fn stocks(&self) -> anyhow::Result<Stocks> {
        self.stocks().await
    }
}

deref! {
    #[derive(Debug, Clone)]
    pub struct Stocks(Vec<Stock>);
}

impl Stocks {
    pub fn sorted(mut self) -> Self {
        self.0.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        self
    }
    /// 加载的列表进行过了
    pub fn filter(mut self, filter: Option<String>) -> Self {
        let Some(filter) = filter else {
            return self;
        };
        self.0.retain(|item| {
            let symbol = item.symbol.clone();
            let name = item.name.clone();
            symbol.contains(&filter) || name.contains(&filter)
        });
        self
    }

    /// 模糊搜索股票并返回股票列表，搜索可根据名称和代码进行
    pub fn search(&self, filter: &str) -> Vec<&Stock> {
        self.0
            .iter()
            .filter(|&item| {
                let symbol = item.symbol.clone();
                let name = item.name.clone();
                symbol.contains(&filter) || name.contains(&filter)
            })
            .collect()
    }

    /// 随机一直股票，当且仅当股票列表不为空时，才会返回股票
    pub fn random(&self) -> Option<Stock> {
        if self.len() == 0 {
            return None;
        }
        let index = fastrand::usize(0..self.0.len());
        Some(self.0[index].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{Stock, Stocks};

    #[test]
    fn test_filter() {
        let stocks = Stocks::new(vec![
            Stock::new("国机通用", "600444"),
            Stock::new("国电电力", "600795"),
            Stock::new("中国中免", "601888"),
        ]);
        let f = stocks.search("国机");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].symbol, "600444");

        let f = stocks.search("国电");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].symbol, "600795");

        let f = stocks.search("国");
        assert_eq!(f.len(), 3);
        assert_eq!(f[0].symbol, "600444");
        assert_eq!(f[1].symbol, "600795");
        assert_eq!(f[2].symbol, "601888");

        let f = stocks.search("60");
        assert_eq!(f.len(), 3);

        let r = stocks.random();
        assert!(r.is_some());

        let stocks = stocks.filter(Some("国机".to_string()));
        assert_eq!(stocks.len(), 1);
    }
}
