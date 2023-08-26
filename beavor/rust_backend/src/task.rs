use pyo3::prelude::{
    pyclass,
    pymethods,
};

use chrono::{
    Datelike,
    Weekday,
    NaiveDate,
};

use std::cmp::max;

use crate::due_date::{
    DueDate,
    PyDueDate,
};

use crate::utils::{
    work_days_from,
    today_date,
};

#[derive(Clone)]
#[pyclass]
pub struct Task{
    #[pyo3(get, set)]
    pub category:         String,
    #[pyo3(get, set)]
    pub finished:         String, // TODO It's inexcusable that this is a string and not an Enum
    #[pyo3(get, set)]
    pub task_name:        String,
    #[pyo3(get)]
    pub _time_budgeted:   i32,
    #[pyo3(get, set)]
    pub time_needed:      i32,
    #[pyo3(get, set)]
    pub time_used:        i32,
    #[pyo3(get, set)]
    pub notes:            String,
    #[pyo3(get, set)]
    pub date_added:       NaiveDate,
    #[pyo3(get, set)]
    pub next_action_date: NaiveDate,
    pub due_date:         DueDate,
    #[pyo3(get)]
    pub id:               Option<i32>,
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
