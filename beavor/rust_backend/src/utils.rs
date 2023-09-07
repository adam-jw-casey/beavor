use chrono::{
    Local,
    NaiveDate,
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
