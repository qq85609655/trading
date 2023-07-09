use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;

use anyhow::{bail, Context};
use chrono::{Datelike, DateTime, Duration, DurationRound, Local, Timelike, TimeZone};

pub mod holidays {
    use std::cmp::Ordering;
    use std::ops::Add;

    use anyhow::{Context, Result};
    use chrono::{Datelike, DateTime, Duration, Local};
    use lazy_static::lazy_static;

    pub const FORMAT: &str = "%Y-%m-%d";
    pub const MONTH_FORMAT: &str = "%Y-%m";
    pub const FULL_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    #[derive(Debug)]
    pub struct Holiday {
        start: &'static str,
        end: &'static str,
    }

    impl Holiday {
        fn new(start: &'static str, end: &'static str) -> Holiday {
            Holiday { start, end }
        }

        fn is_holiday(&self, date: &str) -> bool {
            let date = &date[0..10];
            date >= self.start && date <= self.end
        }
    }

    type Holidays = Vec<Holiday>;

    /// Returns a list of holidays for the given year.
    fn holidays() -> Holidays {
        let mut holidays = Holidays::new();

        //2016
        holidays.push(Holiday::new("2016-01-01", "2016-01-03")); //元旦
        holidays.push(Holiday::new("2016-02-07", "2016-02-13")); //春节
        holidays.push(Holiday::new("2016-04-02", "2016-04-04")); //清明节
        holidays.push(Holiday::new("2016-04-29", "2016-05-01")); //劳动节
        holidays.push(Holiday::new("2016-06-09", "2016-06-11")); //端午节
        holidays.push(Holiday::new("2016-09-15", "2016-09-17")); //中秋节
        holidays.push(Holiday::new("2016-10-01", "2016-10-07")); //国庆节

        //2017
        holidays.push(Holiday::new("2017-01-01", "2017-01-03")); //元旦
        holidays.push(Holiday::new("2017-01-27", "2017-02-02")); //春节
        holidays.push(Holiday::new("2017-04-02", "2017-04-04")); //清明节
        holidays.push(Holiday::new("2017-04-29", "2017-05-01")); //劳动节
        holidays.push(Holiday::new("2017-05-28", "2017-05-30")); //端午节
        holidays.push(Holiday::new("2017-10-01", "2017-10-07")); //国庆节

        //2018
        holidays.push(Holiday::new("2018-01-01", "2018-01-03")); //元旦
        holidays.push(Holiday::new("2018-02-15", "2018-02-21")); //春节
        holidays.push(Holiday::new("2018-04-05", "2018-04-07")); //清明节
        holidays.push(Holiday::new("2018-04-29", "2018-05-01")); //劳动节
        holidays.push(Holiday::new("2018-06-16", "2018-06-18")); //端午节
        holidays.push(Holiday::new("2018-09-22", "2018-09-24")); //中秋节
        holidays.push(Holiday::new("2018-09-29", "2018-10-07")); //国庆节

        //2019
        holidays.push(Holiday::new("2019-01-01", "2019-01-03")); //元旦
        holidays.push(Holiday::new("2019-02-04", "2019-02-10")); //春节
        holidays.push(Holiday::new("2019-04-05", "2019-04-07")); //清明节
        holidays.push(Holiday::new("2019-04-29", "2019-05-01")); //劳动节
        holidays.push(Holiday::new("2019-06-07", "2019-06-09")); //端午节
        holidays.push(Holiday::new("2019-09-13", "2019-09-15")); //中秋节
        holidays.push(Holiday::new("2019-10-01", "2019-10-07")); //国庆节

        //2020
        holidays.push(Holiday::new("2020-01-01", "2020-01-03")); //元旦
        holidays.push(Holiday::new("2020-01-24", "2020-01-30")); //春节
        holidays.push(Holiday::new("2020-04-04", "2020-04-06")); //清明节
        holidays.push(Holiday::new("2020-04-30", "2020-05-04")); //劳动节
        holidays.push(Holiday::new("2020-06-25", "2020-06-27")); //端午节
        holidays.push(Holiday::new("2020-09-30", "2020-10-07")); //国庆节

        //2021
        holidays.push(Holiday::new("2021-01-01", "2021-01-03")); //元旦
        holidays.push(Holiday::new("2021-02-11", "2021-02-17")); //春节
        holidays.push(Holiday::new("2021-04-03", "2021-04-05")); //清明节
        holidays.push(Holiday::new("2021-04-30", "2021-05-04")); //劳动节
        holidays.push(Holiday::new("2021-06-12", "2021-06-14")); //端午节
        holidays.push(Holiday::new("2021-09-19", "2021-09-21")); //中秋节
        holidays.push(Holiday::new("2021-10-01", "2021-10-07")); //国庆节

        //2022
        holidays.push(Holiday::new("2022-01-01", "2022-01-03")); //元旦
        holidays.push(Holiday::new("2022-01-31", "2022-02-06")); //春节
        holidays.push(Holiday::new("2022-04-03", "2022-04-05")); //清明节
        holidays.push(Holiday::new("2022-04-30", "2022-05-04")); //劳动节
        holidays.push(Holiday::new("2022-06-03", "2022-06-05")); //端午节
        holidays.push(Holiday::new("2022-09-10", "2022-09-12")); //中秋节
        holidays.push(Holiday::new("2022-10-01", "2022-10-07")); //国庆节

        //2023
        holidays.push(Holiday::new("2022-12-31", "2023-01-02")); //元旦
        holidays.push(Holiday::new("2023-01-21", "2023-01-27")); //春节
        holidays.push(Holiday::new("2023-04-05", "2023-04-05")); //清明节
        holidays.push(Holiday::new("2023-05-01", "2023-05-03")); //劳动节
        holidays.push(Holiday::new("2023-06-22", "2023-06-24")); //端午节
        holidays.push(Holiday::new("2023-09-29", "2023-10-06")); //中秋节
        holidays.push(Holiday::new("2023-10-01", "2023-10-07")); //国庆节

        holidays
    }

    lazy_static! {
        pub static ref HOLIDAYS: Holidays = holidays();
    }

    /// Returns Whether it's a days or not.
    pub fn is_holiday(date: &str) -> bool {
        let date = &date[0..10];
        HOLIDAYS.iter().any(|holiday| holiday.is_holiday(date))
    }

    pub fn not_holiday(date: &str) -> bool {
        let date = &date[0..10];
        !is_holiday(date)
    }

    pub fn is_weekend(date: &str) -> Result<bool> {
        let date = &date[0..10];
        let weekday = chrono::NaiveDate::parse_from_str(date, FORMAT)
            .context("Invalid date")?
            .weekday();
        Ok(weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun)
    }

    /// 返回指定日期是否是一个节假日
    pub fn is_trading_day(date: &str) -> Result<bool> {
        let date = &date[0..10];
        Ok(!is_weekend(date)? && not_holiday(date))
    }

    pub fn today_is_trading_day() -> Result<bool> {
        let today = chrono::Local::now().format(FORMAT).to_string();
        is_trading_day(&today)
    }

    /// 移动到交易日
    pub fn to_trading_day(date: DateTime<Local>, order: Ordering) -> DateTime<Local> {
        let mut date = date.clone();
        while !is_trading_day(&date.format(FORMAT).to_string()).unwrap() {
            date = date.add(match order {
                Ordering::Less => Duration::days(-1),
                Ordering::Greater => Duration::days(1),
                Ordering::Equal => Duration::days(0),
            });
        }
        date
    }

    #[cfg(test)]
    mod tests {
        use crate::{is_holiday, is_trading_day, is_weekend};

        #[test]
        fn test_holidays() {
            assert!(is_holiday("2020-01-01"));
            assert!(is_weekend("2020-01-04").unwrap());
            assert!(is_trading_day("2020-01-06").unwrap());
        }
    }
}

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Period {
    Minute(usize),
    #[default]
    Day,
    Week,
}

impl FromStr for Period {
    type Err = anyhow::Error;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "day" => Ok(Period::Day),
            "week" => Ok(Period::Week),
            _ => {
                if string.ends_with("m") {
                    Ok(Period::Minute(string[..string.len() - 1].parse::<usize>().context("invaild period value")?))
                } else {
                    bail!("invaild period value")
                }
            }
        }
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Period::Day => write!(f, "day"),
            Period::Week => write!(f, "week"),
            Period::Minute(minutes) => write!(f, "{}m", minutes),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct TradingDay {
    period: Period,
    date: DateTime<Local>,
}

impl FromStr for TradingDay {
    type Err = chrono::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 10 {
            let date = Local.datetime_from_str(format!("{} 00:00:00", s).as_str(), crate::FULL_FORMAT)?;
            let date = holidays::to_trading_day(date, Ordering::Less);
            Ok(TradingDay::new(Period::Day, date))
        } else {
            let date = Local.datetime_from_str(format!("{}:00", s).as_str(), crate::FULL_FORMAT)?;
            let date = holidays::to_trading_day(date, Ordering::Less);
            Ok(TradingDay::new(Period::Minute(5), date))
        }
    }
}

impl TradingDay {
    pub fn new(period: Period, date: DateTime<Local>) -> Self {
        Self { period, date }
    }

    pub fn period(&self) -> &Period {
        &self.period
    }

    pub fn day(date: DateTime<Local>) -> Self {
        Self::new(Period::Day, date)
    }

    pub fn trading(day: Option<String>) -> anyhow::Result<Self> {
        match day {
            Some(day) => FromStr::from_str(&day).context("Invalid trading day"),
            None => Ok(TradingDay::latest()),
        }
    }

    pub fn latest() -> Self {
        let now = Local::now();
        let today = now.duration_trunc(Duration::days(1)).unwrap();
        let mut latest = Self::day(holidays::to_trading_day(today, Ordering::Less));
        let open = latest.open_time();
        if now.lt(&open.date) {
            latest = latest.previous();
        }
        latest
    }

    pub fn format(&self, pattern: &str) -> String {
        self.date.format(pattern).to_string()
    }
}

impl TradingDay {
    /// Returns the previous trading day.
    pub fn previous(&self) -> Self {
        Self::day(holidays::to_trading_day(self.date.sub(Duration::days(1)), Ordering::Less))
    }

    /// Returns the next trading day.
    pub fn next(&self) -> Self {
        Self::day(holidays::to_trading_day(self.date.add(Duration::days(1)), Ordering::Greater))
    }

    pub fn between(&self, other: &TradingDay) -> usize {
        let mut days: usize = 0;
        let (mut min, max) = match self.cmp(other) {
            Ordering::Less => (self.clone(), other),
            Ordering::Greater => (other.clone(), self),
            Ordering::Equal => return 0,
        };
        while min.lt(&max) {
            min = min.next();
            days += 1;
        }
        days
    }

    pub fn days(&self, other: &TradingDay) -> usize {
        self.between(&other)
    }

    pub fn open_time(&self) -> Self {
        let date = self.date.with_hour(9).map(|v| v.with_minute(30)).flatten().unwrap();
        Self::new(Period::Minute(5), date)
    }

    pub fn close_time(&self) -> Self {
        let date = self.date.with_hour(15).map(|v| v.with_minute(0)).flatten().unwrap();
        Self::new(Period::Minute(5), date)
    }

    pub fn is_now_closed(&self) -> bool {
        let now = Local::now();
        self.close_time().date.lt(&now)
    }

    pub fn label(&self, end: &Self) -> Vec<TradingDay> {
        let mut items = vec![self.clone()];
        let mut start = self.clone();
        loop {
            if start.date.cmp(&end.date) != Ordering::Less {
                break;
            }
            start = start.sub(1);
            items.push(start.clone());
        }
        items.push(end.clone());
        items
    }
}

impl TradingDay {
    pub fn week_start_day(&self) -> Self {
        let weekday = self.date.weekday().num_days_from_monday();
        let date = self.date.sub(Duration::days(weekday as i64));
        Self::new(Period::Week, date)
    }

    pub fn week_end_day(&self) -> Self {
        let weekday = 6 - self.date.weekday().num_days_from_monday();
        let date = self.date.add(Duration::days(weekday as i64));
        Self::new(Period::Week, date)
    }
}

impl TradingDay {
    pub fn month_start_day(&self) -> Self {
        let date = self.date.with_day(1).unwrap();
        Self::new(Period::Day, date)
    }

    pub fn month_end_day(&self) -> Self {
        let month = self.date.month();
        let date = self.date.with_day(1)
            .map(|v|v.with_month(month + 1))
            .flatten()
            .map(|v|v.sub(Duration::days(1)) )
            .unwrap();
        Self::new(Period::Day, date)
    }
}

impl TradingDay {
    pub fn with_period(mut self, period: Period) -> Self {
        match period {
            Period::Week => {
                self.date = self.week_start_day().date;
            }
            _ => {}
        }
        self.period = period;
        self
    }
}

impl Add<usize> for TradingDay {
    type Output = Self;
    fn add(mut self, step: usize) -> Self::Output {
        match self.period {
            Period::Day => {
                for _ in 0..step {
                    self.date = self.date.add(Duration::days(1));
                    self.date = holidays::to_trading_day(self.date, Ordering::Greater)
                }
            }

            Period::Week => {
                self.date = self.week_start_day().date;
                for _ in 0..step {
                    self.date = self.date.add(Duration::days(7));
                }
            }

            Period::Minute(minutes) => {
                for _ in 0..step {
                    let close = self.close_time();
                    self.date = self.date.add(Duration::minutes(minutes as i64));
                    if self.date.gt(&close.date) {
                        self.date = self.next().open_time().date.add(Duration::minutes(minutes as i64));
                    }
                }
            }
        }
        self
    }
}

impl Sub<usize> for TradingDay {
    type Output = Self;
    fn sub(mut self, step: usize) -> Self::Output {
        match self.period {
            Period::Day => {
                for _ in 0..step {
                    self.date = self.date.sub(Duration::days(1));
                    self.date = holidays::to_trading_day(self.date, Ordering::Less)
                }
            }
            Period::Week => {
                self.date = self.week_start_day().date;
                for _ in 0..step {
                    self.date = self.date.sub(Duration::days(7));
                }
            }
            Period::Minute(minutes) => {
                for _ in 0..step {
                    let open = self.open_time();
                    self.date = self.date.sub(Duration::minutes(minutes as i64));
                    if self.date.lt(&open.date) {
                        self.date = self.previous().close_time().date;
                    }
                }
            }
        }
        self
    }
}

impl Add<Duration> for TradingDay {
    type Output = Self;
    fn add(self, d: Duration) -> Self::Output {
        let step = match self.period {
            Period::Day => d.num_days() as usize,
            Period::Week => d.num_weeks() as usize,
            Period::Minute(minutes) => d.num_minutes() as usize / minutes,
        };
        if step > 0 {
            return self.add(step);
        }
        self
    }
}

impl Sub<Duration> for TradingDay {
    type Output = Self;
    fn sub(self, step: Duration) -> Self::Output {
        let step = match self.period {
            Period::Day => step.num_days() as usize,
            Period::Week => step.num_weeks() as usize,
            Period::Minute(minutes) => step.num_minutes() as usize / minutes,
        };
        if step > 0 {
            return self.sub(step);
        }
        self
    }
}

impl Display for TradingDay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.period {
            Period::Day => write!(f, "{}", self.date.format(holidays::FORMAT)),
            Period::Week => write!(f, "{}", self.date.format(holidays::FORMAT)),
            Period::Minute(_) => write!(f, "{}", self.date.format(holidays::FULL_FORMAT)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Duration;

    use crate::{Period, TradingDay};

    #[test]
    fn test_period() {
        let day = Period::from_str("day").unwrap();
        let week = Period::Week;
        let minute = Period::Minute(5);
        let period1 = Period::from_str("5m").unwrap();
        let period2 = Period::Minute(15);
        assert_ne!(day, week);
        assert_ne!(period1, period2);
        assert_eq!(minute, period1);
    }

    #[test]
    fn test_trading_day() {
        {
            let day = TradingDay::from_str("2023-07-06").unwrap();
            let open_time = day.open_time();
            let close_time = day.close_time();
            assert_eq!("2023-07-06", day.to_string());
            assert_eq!("2023-07-06 09:30:00", open_time.to_string());
            assert_eq!("2023-07-06 15:00:00", close_time.to_string());
        }

        {
            let day = TradingDay::from_str("2023-07-06 10:42").unwrap();
            let open_time = day.open_time();
            let close_time = day.close_time();
            assert_eq!("2023-07-06 09:30:00", open_time.to_string());
            assert_eq!("2023-07-06 15:00:00", close_time.to_string());
        }
    }

    #[test]
    fn test_week() {
        let day = TradingDay::from_str("2023-07-06").unwrap();
        let week_start = day.week_start_day();
        let week_end = day.week_end_day();
        assert_eq!("2023-07-03", week_start.to_string(), "week start");
        assert_eq!("2023-07-09", week_end.to_string(), "week end");

        let day = week_start;
        let week_start = day.week_start_day();
        let week_end = day.week_end_day();
        assert_eq!("2023-07-03", week_start.to_string(), "week start");
        assert_eq!("2023-07-09", week_end.to_string(), "week end");
    }

    #[test]
    fn test_ops_day() {
        let day = TradingDay::from_str("2023-07-06").unwrap();

        let d2 = day.clone() + 1;
        assert_eq!("2023-07-07", d2.to_string(), "add 1 day");

        let d1 = day.clone() - 1;
        assert_eq!("2023-07-05", d1.to_string(), "sub 1 day");

        let d3 = day + 2;
        assert_eq!("2023-07-10", d3.to_string(), "add 2 day");
    }

    #[test]
    fn test_ops_week() {
        let day = TradingDay::from_str("2023-07-06").unwrap().with_period(Period::Week);
        assert_eq!("2023-07-03", day.to_string(), "week start");

        let d2 = day.clone() + 1;
        assert_eq!("2023-07-10", d2.to_string(), "add 1 week");

        let d1 = day.clone() - 1;
        assert_eq!("2023-06-26", d1.to_string(), "sub 1 week");

        let d3 = day.clone() + 2;
        assert_eq!("2023-07-17", d3.to_string(), "add 2 week");

        let d4 = day - 2;
        assert_eq!("2023-06-19", d4.to_string(), "sub 2 week");
    }

    #[test]
    fn test_ops_minutes() {
        let day = TradingDay::from_str("2023-07-06 14:30").unwrap().with_period(Period::Minute(5));

        let d2 = day.clone() + 1;
        assert_eq!("2023-07-06 14:35:00", d2.to_string(), "add 5 minute");

        let d1 = day.clone() - 1;
        assert_eq!("2023-07-06 14:25:00", d1.to_string(), "sub 5 minute");
    }

    #[test]
    fn test_ops_duration() {
        {
            let day = TradingDay::from_str("2023-07-06").unwrap();

            let d2 = day.clone() + Duration::seconds(10);
            assert_eq!("2023-07-06", d2.to_string(), "add 10s");

            let d1 = day.clone() - Duration::seconds(86400);
            assert_eq!("2023-07-05", d1.to_string(), "sub 1 day");

            let d3 = day + Duration::seconds(86400 * 2);
            assert_eq!("2023-07-10", d3.to_string(), "add 2 day");
        }
        {
            let day = TradingDay::from_str("2023-07-06 13:10").unwrap();
            let d2 = day.clone() + Duration::minutes(10);
            assert_eq!("2023-07-06 13:20:00", d2.to_string(), "add 10s");

            let day = TradingDay::from_str("2023-07-06 13:10").unwrap();
            let d2 = day.clone() + Duration::minutes(7);
            assert_eq!("2023-07-06 13:15:00", d2.to_string(), "add 7m");
        }
    }

    #[test]
    fn test_month(){
        let day = TradingDay::from_str("2023-05-15").unwrap();
        println!("month start day: {}", day.month_start_day());
        println!("month end day: {}", day.month_end_day());
    }
}
