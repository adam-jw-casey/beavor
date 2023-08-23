use pyo3::prelude::{
    pyfunction,
    pymodule,
    pyclass,
    pymethods,
    PyResult,
    PyModule,
    Python
};
use pyo3::types::PyType;
use pyo3::{
    wrap_pyfunction,
    PyErr,
};
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

use std::cmp::{
    max,
    Ordering
};

mod database;
use database::DatabaseManager;

#[pyfunction]
// Tested and this is ~3x faster than the exact same implementation in Python,
// even with the API calls
fn green_red_scale(low: f32, high: f32, val: f32) -> String {
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
fn format_date(date: NaiveDate) -> String{
    format_date_borrowed(&date)
}

fn format_date_borrowed(date: &NaiveDate) -> String{
    date.format("%F").to_string()
}

#[pyfunction]
fn parse_date(date_string: &str) -> Result<NaiveDate, ParseDateError>{
    match NaiveDate::parse_from_str(date_string, "%F"){
        Ok(nd) => Ok(nd),
        _ => Err(ParseDateError)
    }
}

#[pyfunction]
fn today_str() -> String{
    format_date(today_date())
}

#[pyfunction]
fn today_date() -> NaiveDate{
    Local::now().naive_local().date()
}

fn work_days_from(d1: NaiveDate, d2: NaiveDate) -> i32{
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
enum PyDueDateType{
    NONE,
    Date,
    ASAP,
}

#[pyclass]
#[derive(Clone, PartialEq)]
struct PyDueDate{
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
enum DueDate{
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
struct ParseDateError;

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

#[derive(Clone)]
#[pyclass]
struct Task{
    #[pyo3(get, set)]
    category:         String,
    #[pyo3(get, set)]
    finished:         String, // TODO It's inexcusable that this is a string and not an Enum
    #[pyo3(get, set)]
    task_name:        String,
    #[pyo3(get)]
    _time_budgeted:   i32,
    #[pyo3(get, set)]
    time_needed:      i32,
    #[pyo3(get, set)]
    time_used:        i32,
    #[pyo3(get, set)]
    notes:            String,
    #[pyo3(get, set)]
    date_added:       NaiveDate,
    #[pyo3(get, set)]
    next_action_date: NaiveDate,
    due_date:         DueDate,
    #[pyo3(get)]
    id:               Option<i32>,
}

#[pymethods]
impl Task{
    #[getter]
    fn get_due_date(&self) -> PyDueDate{
        self.due_date.into()
    }

    #[setter]
    fn set_due_date(&mut self, due_date: PyDueDate){
        self.due_date = (&due_date).into()
    }

    // Return the number of minutes per day you would have to work
    // on this task to complete it by its deadline
    fn workload_on_day(&self, date: NaiveDate) -> u32{
        match date.weekday(){
            Weekday::Sun | Weekday::Sat=> 0,
            _ => {
                match self.due_date{
                    DueDate::NONE => 0,
                    DueDate::ASAP => {
                        if (DueDate::Date(today_date()) > self.due_date && date == today_date()) || date == self.next_action_date{
                            (self.time_needed -  self.time_used).try_into().expect("Value should be nonnegative")
                        }else{
                            0
                        }
                    },
                    DueDate::Date(due_date) => {
                    if date >= self.next_action_date && DueDate::Date(date) <= self.due_date{
                            TryInto::<u32>::try_into(self.time_needed -  self.time_used).unwrap_or(0) // Remaining time
                            / // Divided by
                            TryInto::<u32>::try_into(work_days_from(max(today_date(), self.next_action_date), max(today_date(), due_date))).unwrap_or(1) // Days remaining
                        }else{
                            0
                        }
                    }
                }
            }
        }
    }
}

#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_date, m)?)?;
    m.add_function(wrap_pyfunction!(green_red_scale, m)?)?;
    m.add_function(wrap_pyfunction!(parse_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_str, m)?)?;
    m.add_class::<Task>().unwrap();
    m.add_class::<PyDueDate>().unwrap();
    m.add_class::<PyDueDateType>().unwrap();
    m.add_class::<DatabaseManager>().unwrap();
    Ok(())
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
    fn test_workload_on_day_with_ASAP(){
        let due_date_task = Task{
            category: "".to_string(),
            finished: "X".to_string(),
            task_name: "Test".to_string(),
            _time_budgeted: 600,
            time_needed: 600,
            time_used: 0,
            notes: "".to_string(),
            date_added: today_date(),
            next_action_date: NaiveDate::from_ymd(3000, 01, 01),
            due_date: DueDate::ASAP,
            id: None,
        };

        // A date before the due_date_task starts
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(2999,12,31)),
            0
        );

        // A date after the due_date_task ends
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,09)),
            0
        );

        // A weekend during the due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,04)),
            0
        );

        // First day of the due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,01)),
            600
        );

        // Last day of the due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,08)),
            0
        );

        // Weekday in middle of due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,06)),
            0
        );
    }

    #[test]
    fn test_workload_on_day_with_due_date(){
        let due_date_task = Task{
            category: "".to_string(),
            finished: "X".to_string(),
            task_name: "Test".to_string(),
            _time_budgeted: 600,
            time_needed: 600,
            time_used: 0,
            notes: "".to_string(),
            date_added: today_date(),
            next_action_date: NaiveDate::from_ymd(3000, 01, 01),
            due_date: DueDate::Date(NaiveDate::from_ymd(3000, 01, 08)),
            id: None,
        };

        // A date before the due_date_task starts
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(2999,12,31)),
            0
        );

        // A date after the due_date_task ends
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,09)),
            0
        );

        // A weekend during the due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,04)),
            0
        );

        // First day of the due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,01)),
            100
        );

        // Last day of the due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,08)),
            100
        );

        // Weekday in middle of due_date_task
        assert_eq!(
            due_date_task.workload_on_day(NaiveDate::from_ymd(3000,01,06)),
            100
        );
    }

    #[test]
    fn test_workload_on_day_with_NONE(){
        let NONE_task = Task{
            category: "".to_string(),
            finished: "X".to_string(),
            task_name: "Test".to_string(),
            _time_budgeted: 600,
            time_needed: 600,
            time_used: 0,
            notes: "".to_string(),
            date_added: today_date(),
            next_action_date: NaiveDate::from_ymd(3000, 01, 01),
            due_date: DueDate::NONE,
            id: None,
        };

        // A date before the NONE_task starts
        assert_eq!(
            NONE_task.workload_on_day(NaiveDate::from_ymd(2999,12,31)),
            0
        );

        // A date after the NONE_task ends
        assert_eq!(
            NONE_task.workload_on_day(NaiveDate::from_ymd(3000,01,09)),
            0
        );

        // A weekend during the NONE_task
        assert_eq!(
            NONE_task.workload_on_day(NaiveDate::from_ymd(3000,01,04)),
            0
        );

        // First day of the NONE_task
        assert_eq!(
            NONE_task.workload_on_day(NaiveDate::from_ymd(3000,01,01)),
            0
        );

        // Last day of the NONE_task
        assert_eq!(
            NONE_task.workload_on_day(NaiveDate::from_ymd(3000,01,08)),
            0
        );

        // Weekday in middle of NONE_task
        assert_eq!(
            NONE_task.workload_on_day(NaiveDate::from_ymd(3000,01,06)),
            0
        );
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
