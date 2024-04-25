//! Utility functions.

use crate::Date;
use core::ops::{Add, Deref, Sub};

/// Number of **Julian Month** since mid-day 1/1/1900 (julian day: 2415021).
#[derive(Clone, Copy, Debug)]
struct JulianMonthIndex(pub i32);

const JULIAN_MOON_CYCLE: f64 = 29.530588853;

impl JulianMonthIndex {
    pub fn from_julian_day(value: f64) -> Self {
        let offset = value - JULIAN_DAY_NOON_JAN_1_1900;
        let k_value = (offset / JULIAN_MOON_CYCLE) as i32;
        Self(k_value)
    }
}

impl JulianMonthIndex {
    pub fn new(value: i32) -> JulianMonthIndex {
        JulianMonthIndex(value)
    }
}

impl Into<f64> for JulianMonthIndex {
    fn into(self) -> f64 {
        let value = self.0;
        value.into()
    }
}

impl Add for JulianMonthIndex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let k = *self + *rhs;
        Self(k)
    }
}
impl Sub for JulianMonthIndex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let k = *self - *rhs;
        Self(k)
    }
}

impl From<f64> for JulianMonthIndex {
    fn from(value: f64) -> Self {
        let value = if value <= i32::MAX.into() {
            value as i32
        } else {
            panic!("K value is greater than i32::MAX")
        };

        Self(value)
    }
}

impl From<i32> for JulianMonthIndex {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Deref for JulianMonthIndex {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Astronomical constants
const JULIAN_CENTURY: f64 = 36525.0;
const EPOCH_2000_12: f64 = 2451545.0;

/// Mean anomaly constants
const MEAN_ANOMALY_BASE: f64 = 357.52910;
const MEAN_ANOMALY_COEF: f64 = 35999.05030;
const MEAN_ANOMALY_QUAD: f64 = 0.0001559;
const MEAN_ANOMALY_CUBE: f64 = 0.00000048;

/// Mean longitude constants
const MEAN_LONGITUDE_BASE: f64 = 280.46645;
const MEAN_LONGITUDE_COEF: f64 = 36000.76983;
const MEAN_LONGITUDE_QUAD: f64 = 0.0003032;

/// Equation of the center constants
const EQUATION_CENTER_BASE: f64 = 1.914600;
const EQUATION_CENTER_T_COEF: f64 = 0.004817;
const EQUATION_CENTER_T2_COEF: f64 = 0.000014;
const EQUATION_CENTER_FIRST_HARMONIC: f64 = 0.019993;
const EQUATION_CENTER_FIRST_HARMONIC_DECAY: f64 = 0.000101;
const EQUATION_CENTER_SECOND_HARMONIC: f64 = 0.000290;

const JULIAN_DAY_NOON_JAN_1_1900: f64 = 2415021.076998695;

/// Normalize an angle in degrees to the range (0, 360)
fn normalize_longitude(longitude: f64) -> f64 {
    longitude % 360.0
}

/// Calculate sun's position in the sky
///
/// Using AA98 equation
///
/// Parameters:
/// - `jdn`: Julian day at **12:00:00**
///
/// Return: degrees value from 0.0 to 360.0
fn sun_longitude_aa98(jdn: f64) -> f64 {
    // Time in Julian centuries from the epoch 2000-01-01 12:00:00
    let t = (jdn - EPOCH_2000_12) / JULIAN_CENTURY;
    let t_2 = t * t;
    let mean_anomaly = MEAN_ANOMALY_BASE + (MEAN_ANOMALY_COEF * t)
        - (MEAN_ANOMALY_QUAD * t_2)
        - (MEAN_ANOMALY_CUBE * t * t_2);
    let mean_longitude =
        MEAN_LONGITUDE_BASE + (MEAN_LONGITUDE_COEF * t) + (MEAN_LONGITUDE_QUAD * t_2);
    let equation_of_the_center =
        (EQUATION_CENTER_BASE - (EQUATION_CENTER_T_COEF * t) - (EQUATION_CENTER_T2_COEF * t_2))
            * f64::sin(mean_anomaly.to_radians())
            + (EQUATION_CENTER_FIRST_HARMONIC - EQUATION_CENTER_FIRST_HARMONIC_DECAY * t)
                * f64::sin(2.0 * mean_anomaly.to_radians())
            + EQUATION_CENTER_SECOND_HARMONIC * f64::sin(3.0 * mean_anomaly.to_radians());
    let true_longitude = mean_longitude + equation_of_the_center;

    // Normalize to (0, 360)
    normalize_longitude(true_longitude)
}

/// Get the sun's position in the sky aligned with local mid-day.
///
/// Adjusts the Julian day number to account for the timezone difference from UTC,
/// aligning the calculation to mid-day as expected by the AA98 astronomical equation.
///
/// Parameters:
/// - `jdn`: Julian day number at midnight UTC of the target date.
/// - `timezone`: Local timezone offset from UTC in hours (e.g., -5 for EST).
///
/// Return: Sun's longitude in degrees from 0.0 to 360.0.
fn get_sun_longitude(jdn: f64, timezone: f64) -> f64 {
    // Align with timezone
    let jdn_adjusted = jdn - 0.5 - timezone / 24.0;
    sun_longitude_aa98(jdn_adjusted)
}

/// Calculate the first day of month in LichTa Calendar in Julian day
///
/// Parameters:
/// - `k`: number of **Julian Month** since mid-day 1/1/1900 (julian day: 2415021).
///
/// Return: Julian day
fn new_moon_aa98(julian_month_index: JulianMonthIndex) -> f64 {
    let julian_month_index: f64 = julian_month_index.into();
    // Time in Julian centuries from 1900 January 0.5
    let t = julian_month_index / 1236.85;
    let t_2 = t * t;
    let t_3 = t_2 * t;
    let mean_new_moon = 2415020.75933 + 29.53058868 * julian_month_index + 0.0001178 * t_2
        - 0.000000155 * t_3
        + 0.00033 * f64::sin((166.56 + 132.87 * t - 0.009173 * t_2).to_radians()); // Mean new moon
    let sun_mean_anomaly =
        359.2242 + 29.10535608 * julian_month_index - 0.0000333 * t_2 - 0.00000347 * t_3; // Sun's mean anomaly
    let moon_mean_anomaly =
        306.0253 + 385.81691806 * julian_month_index + 0.0107306 * t_2 + 0.00001236 * t_3; // Moon's mean anomaly
    let moon_argument_latitude =
        21.2964 + 390.67050646 * julian_month_index - 0.0016528 * t_2 - 0.00000239 * t_3; // Moon's argument of latitude
    let mut lunar_correction = (0.1734 - 0.000393 * t) * f64::sin(sun_mean_anomaly.to_radians())
        + 0.0021 * f64::sin(2.0 * sun_mean_anomaly.to_radians());
    lunar_correction -= 0.4068 * f64::sin(moon_mean_anomaly.to_radians())
        + 0.0161 * f64::sin(2.0 * moon_mean_anomaly.to_radians());
    lunar_correction -= 0.0004 * f64::sin(3.0 * moon_mean_anomaly.to_radians());
    lunar_correction += 0.0104 * f64::sin(2.0 * moon_argument_latitude.to_radians())
        - 0.0051 * f64::sin((sun_mean_anomaly + moon_mean_anomaly).to_radians());
    lunar_correction -= 0.0074 * f64::sin((sun_mean_anomaly - moon_mean_anomaly).to_radians())
        + 0.0004 * f64::sin((2.0 * moon_argument_latitude + sun_mean_anomaly).to_radians());
    lunar_correction -= 0.0004
        * f64::sin((2.0 * moon_argument_latitude - sun_mean_anomaly).to_radians())
        - 0.0006 * f64::sin((2.0 * moon_argument_latitude + moon_mean_anomaly).to_radians());
    lunar_correction += 0.0010
        * f64::sin((2.0 * moon_argument_latitude - moon_mean_anomaly).to_radians())
        + 0.0005 * f64::sin((2.0 * moon_mean_anomaly + sun_mean_anomaly).to_radians());
    let delta_t = if t < -11.0 {
        0.001 + 0.000839 * t + 0.0002261 * t_2 - 0.00000845 * t_3 - 0.000000081 * t * t_3
    } else {
        -0.000278 + 0.000265 * t + 0.000262 * t_2
    };
    mean_new_moon + lunar_correction - delta_t
}

/// Get the first day of month in LichTa Calendar in Julian day.
///
/// Adjusts the Julian day number to account for the timezone difference from UTC,
/// aligning the calculation to mid-day as expected by the AA98 astronomical equation.
///
/// Parameters:
/// - `k`: number of **Julian Month** since mid-day 1/1/1900 (julian day: 2415021).
/// - `timezone`: Local timezone offset from UTC in hours (e.g., -5 for EST).
///
/// Return: Julian day number
fn get_new_moon_day(julian_month_index: JulianMonthIndex, timezone: f64) -> f64 {
    let jd = new_moon_aa98(julian_month_index);
    (jd + 0.5 + timezone / 24.0).floor()
}

const SOLAR_LONGITUDE_THRESHOLD: f64 = 9.0;

/// Get the Julian day for the beginning of month 11 in the LichTa calendar for a given year.
///
/// The month typically corresponds to the time around the winter solstice, which occurs
/// around December 21 or 22. This function starts by looking at the last day of December
/// and works backward to find the new moon day corresponding to the 11th lunar month.
///
/// Parameters:
/// - `year`: The year for which to find the month.
/// - `timezone`: Local timezone offset from UTC in hours (e.g., -5 for EST).
///
/// Returns: Julian day number for the start of the 11th lunar month.
fn get_lunar_month_11(year: i32, timezone: f64) -> f64 {
    let date =
        Date::from_calendar_date(year, time::Month::December, 31).expect("Invalid date for year");
    let julian_day: f64 = date.to_julian_day().into();
    let k = JulianMonthIndex::from_julian_day(julian_day);
    // Calculate the new moon day for the current k value.
    let new_moon_day = get_new_moon_day(k, timezone);

    // Determine the solar longitude and adjust for the beginning of lunar month 11.
    let sun_longitute = (get_sun_longitude(new_moon_day, timezone) / 30.0).trunc();
    if sun_longitute >= SOLAR_LONGITUDE_THRESHOLD {
        // If the solar longitude indicates a new lunar month has started, adjust k.
        get_new_moon_day(k - JulianMonthIndex::new(1), timezone)
    } else {
        new_moon_day
    }
}

const SOLAR_LONGITUDE_SEGMENT: f64 = 30.0; // Each segment of solar longitude for a lunar month

/// Get the leap month offset for a lunar calendar year potentially having 13 months.
///
//// The leap month is determined based on consecutive lunar months having the same solar longitude,
/// indicating a leap month insertion. The check starts from the given month 11.
///
/// Parameters:
/// - `first_month_11`: Begin day of month 11 which one of 13 month is leap month.
/// - `timezone`: Local timezone offset from UTC in hours (e.g., -5 for EST).
///
/// Returns: Index of the leap month after month 11, or 14 if no leap month is found.
fn get_leap_month_offset(first_month_11: i32, timezone: f64) -> i32 {
    let a11: f64 = first_month_11.try_into().unwrap();
    let julian_month_index = JulianMonthIndex::from_julian_day(a11);
    let mut last_solar_longitude = 0.0;
    for i in 1..14 {
        let day_number = get_new_moon_day(julian_month_index + JulianMonthIndex::new(i), timezone);
        let solar_longitude =
            (get_sun_longitude(day_number, timezone) / SOLAR_LONGITUDE_SEGMENT).floor();
        if solar_longitude == last_solar_longitude {
            return i - 1;
        }
        last_solar_longitude = solar_longitude;
    }
    14
}

fn calculate_month_between_julian_days(julian_day_1: f64, julian_day_2: f64) -> i32 {
    ((julian_day_1 - julian_day_2) / 29.0) as i32
}

/// Convert Gregorian day to Lichta day
///
/// The leap month is determined based on consecutive lunar months having the same solar longitude,
/// indicating a leap month insertion. The check starts from the given month 11.
///
/// Parameters:
/// - `first_month_11`: Begin day of month 11 which one of 13 months is leap month.
/// - `timezone`: Local timezone offset from UTC in hours (e.g., -5 for EST).
///
/// Returns: (day: i32, month: i32, year: i32, leap: bool)
pub fn convert_date_to_lichta(date: Date, timezone: f64) -> (i32, i32, i32, i32) {
    let julian_day: f64 = date.to_julian_day().into();
    let julian_month_index = JulianMonthIndex::from_julian_day(julian_day);

    let mut month_start = get_new_moon_day(julian_month_index + JulianMonthIndex::new(1), timezone);
    if month_start > julian_day {
        month_start = get_new_moon_day(julian_month_index, 7.0);
    }

    let mut first_month_11 = get_lunar_month_11(date.year(), timezone);
    let mut last_month_11 = first_month_11;
    if first_month_11 >= month_start {
        first_month_11 = get_lunar_month_11(date.year() - 1, timezone);
    } else {
        last_month_11 = get_lunar_month_11(date.year() + 1, timezone);
    }
    let lunar_day = (julian_day - month_start + 1.0) as i32;

    let month_difference = calculate_month_between_julian_days(month_start, first_month_11);

    let mut lunar_leap = 0;

    let mut lunar_month = month_difference + 11;
    if last_month_11 - first_month_11 > 365.0 {
        let leap_month_index = get_leap_month_offset(first_month_11 as i32, timezone);
        if month_difference >= leap_month_index {
            lunar_month = month_difference + 10;
            if month_difference == leap_month_index {
                // Indicate leap month
                lunar_leap = 1;
            }
        }
    }

    // Normalize the lunar month to ensure it falls within the typical 1-12 range
    if lunar_month > 12 {
        lunar_month -= 12;
    }

    // Adjust the lunar year based on the lunar month
    let mut lunar_year = date.year();
    if lunar_month >= 11 && month_difference < 4 {
        lunar_year -= 1;
    }

    (lunar_day, lunar_month, lunar_year, lunar_leap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Date;

    #[test]
    fn test_get_sun_longitude() {
        let jdn = 2451520_f64;
        assert_eq!(get_sun_longitude(jdn, 7.0), 254.13250183229925);
    }

    #[test]
    fn test_new_moon_aa98() {
        let k = JulianMonthIndex::new(1533);
        assert_eq!(new_moon_aa98(k), 2_460_291.49468915);
    }

    #[test]
    fn test_get_lunar_month_11() {
        assert_eq!(get_lunar_month_11(2024, 7.0), 2_460_646_f64);
    }
    #[test]
    fn test_get_leap_month_offset() {
        let a11 = get_lunar_month_11(2022, 7.0) as i32;
        assert_eq!(get_leap_month_offset(a11, 7.0), 3);
    }

    #[test]
    fn test_convert_to_lich_ta() {
        let date = Date::from_calendar_date(2024, time::Month::May, 24).unwrap();
        let lichta = convert_date_to_lichta(date, 7.0);
        assert_eq!(lichta, (17, 4, 2024, 0));

        let date = Date::from_calendar_date(2022, time::Month::May, 24).unwrap();
        let lichta = convert_date_to_lichta(date, 7.0);
        assert_eq!(lichta, (24, 4, 2022, 0));
    }
}
