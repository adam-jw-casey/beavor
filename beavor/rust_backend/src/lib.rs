use pyo3::prelude::{
    pyfunction,
    pymodule,
    pyclass,
    pymethods,
    PyResult,
    PyModule,
    Python
};
use pyo3::wrap_pyfunction;

use chrono::{
    Datelike, // This isn't explicitly used, but gives access to certain trait methods on NaiveDate
    Weekday,
    NaiveDate,
};

use std::cmp::max;

mod database;
use database::DatabaseManager;

mod date;
use date::{
    DueDate,
    PyDueDate,
    PyDueDateType,
    today_date,
    today_str,
    work_days_from,
    format_date,
    parse_date,
    ParseDateError
};

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
}
