use chrono::{
    Datelike,
    Local,
    NaiveDate,
    Weekday,
};

use crate::ParseDateError;

use pyo3::prelude::pyfunction;

#[pyfunction]
pub fn green_red_scale(low: f32, high: f32, val: f32) -> String {
    let frac = f32::max(0.0,f32::min(1.0,(val-low)/(high-low)));

    let red: u8;
    let green: u8;

    if frac > 0.5{
        red = 255;
        green = ((2.0-2.0*frac) * 255.0) as u8;
    }else{
        red = ((2.0*frac) * 255.0) as u8;
        green = 255
    }

    format!("#{red:02X}{green:02X}00")
}

#[pyfunction]
pub fn format_date(date: NaiveDate) -> String{
    format_date_borrowed(&date)
}

pub fn format_date_borrowed(date: &NaiveDate) -> String{
    date.format("%F").to_string()
}

#[pyfunction]
pub fn parse_date(date_string: &str) -> Result<NaiveDate, ParseDateError>{
    match NaiveDate::parse_from_str(date_string, "%F"){
        Ok(nd) => Ok(nd),
        _ => Err(ParseDateError)
    }
}

#[pyfunction]
pub fn today_str() -> String{
    format_date(today_date())
}

#[pyfunction]
pub fn today_date() -> NaiveDate{
    Local::now().naive_local().date()
}

pub fn work_days_from(d1: NaiveDate, d2: NaiveDate) -> i32{
    let weeks_between = (d2-d1).num_weeks() as i32;

    let marginal_workdays: u32 = match d2.weekday(){
        Weekday::Sat | Weekday::Sun => match d1.weekday(){
            Weekday::Sat | Weekday::Sun => 0,
            weekday1 => Weekday::Fri.number_from_monday() - weekday1.number_from_monday() + 1,
        },
        weekday2 => match d1.weekday(){
            Weekday::Sat | Weekday::Sun => weekday2.number_from_monday() - Weekday::Mon.number_from_monday(),
            weekday1 => (weekday2.number_from_monday() as i32 - weekday1.number_from_monday() as i32).rem_euclid(5) as u32 + 1,
        },
    };

    weeks_between * 5 + marginal_workdays as i32
}

#[cfg(test)]
#[allow(deprecated)]
#[allow(clippy::zero_prefixed_literal)]
mod tests{
    use super::*;

    #[test]
    fn test_work_days_from() {
        assert_eq!(
            work_days_from(
                NaiveDate::from_ymd(2023, 08, 21),
                NaiveDate::from_ymd(2023, 08, 25)
            ),
            5 // This is a simple workweek
        );

        assert_eq!(
            work_days_from(
                NaiveDate::from_ymd(2023, 08, 11),
                NaiveDate::from_ymd(2023, 08, 14)
            ),
            2 // Friday to Monday
        );

        assert_eq!(
            work_days_from(
                NaiveDate::from_ymd(2023, 08, 1),
                NaiveDate::from_ymd(2023, 08, 23)
            ),
        17 // Multiple weeks, starting day of week is earlier
        );

        assert_eq!(
            work_days_from(
                NaiveDate::from_ymd(2023, 08, 4),
                NaiveDate::from_ymd(2023, 08, 23)
            ),
            14 // Multiple weeks, starting day of week is later
        );

        assert_eq!(
            work_days_from(
                NaiveDate::from_ymd(2023, 08, 19),
                NaiveDate::from_ymd(2023, 08, 27)
            ),
            5 // Start and end on a weekend
        );
    }
}
