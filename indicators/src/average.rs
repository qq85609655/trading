use crate::ValueIndex;
use trading_data::Bar;
use trading_ext::LatestIterExt;

#[derive(Clone)]
pub struct Indicator {
    args: Vec<usize>,
    bars: Vec<f64>,
    count: usize,
}

impl Default for Indicator {
    fn default() -> Self {
        Indicator::new(vec![3, 5, 8, 13, 21, 34, 55, 89, 144, 233])
    }
}

impl Indicator {
    fn new(args: Vec<usize>) -> Self {
        Self { count: args.iter().max_by(|a, b| a.cmp(b)).unwrap().clone(), args, bars: vec![] }
    }

    fn avg(&self, limit: &usize) -> f64 {
        let limit = *limit;
        if self.bars.len() >= limit {
            self.bars.latest_iter(limit, 0).sum::<f64>() / limit as f64
        } else {
            self.bars.latest_iter(limit, 0).sum::<f64>() / self.bars.len() as f64
        }
    }
}

impl crate::Indicator for Indicator {
    fn name(&self) -> &'static str {
        "MA"
    }

    fn arguments(&self) -> &Vec<usize> {
        &self.args
    }

    fn index(&self) -> Vec<ValueIndex> {
        self.args
            .iter()
            .enumerate()
            .map(|v| ValueIndex { name: format!("MA{:02}", v.1), index: v.0 })
            .collect()
    }

    fn next(&mut self, bar: &Bar) -> Vec<f64> {
        self.bars.push(bar.close);
        if self.bars.len() > self.count {
            self.bars.remove(0);
        }
        self.args.iter().map(|v| self.avg(v)).collect()
    }

    fn reset(&mut self) {
        self.bars.clear();
    }
}

#[cfg(test)]
mod tests {
    use trading_data::Bar;

    use crate::Indicator;

    #[test]
    fn avg() {
        let mut avg = super::Indicator::default();
        for i in 1..=20 {
            let mut bar = Bar::new("");
            bar.close = i as f64;
            let out = avg.next(&bar);
            println!("{}: {:?}", i, out);
        }
    }
}
