use ta::{Next, Reset};

use trading_data::Bar;

use crate::ValueIndex;

pub struct Indicator {
    rsi: Vec<ta::indicators::RelativeStrengthIndex>,
    args: Vec<usize>,
}

impl Indicator {
    fn new(args: Vec<usize>) -> Self {
        Self {
            rsi: args
                .clone()
                .into_iter()
                .map(ta::indicators::RelativeStrengthIndex::new)
                .flatten()
                .collect(),
            args,
        }
    }
}

impl Default for Indicator {
    fn default() -> Self {
        Indicator::new(vec![9, 9, 9])
    }
}

impl crate::Indicator for Indicator {
    fn name(&self) -> &'static str {
        "RSI"
    }

    fn arguments(&self) -> &Vec<usize> {
        &self.args
    }

    fn index(&self) -> Vec<ValueIndex> {
        self.args
            .iter()
            .enumerate()
            .map(|v| ValueIndex { name: format!("RSI{:02}", v.1), index: v.0 })
            .collect()
    }

    fn next(&mut self, bar: &Bar) -> Vec<f64> {
        self.rsi.iter_mut().map(|v| v.next(bar.close)).collect()
    }

    fn reset(&mut self) {
        for rsi in self.rsi.iter_mut() {
            rsi.reset()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use trading_data::{ChartLoader, LocalLoader, Stock, TradingDay};

    use crate::Indicator;

    #[tokio::test]
    async fn rsi() {
        let stock = Stock::new("", "601888");
        let loader = LocalLoader::base().unwrap();
        let chart = loader.chart(&stock).await;
        assert!(chart.is_ok());
        let chart = chart.unwrap();
        let mut kdj = super::Indicator::default();
        let day = (TradingDay::latest() - 10).to_string();
        for bar in chart.iter() {
            let kdj = kdj.next(bar);
            if bar.date.cmp(&day) == Ordering::Greater {
                println!("{}: {:?}", bar.date, kdj);
            }
        }
    }
}
