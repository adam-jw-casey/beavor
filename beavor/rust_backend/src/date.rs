use pyo3::prelude::{
    pyfunction,
    pyclass,
    pymethods,
    PyResult,
};
use pyo3::PyErr;
use pyo3::exceptions::{
    PyValueError,
    PyNotImplementedError,
};
use pyo3::basic::CompareOp;
use pyo3::types::PyType;

use chrono::Local;
use chrono::naive::NaiveDate;

use std::convert::From;
use std::str::FromStr;
use core::fmt::Display;

#[pyfunction]
pub fn format_date(date: NaiveDate) -> String{
    format_date_borrowed(&date)
}

fn format_date_borrowed(date: &NaiveDate) -> String{
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

#[pyclass]
#[derive(Clone)]
pub enum PyAvailabilityType{
    Any,
    Date,
    DeliverableID
}

#[pyclass]
#[derive(Clone)]
pub struct PyAvailability{
    availability_type: PyAvailabilityType,
    date: Option<NaiveDate>,
    deliverable_id: Option<i64>,
}

#[derive(Clone)]
pub enum Availability{
    Any,
    Date(NaiveDate),
    DeliverableID(i64),
}

impl TryFrom<String> for Availability{
    type Error = ParseDateError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str(){
            "Any" => Availability::Any,
            other => {
                if let Ok(deliverable_id) = other.parse::<i64>(){
                    Availability::DeliverableID(deliverable_id)
                }else{
                    Availability::Date(parse_date(other)?)
                }
            }
        })
    }
}

impl From<&Availability> for String{
    fn from(value: &Availability) -> Self {
        match value{
            Availability::Any => "Any".into(),
            Availability::Date(date) => format_date(*date),
            Availability::DeliverableID(deliverable_id) => deliverable_id.to_string(),
        }
    }
}

impl From<&Availability> for PyAvailability{
    fn from(value: &Availability) -> Self {
        match value{
            Availability::Any => PyAvailability{
                availability_type: PyAvailabilityType::Any,
                date: None,
                deliverable_id: None,
            },
            Availability::Date(date) => PyAvailability{
                availability_type: PyAvailabilityType::Date,
                date: Some(*date),
                deliverable_id: None,
            },
            Availability::DeliverableID(id) => PyAvailability{
                availability_type: PyAvailabilityType::DeliverableID,
                date: None,
                deliverable_id: Some(*id),
            },
        }
    }
}

impl From<&PyAvailability> for Availability{
    fn from(value: &PyAvailability) -> Self {
        match value.availability_type{
            PyAvailabilityType::Any => Availability::Any,
            PyAvailabilityType::Date => Availability::Date(
                value.date.expect("If type is date, date should be Some")),
            PyAvailabilityType::DeliverableID => Availability::DeliverableID(
                value.deliverable_id.expect("If type is deliverable_id, it should be Some")),
        }
    }
}

#[pyclass]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, PartialEq)]
pub enum PyDueDateType{
    None,
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

#[derive(Clone, Copy, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum DueDate{
    None,
    Date(NaiveDate),
    ASAP,
}

impl From<DueDate> for PyDueDate{
    fn from(rust_due_date: DueDate) -> Self {
        match rust_due_date{
            DueDate::None => PyDueDate{date_type: PyDueDateType::None, date: None},
            DueDate::Date(date) => PyDueDate{date_type: PyDueDateType::Date, date: Some(date)},
            DueDate::ASAP => PyDueDate{date_type: PyDueDateType::ASAP, date: None},
        }
    }
}

impl From<&PyDueDate> for DueDate{
    fn from(pyvalue: &PyDueDate) -> Self {
        match pyvalue.date_type{
            PyDueDateType::None => DueDate::None,
            PyDueDateType::Date => DueDate::Date(pyvalue.date.expect("If PyDueDateType is Date then date will be Some")),
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
            "None" => DueDate::None,
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
            DueDate::None => "None".into(),
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
