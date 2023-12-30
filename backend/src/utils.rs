use chrono::{
    Local,
    NaiveDate,
    NaiveTime,
};

use anyhow::Result;

/// Pure
#[must_use] pub fn format_date(date: NaiveDate) -> String {
    format_date_borrowed(&date)
}

/// Pure
#[must_use] pub fn format_date_borrowed(date: &NaiveDate) -> String {
    date.format("%F").to_string()
}

/// Pure
///
/// # Errors
/// Returns an error if the string cannot be parsed as a date
pub fn parse_date(date_string: &str) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(date_string, "%F")?)
}

/// Impure (reads system clock)
#[must_use] pub fn today_string() -> String {
    format_date(today_date())
}

/// Impure (reads system clock)
#[must_use] pub fn today_date() -> NaiveDate {
    Local::now().naive_local().date()
}

/// Pure
#[must_use] pub fn format_time(time: NaiveTime) -> String {
    time.format("%H:%M:%S").to_string()
}

/// Pure
/// # Errors
/// Returns an error if the string cannot be parsed as an `%H:%M:%S` time
pub fn parse_time(time_string: &str) -> Result<NaiveTime> {
    Ok(NaiveTime::parse_from_str(time_string, "%H:%M:%S")?)
}

/// Impure (reads system clock)
#[must_use] pub fn now_time() -> NaiveTime {
    Local::now().naive_local().time()
}

/// Impure (reads system clock)
#[must_use] pub fn now_string() -> String {
    format_time(now_time())
}

#[allow(clippy::zero_prefixed_literal)]
#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn test_parse_format_date() {
        let d = NaiveDate::from_ymd_opt(1971,01,10).unwrap();
        assert_eq!(parse_date(&format_date(d)).unwrap(), d);
    }

    #[test]
    fn test_parse_format_today() {
        assert_eq!(today_date(), parse_date(&today_string()).unwrap());
    }

    #[test]
    fn test_parse_format_time() {
        let t = NaiveTime::from_hms_opt(7,01,10).unwrap();
        assert_eq!(parse_time(&format_time(t)).unwrap(), t);
    }
}
