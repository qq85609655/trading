use trading_data::Bar;

pub mod average;
pub mod kdj;
pub mod macd;
pub mod rsi;
pub mod volume;

#[derive(Debug, Clone)]
pub struct ValueIndex {
    pub name: String,
    pub index: usize,
}

impl ValueIndex {
    pub fn new(name: String, index: usize) -> Self {
        Self { name, index }
    }

    pub fn value(&self, items: &Vec<f64>) -> f64 {
        items[self.index]
    }
}

pub trait Indicator {
    fn name(&self) -> &'static str;

    fn arguments(&self) -> &Vec<usize>;

    fn index(&self) -> Vec<ValueIndex>;

    fn next(&mut self, bar: &Bar) -> Vec<f64>;

    fn reset(&mut self) {}
}
