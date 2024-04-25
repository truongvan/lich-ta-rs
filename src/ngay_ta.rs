//! The [`LichTa`] struct and its associated `impl`s.

use crate::util::convert_date_to_lichta;
use crate::Date;

/// NgayTa in the LichTa calendar.
#[derive(Clone, Copy, Debug)]
pub struct NgayTa {
    day: i32,
    month: i32,
    year: i32,
    is_leap_month: bool,
}

impl NgayTa {
    pub fn day(&self) -> i32 {
        self.day
    }
    pub fn month(&self) -> i32 {
        self.month
    }
    pub fn year(&self) -> i32 {
        self.year
    }
    pub fn is_leap_month(&self) -> bool {
        self.is_leap_month
    }
}

impl NgayTa {
    pub fn new(day: i32, month: i32, year: i32, is_leap_month: bool) -> Self {
        Self {
            day,
            month,
            year,
            is_leap_month,
        }
    }
    pub fn from_date(date: Date, timezone: f64) -> Self {
        let (day, month, year, is_leap_month) = convert_date_to_lichta(date, timezone);
        Self::new(day, month, year, is_leap_month == 1)
    }
}
