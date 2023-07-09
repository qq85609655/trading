use std::cmp::Ordering;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{deref, Percent, Period};
use crate::stock::GetSymbolCode;

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

impl Bar {
    pub fn merge(&mut self, bar: Bar) {
        self.volume += bar.volume;
        self.high = self.high.max(bar.high);
        self.low = self.low.min(bar.low);
        self.close = bar.close;
    }
}

#[derive(Clone, Debug, Default)]
pub struct Chart {
    items: Vec<Bar>,
    period: Period,
}

deref!(Chart, Vec<Bar>, items);

impl Chart {
    pub fn new(items: Vec<Bar>) -> Self {
        Self { items, period: Period::Day }
    }

    pub fn with_period(items: Vec<Bar>, period: Period) -> Self {
        Self { items, period }
    }

    pub fn period(&self) -> &Period {
        &self.period
    }

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
        if let Ok(index) = self.items.binary_search_by_key(&start_day, |d| &d.date) {
            self.items = self.items.split_off(index);
        }
    }

    /// 从后往前保持个数
    pub fn length(&mut self, length: usize) {
        if self.items.len() > length {
            self.items = self.items.split_off(self.items.len() - length);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChartParamter {
    pub period: Period,
    pub symbol: String,
    pub end: Option<String>,
    pub limit: Option<usize>,
}

impl ChartParamter {
    pub fn new(symbol: impl GetSymbolCode, period: Period) -> Self {
        Self { period, symbol: symbol.symbol().to_string(), limit: None, end: None }
    }

    pub fn day(symbol: impl GetSymbolCode) -> Self {
        Self::new(symbol, Period::Day)
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn end(mut self, end: impl ToString) -> Self {
        self.end = Some(end.to_string());
        self
    }
}

impl<T> From<T> for ChartParamter
    where T: GetSymbolCode
{
    fn from(value: T) -> Self {
        let symbol = value.symbol();
        ChartParamter::new(symbol, Period::Day)
    }
}


#[async_trait::async_trait]
pub trait ChartLoader {
    async fn chart(&self, param: impl Into<ChartParamter> + Send) -> anyhow::Result<Chart>;
}

#[async_trait::async_trait]
pub trait BarLoader {
    async fn current(&self, symbol: impl GetSymbolCode + Send) -> anyhow::Result<Bar>;
}

#[async_trait::async_trait]
pub trait MarketCurrentLoader {
    async fn market(&self) -> anyhow::Result<HashMap<String, Bar>>;
}
