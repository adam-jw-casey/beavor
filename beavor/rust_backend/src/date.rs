use pyo3::prelude::{
    pyfunction,
    pyclass,
    pymethods,
    PyResult,
};
use pyo3::types::PyType;
use pyo3::PyErr;
use pyo3::exceptions::{
    PyValueError,
    PyNotImplementedError,
};
use pyo3::basic::CompareOp;

use chrono::{
    Local,
    Datelike, // This isn't explicitly used, but gives access to certain trait methods on NaiveDate
    Weekday,
    NaiveDate,
};

use std::str::FromStr;
use std::convert::From;
use core::fmt::Display;

use std::cmp::Ordering;

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

#[pyclass]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, PartialEq)]
pub enum PyDueDateType{
    NONE,
    Date,
    ASAP,
}

#[pyclass]
#[derive(Clone, PartialEq)]
pub struct PyDueDate{
    #[pyo3(get, set)]
    date_type: PyDueDateType,
    #[pyo3(get, set)]
    date: Option<NaiveDate>,
}

#[pymethods]
impl PyDueDate{
    fn __str__(&self) -> String{
        (&DueDate::from(self)).into()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op{
            CompareOp::Eq => Ok(*self == *other),
            CompareOp::Ne => Ok(*self != *other),
            _ => Err(PyNotImplementedError::new_err(format!("{:#?} is not implemented for PyDueDate", op))),
        }
    }

    #[classmethod]
    fn parse(_cls: &PyType, s: String) -> PyResult<Self>{
        Ok(DueDate::try_from(s)?.into())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

impl From<DueDate> for PyDueDate{
    fn from(rust_due_date: DueDate) -> Self {
        match rust_due_date{
            DueDate::NONE => PyDueDate{date_type: PyDueDateType::NONE, date: None},
            DueDate::Date(date) => PyDueDate{date_type: PyDueDateType::Date, date: Some(date)},
            DueDate::ASAP => PyDueDate{date_type: PyDueDateType::ASAP, date: None},
        }
    }
}

impl From<&PyDueDate> for DueDate{
    fn from(pyvalue: &PyDueDate) -> Self {
        match pyvalue.date_type{
            PyDueDateType::NONE => DueDate::NONE,
            PyDueDateType::Date => DueDate::Date(pyvalue.date.expect("If PyDueDateType is Date then date will no be None")),
            PyDueDateType::ASAP => DueDate::ASAP,
        }
    }
}

#[derive(Debug)]
pub struct ParseDateError;

impl From<ParseDateError> for PyErr{
    fn from(_: ParseDateError) -> Self {
        PyValueError::new_err("Error parsing date")
    }
}

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
