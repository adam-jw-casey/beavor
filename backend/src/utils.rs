use chrono::{
    Local,
    NaiveDate,
};

use crate::due_date::ParseDateError;

#[must_use] pub fn format_date(date: NaiveDate) -> String{
    format_date_borrowed(&date)
}

#[must_use] pub fn format_date_borrowed(date: &NaiveDate) -> String{
    date.format("%F").to_string()
}

pub fn parse_date(date_string: &str) -> Result<NaiveDate, ParseDateError>{
    NaiveDate::parse_from_str(date_string, "%F").or(Err(ParseDateError))
}

#[must_use] pub fn today_string() -> String{
    format_date(today_date())
}

#[must_use] pub fn today_date() -> NaiveDate{
    Local::now().naive_local().date()
}

#[allow(clippy::zero_prefixed_literal)]
#[cfg(test)]
mod tests{
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn test_parse_format_date_today(){
        let d = NaiveDate::from_ymd_opt(1971,01,10).unwrap();
        assert_eq!(parse_date(&format_date(d)).unwrap(), d);

        assert_eq!(today_date(), parse_date(&today_string()).unwrap());
    }
}
