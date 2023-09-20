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
    NaiveDate::parse_from_str(date_string, "%F").or(Err(ParseDateError))
}

#[pyfunction]
pub fn today_string() -> String{
    format_date(today_date())
}

#[pyfunction]
pub fn today_date() -> NaiveDate{
    Local::now().naive_local().date()
}

#[allow(deprecated)]
#[allow(clippy::zero_prefixed_literal)]
mod tests{
    use crate::{
        green_red_scale,
        format_date,
        parse_date,
        today_date,
        today_string
    };
    use chrono::NaiveDate;

    #[test]
    fn test_green_red_scale(){
        assert_eq!(green_red_scale(0.0,100.0,100.0), "#FF0000");
        assert_eq!(green_red_scale(0.0,100.0,0.0), "#00FF00");
    }

    #[test]
    fn test_parse_format_date_today(){
        let d = NaiveDate::from_ymd(1971,01,10);
        assert_eq!(parse_date(&format_date(d)).unwrap(), d);

        assert_eq!(today_date(), parse_date(&today_string()).unwrap());
    }
}
