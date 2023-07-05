use std::cmp::Ordering;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::stock::GetSymbolCode;
use crate::{deref, Percent};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Bar {
    #[serde(alias = "time")]
    pub date: String,

    pub open: f64,
    pub high: f64,
    pub low: f64,
    #[serde(alias = "price")]
    pub close: f64,
    pub volume: f64,
    pub yesterday: f64,
}

impl PartialEq<Self> for Bar {
    fn eq(&self, other: &Self) -> bool {
        self.date.eq(&other.date)
    }
}

impl Eq for Bar {}

impl PartialOrd<Self> for Bar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl Ord for Bar {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

impl Bar {
    pub fn new(date: &str) -> Bar {
        Bar { date: date.to_string(), ..Default::default() }
    }

    pub fn random(date: &str, min: f64, max: f64) -> Bar {
        let open = fastrand::f64() * (max - min) + min;
        let high = open * (fastrand::f64() / 10.0 + 1.0);
        let low = open * (1.0 - fastrand::f64() / 10.0);
        let close = fastrand::f64() * (high - low) + low;
        Self { date: date.to_string(), open, high, low, close, ..Default::default() }
    }

    pub fn is_ok(&self) -> bool {
        self.open > 0.0 && self.high > 0.0 && self.low > 0.0 && self.close > 0.0
    }

    pub fn red(&self) -> bool {
        self.close >= self.open
    }

    pub fn green(&self) -> bool {
        self.close < self.open
    }

    pub fn markup(&self) -> f64 {
        if self.yesterday == 0.0 {
            self.close.percent(self.open)
        } else {
            self.close.percent(self.yesterday)
        }
    }

    pub fn amplitude(&self) -> f64 {
        (self.high - self.low) / self.yesterday * 100.0
    }
}

deref! {
    #[derive(Clone, Debug, Default)]
    pub struct Chart(Vec<Bar>);
}

impl Chart {
    pub fn replace_last(&mut self, bar: Bar) {
        if !self.is_empty() {
            let index = self.len() - 1;
            self.remove(index);
        }
        self.push(bar);
    }

    pub fn search(&self, date: &str) -> Option<&Bar> {
        match self.binary_search_by_key(&date, |f| &f.date) {
            Ok(index) => self.get(index),
            _ => None,
        }
    }

    // limit 最大限制,并且返回限制日期数据
    pub fn limit(&mut self, end_day: &str) -> Option<Bar> {
        let mut current = None;
        if let Ok(index) = self.binary_search_by_key(&end_day, |d| &d.date) {
            current = Some(self.remove(index));
            unsafe {
                self.set_len(index);
            }
        }
        current
    }

    // skip 设置开始时间
    pub fn offset(&mut self, start_day: &str) {
        if let Ok(index) = self.binary_search_by_key(&start_day, |d| &d.date) {
            *self = Self::new(self.0.split_off(index));
        }
    }
}

#[async_trait::async_trait]
pub trait Loader {
    async fn chart(&self, symbol: impl GetSymbolCode + Send) -> anyhow::Result<Chart>;
}

#[async_trait::async_trait]
pub trait BarLoader {
    async fn current(&self, symbol: impl GetSymbolCode + Send) -> anyhow::Result<Bar>;
}

#[async_trait::async_trait]
pub trait MarketCurrentLoader {
    async fn market(&self) -> anyhow::Result<HashMap<String, Bar>>;
}
