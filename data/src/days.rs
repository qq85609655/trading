use anyhow::Context;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;

use chrono::{DateTime, Duration, DurationRound, Local, TimeZone, Timelike};

use crate::deref;

pub mod holidays {
    use std::cmp::Ordering;
    use std::ops::Add;

    use anyhow::{Context, Result};
    use chrono::{DateTime, Datelike, Duration, Local};
    use lazy_static::lazy_static;

    pub const FORMAT: &str = "%Y-%m-%d";

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
        HOLIDAYS.iter().any(|holiday| holiday.is_holiday(date))
    }

    pub fn not_holiday(date: &str) -> bool {
        !is_holiday(date)
    }

    pub fn is_weekend(date: &str) -> Result<bool> {
        let weekday = chrono::NaiveDate::parse_from_str(date, FORMAT)
            .context("Invalid date")?
            .weekday();
        Ok(weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun)
    }

    /// 返回指定日期是否是一个节假日
    pub fn is_trading_day(date: &str) -> Result<bool> {
        Ok(!is_weekend(date)? && not_holiday(date))
    }

    pub fn today_is_trading_day() -> Result<bool> {
        let today = chrono::Local::now().format(FORMAT).to_string();
        is_trading_day(&today)
    }

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
}

deref! {
    #[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
    pub struct TradingDay(DateTime<Local>);
}

impl FromStr for TradingDay {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let date = Local
            .datetime_from_str(format!("{} 00:00:00", s).as_str(), "%Y-%m-%d %H:%M:%S")?;
        let date = holidays::to_trading_day(date, Ordering::Less);
        Ok(TradingDay::new(date))
    }
}

impl TradingDay {
    pub fn trading(day: Option<String>) -> anyhow::Result<Self> {
        match day {
            Some(day) => FromStr::from_str(&day).context("Invalid trading day"),
            None => Ok(TradingDay::latest()),
        }
    }

    pub fn latest() -> Self {
        let now = Local::now();
        let today = now.duration_trunc(Duration::days(1)).unwrap();
        let mut latest = Self::new(holidays::to_trading_day(today, Ordering::Less));
        let open = latest.open_time();
        if now.lt(&open) {
            latest = latest.previous();
        }
        latest
    }

    pub fn previous(&self) -> Self {
        Self(holidays::to_trading_day(self.0.sub(Duration::days(1)), Ordering::Less))
    }

    pub fn next(&self) -> Self {
        Self(holidays::to_trading_day(self.0.add(Duration::days(1)), Ordering::Greater))
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

    #[allow(unused)]
    pub fn open_time(&self) -> Self {
        let open = self.0.with_hour(9).unwrap();
        Self(open.with_minute(30).unwrap())
    }

    pub fn close_time(&self) -> Self {
        Self(self.0.with_hour(15).unwrap())
    }

    pub fn is_now_closed(&self) -> bool {
        let now = Local::now();
        self.close_time().value().lt(&now)
    }

    #[inline]
    pub fn sub(&self, day: usize) -> Self {
        self.clone() - day
    }

    #[inline]
    pub fn add(&self, day: usize) -> Self {
        self.clone() + day
    }

    pub fn label(&self, end: &Self) -> Vec<TradingDay> {
        let mut items = vec![self.clone()];
        let mut start = self.clone();
        loop {
            if start.0.cmp(&end.0) != Ordering::Less {
                break;
            }
            start = start.next();
            items.push(start.clone());
        }
        items.push(end.clone());
        items
    }
}

impl Add<usize> for TradingDay {
    type Output = Self;
    fn add(self, day: usize) -> Self::Output {
        let mut out = self;
        for _ in 0..day {
            out = out.next();
        }
        out
    }
}

impl Sub<usize> for TradingDay {
    type Output = Self;
    fn sub(self, day: usize) -> Self::Output {
        let mut out = self;
        for _ in 0..day {
            out = out.previous();
        }
        out
    }
}

impl Display for TradingDay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format(holidays::FORMAT))
    }
}
