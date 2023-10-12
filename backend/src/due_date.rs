use chrono::NaiveDate;

use std::str::FromStr;
use std::convert::{From, Into};
use core::fmt::Display;

use std::cmp::Ordering;

use crate::utils::{
    format_date_borrowed,
    parse_date,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum DueDate{
    NONE,
    Date(NaiveDate),
    ASAP,
}

impl PartialOrd for DueDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DueDate {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            DueDate::NONE => match other{
                DueDate::NONE    => Ordering::Equal,
                DueDate::Date(_) => Ordering::Greater,
                DueDate::ASAP    => Ordering::Greater,
            },
            DueDate::ASAP => match other{
                DueDate::NONE    => other.cmp(self).reverse(),
                DueDate::Date(_) => Ordering::Less,
                DueDate::ASAP    => Ordering::Equal,
            },
            DueDate::Date(self_date) => match other{
                DueDate::NONE             => other.cmp(self).reverse(),
                DueDate::Date(other_date) => self_date.cmp(other_date),
                DueDate::ASAP             => other.cmp(self).reverse(),
            },
        }
    }
}

#[derive(Debug)]
pub struct ParseDateError;

impl FromStr for DueDate{
    type Err = ParseDateError;

    fn from_str(date_string: &str) -> Result<Self, Self::Err> {
        Ok(match date_string{
            "None" => DueDate::NONE,
            "ASAP" => DueDate::ASAP,
            date_string => DueDate::Date(parse_date(date_string)?),
        })
    }
}

impl TryFrom<String> for DueDate{
    type Error = ParseDateError;

    fn try_from(date_string: String) -> Result<Self, Self::Error> {
        DueDate::from_str(&date_string)
    }
}

impl From<&DueDate> for String{
    fn from(value: &DueDate) -> Self {
        match value{
            DueDate::NONE => "None".into(),
            DueDate::ASAP => "ASAP".into(),
            DueDate::Date(date) => format_date_borrowed(date),
        }
    }
}

impl Display for DueDate{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[cfg(test)]
#[allow(deprecated)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(non_snake_case)]
mod tests{
    use super::*;

    #[test]
    fn test_cmp_due_date(){
        assert!(DueDate::NONE == DueDate::NONE);
        assert!(DueDate::NONE > DueDate::ASAP);
        assert!(DueDate::NONE > DueDate::Date(NaiveDate::from_ymd(1971,01,01)));

        assert!(DueDate::ASAP < DueDate::NONE);
        assert!(DueDate::ASAP == DueDate::ASAP);
        assert!(DueDate::ASAP < DueDate::Date(NaiveDate::from_ymd(1971,01,01)));

        assert!(DueDate::Date(NaiveDate::from_ymd(1971,01,01)) < DueDate::NONE);
        assert!(DueDate::Date(NaiveDate::from_ymd(1971,01,01)) > DueDate::ASAP);
        assert!(DueDate::Date(NaiveDate::from_ymd(1971,01,01)) == DueDate::Date(NaiveDate::from_ymd(1971,01,01)));
        assert!(DueDate::Date(NaiveDate::from_ymd(1971,01,01)) < DueDate::Date(NaiveDate::from_ymd(1971,01,02)));
        assert!(DueDate::Date(NaiveDate::from_ymd(1971,01,01)) > DueDate::Date(NaiveDate::from_ymd(1970,12,31)));
    }

    #[test]
    fn test_due_date_string_parse(){
        for dd in [DueDate::ASAP, DueDate::NONE, DueDate::Date(NaiveDate::from_ymd(1971,01,01))]{
            assert_eq!(DueDate::from_str(&dd.to_string()).unwrap(), dd);
        }
    }
}
