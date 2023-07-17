use std::string::ToString;

use lazy_static::lazy_static;
use ta::Next;

use trading_data::Bar;

use crate::ValueIndex;

#[derive(Debug, Clone)]
struct Chip {
    period: usize,
    value: f64,
}

impl Chip {
    pub fn new(period: usize) -> Self {
        Self { period, value: 0.0 }
    }
}

impl Next<f64> for Chip {
    type Output = f64;
    fn next(&mut self, input: f64) -> Self::Output {
        let v = (self.value * 2.0 + input * 1.0) / self.period as f64;
        self.value = v;
        v
    }
}

#[derive(Debug, Clone)]
pub struct Indicator {
    args: Vec<usize>,
    n: usize,
    items: Vec<Bar>,
    k: Chip,
    d: Chip,
}

impl Indicator {
    fn new(args: Vec<usize>) -> Self {
        Self { n: args[0], k: Chip::new(args[1]), d: Chip::new(args[2]), items: vec![], args }
    }

    fn rsv_value(&self) -> f64 {
        // CLOSE
        let close = self.items.last().unwrap().close;
        // LLV(LOW,P1)
        let low = self.items.iter().min_by(|&a, &b| a.low.total_cmp(&b.low)).unwrap().low;
        // HHV(HIGH,P1)
        let high = self.items.iter().max_by(|&a, &b| a.high.total_cmp(&b.high)).unwrap().high;

        // RSV:=(CLOSE-LLV(LOW,P1))/(HHV(HIGH,P1)-LLV(LOW,P1))*100;
        let rsv = (close - low) / (high - low) * 100.0;

        if rsv > 100.0 {
            100.0
        } else if rsv < 0.0 {
            0.0
        } else {
            rsv
        }
    }
}

impl Default for Indicator {
    fn default() -> Self {
        Indicator::new(vec![9, 3, 3])
    }
}

lazy_static! {
    pub static ref K: ValueIndex = ValueIndex::new("K".to_string(), 0);
    pub static ref D: ValueIndex = ValueIndex::new("D".to_string(), 1);
    pub static ref J: ValueIndex = ValueIndex::new("J".to_string(), 2);
}

impl crate::Indicator for Indicator {
    fn name(&self) -> &'static str {
        "KDJ"
    }

    fn arguments(&self) -> &Vec<usize> {
        &self.args
    }

    fn index(&self) -> Vec<ValueIndex> {
        vec![K.clone(), D.clone(), J.clone()]
    }

    fn next(&mut self, bar: &Bar) -> Vec<f64> {
        self.items.push(bar.clone());
        if self.items.len() > self.n {
            self.items.remove(0);
        }

        let rsv = self.rsv_value();

        // K:SMA(RSV,P2,1);
        let k = self.k.next(rsv);

        // D:SMA(K,P3,1);
        let d = self.d.next(k);

        // J:3*K-2*D
        let j = (k * 3.0) - (d * 2.0);
        //Output { rsv, k, d, j }
        vec![k, d, j]
    }

    fn reset(&mut self) {
        self.items.clear()
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use trading_data::{ChartLoader, LocalLoader, Stock, TradingDay};

    use crate::Indicator;

    #[tokio::test]
    async fn kdj() {
        let stock = Stock::new("", "601888");
        let loader = LocalLoader::base().unwrap();
        let chart = loader.chart(&stock).await;
        assert!(chart.is_ok());
        let chart = chart.unwrap();
        let mut kdj = super::Indicator::new(vec![9, 3, 3]);
        let day = (TradingDay::latest() - 10).to_string();
        for bar in chart.iter() {
            let kdj = kdj.next(bar);
            if bar.date.cmp(&day) == Ordering::Greater {
                println!("{}: {:?}", bar.date, kdj);
            }
        }
    }
}
