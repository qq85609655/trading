use lazy_static::lazy_static;
use std::string::ToString;
use ta::{Next, Reset};

use crate::ValueIndex;
use trading_data::Bar;

#[derive(Debug, Clone)]
pub struct Indicator {
    args: Vec<usize>,
    long_period: usize,
    short_ema: ta::indicators::ExponentialMovingAverage,
    long_ema: ta::indicators::ExponentialMovingAverage,
    signal_ema: ta::indicators::ExponentialMovingAverage,
    current: usize,
}

impl Indicator {
    fn new(args: Vec<usize>) -> Self {
        Self {
            long_period: args[1],
            short_ema: ta::indicators::ExponentialMovingAverage::new(args[0]).unwrap(),
            long_ema: ta::indicators::ExponentialMovingAverage::new(args[1]).unwrap(),
            signal_ema: ta::indicators::ExponentialMovingAverage::new(args[2]).unwrap(),
            current: 0,
            args,
        }
    }
}

impl Default for Indicator {
    fn default() -> Self {
        Indicator::new(vec![12, 26, 9])
    }
}

lazy_static! {
    pub static ref DIFF: ValueIndex = ValueIndex::new("DIFF".to_string(), 0);
    pub static ref DEA: ValueIndex = ValueIndex::new("DEA".to_string(), 1);
    pub static ref MACD: ValueIndex = ValueIndex::new("MACD".to_string(), 2);
}

impl crate::Indicator for Indicator {
    fn name(&self) -> &'static str {
        "MACD"
    }

    fn arguments(&self) -> &Vec<usize> {
        &self.args
    }

    fn index(&self) -> Vec<ValueIndex> {
        vec![DIFF.clone(), DEA.clone(), MACD.clone()]
    }

    fn next(&mut self, bar: &Bar) -> Vec<f64> {
        self.current = self.current + 1;
        let short = self.short_ema.next(bar.close);
        let long = self.long_ema.next(bar.close);

        if self.current < self.long_period {
            return vec![0.0, 0.0, 0.0];
        }

        let diff = short - long;
        let dea = self.signal_ema.next(diff);
        let macd = (diff - dea) * 2.0;

        vec![diff, dea, macd]
    }

    fn reset(&mut self) {
        self.long_ema.reset();
        self.short_ema.reset();
        self.signal_ema.reset();
        self.current = 0;
    }
}

#[cfg(test)]
mod tests {
    use trading_data::Bar;

    use crate::Indicator;

    #[test]
    fn macd() {
        let mut avg = super::Indicator::default();
        for i in 1..=100 {
            let mut bar = Bar::new("");
            bar.close = i as f64;
            let out = avg.next(&bar);
            println!("{}: {:?}", i, out);
        }
    }
}
