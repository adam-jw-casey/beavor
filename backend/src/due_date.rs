use chrono::NaiveDate;

use std::str::FromStr;
use std::convert::{From, Into};
use core::fmt::Display;

use std::cmp::Ordering;

use crate::utils::{
    format_date_borrowed,
    parse_date,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
pub enum DueDate{
    Never,
    Date(NaiveDate),
    Asap,
}

impl DueDate{
    #[must_use] pub fn new() -> Self{
        Self::default()
    }
}

impl Default for DueDate{
    fn default() -> Self {
        Self::Asap
    }
}

impl PartialOrd for DueDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DueDate {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            DueDate::Never => match other{
                DueDate::Never  => Ordering::Equal,
                DueDate::Date(_) | DueDate::Asap => Ordering::Greater,
            },
            DueDate::Asap => match other{
                DueDate::Never    => other.cmp(self).reverse(),
                DueDate::Date(_) => Ordering::Less,
                DueDate::Asap    => Ordering::Equal,
            },
            DueDate::Date(self_date) => match other{
                DueDate::Date(other_date) => self_date.cmp(other_date),
                DueDate::Never | DueDate::Asap => other.cmp(self).reverse(),
            },
        }
    }
}

impl FromStr for DueDate{
    type Err = anyhow::Error;

    fn from_str(date_string: &str) -> Result<Self, Self::Err> {
        Ok(match date_string{
            "None" => DueDate::Never,
            "ASAP" => DueDate::Asap,
            date_string => DueDate::Date(parse_date(date_string)?),
        })
    }
}

impl TryFrom<String> for DueDate{
    type Error = anyhow::Error;

    fn try_from(date_string: String) -> Result<Self, Self::Error> {
        DueDate::from_str(&date_string)
    }
}

impl From<&DueDate> for String{
    fn from(value: &DueDate) -> Self {
        match value{
            DueDate::Never => "None".into(),
            DueDate::Asap => "ASAP".into(),
            DueDate::Date(date) => format_date_borrowed(date),
        }
    }
}

impl Display for DueDate{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<&NaiveDate> for DueDate{
    fn from(value: &NaiveDate) -> Self {
        Self::Date(*value)
    }
}

impl From<NaiveDate> for DueDate{
    fn from(value: NaiveDate) -> Self {
        Self::from(&value)
    }
}

#[cfg(test)]
#[allow(clippy::zero_prefixed_literal)]
mod tests{
    use super::*;

    #[test]
    fn test_cmp_due_date(){
        assert!(DueDate::Never == DueDate::Never);
        assert!(DueDate::Never > DueDate::Asap);
        assert!(DueDate::Never > DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()));

        assert!(DueDate::Asap < DueDate::Never);
        assert!(DueDate::Asap == DueDate::Asap);
        assert!(DueDate::Asap < DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()));

        assert!(DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()) < DueDate::Never);
        assert!(DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()) > DueDate::Asap);
        assert!(DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()) == DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()));
        assert!(DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()) < DueDate::Date(NaiveDate::from_ymd_opt(1971,01,02).unwrap()));
        assert!(DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap()) > DueDate::Date(NaiveDate::from_ymd_opt(1970,12,31).unwrap()));
    }

    #[test]
    fn test_due_date_string_parse(){
        for dd in [DueDate::Asap, DueDate::Never, DueDate::Date(NaiveDate::from_ymd_opt(1971,01,01).unwrap())]{
            assert_eq!(DueDate::from_str(&dd.to_string()).unwrap(), dd);
        }
    }
}
