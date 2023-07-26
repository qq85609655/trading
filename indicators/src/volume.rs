use lazy_static::lazy_static;

use trading_data::Bar;

use crate::ValueIndex;

lazy_static! {
    pub static ref VOL: ValueIndex = ValueIndex::new(format!("VOL"), 0);
    pub static ref MAVOL: ValueIndex = ValueIndex::new(format!("MAVOL"), 1);
}

pub struct Indicator {
    count: usize,
    items: Vec<f64>,
    args: Vec<usize>,
}

impl Default for Indicator {
    fn default() -> Indicator {
        Indicator::new(vec![100])
    }
}

impl Indicator {
    pub fn new(args: Vec<usize>) -> Self {
        Self { count: args[0], items: vec![], args }
    }
}

impl crate::Indicator for Indicator {
    fn name(&self) -> &'static str {
        "VOL"
    }

    fn arguments(&self) -> &Vec<usize> {
        &self.args
    }

    fn index(&self) -> Vec<ValueIndex> {
        vec![VOL.clone(), MAVOL.clone()]
    }

    fn next(&mut self, bar: &Bar) -> Vec<f64> {
        self.items.push(bar.volume);
        if self.items.len() > self.count {
            self.items.remove(0);
        }

        let volume = if bar.close >= bar.open { bar.volume } else { 0.0 - bar.volume };
        let avg = self.items.iter().sum::<f64>() / self.items.len() as f64;
        vec![volume, avg]
    }

    fn reset(&mut self) {
        self.items.clear();
    }
}
