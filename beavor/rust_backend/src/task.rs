use pyo3::prelude::{
    pyclass,
    pymethods,
};

use chrono::NaiveDate;

use crate::due_date::{
    DueDate,
    PyDueDate,
};

use crate::utils::today_date;

use std::cmp::max;

#[derive(Clone)]
#[pyclass]
pub struct Task{
    #[pyo3(get, set)]
    pub category:         String,
    #[pyo3(get, set)]
    pub finished:         String, // TODO It's inexcusable that this is a string and not an Enum
    #[pyo3(get, set)]
    pub task_name:        String, // TODO this should just be `name`
    #[pyo3(get)]
    pub _time_budgeted:   u32,
    #[pyo3(get, set)]
    pub time_needed:      u32,
    #[pyo3(get, set)]
    pub time_used:        u32,
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

impl Task{
    pub fn time_remaining(&self) -> u32{
        self.time_needed - self.time_used
    }

    pub fn first_available_date(&self) -> NaiveDate{
        max(today_date(), self.next_action_date)
    }

    pub fn last_available_date(&self) -> Option<NaiveDate>{
        match self.due_date{
            DueDate::NONE => None,
            DueDate::Date(due_date) => Some(max(due_date, today_date())),
            DueDate::ASAP => Some(self.first_available_date()),
        }
    }
}

impl IntoIterator for Task{
    type Item = NaiveDate;
    type IntoIter = DateIterator;

    fn into_iter(self) -> Self::IntoIter {
        DateIterator{
            prev: self.first_available_date(),
            next: self.first_available_date(),
            last: self.last_available_date(),
        }
    }
}

/// This stores more state than necessary but I don't feel like optimizing it and I doubt it'll be
/// a bottleneck anytime soon
pub struct DateIterator{
    prev: NaiveDate,
    next: NaiveDate,
    last: Option<NaiveDate>,
}

impl Iterator for DateIterator{
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        match self.last{
            Some(date) => {
                self.prev = self.next;
                self.next = self.prev.succ_opt().expect("This panics on huge dates");
                Some(self.prev).filter(|d| *d <= date)
            },
            None => None,
        }
    }
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
}

#[cfg(test)]
#[allow(deprecated)]
#[allow(clippy::zero_prefixed_literal)]
mod tests{
    use super::*;

    #[test]
    fn test_task_date_iteration() {
        let task_date = Task{
            category: "".to_string(),
            finished: "O".to_string(),
            task_name: "".to_string(),
            _time_budgeted: 0,
            time_needed: 0,
            time_used: 0,
            notes: "".to_string(),
            date_added: NaiveDate::from_ymd(3000, 01, 01),
            next_action_date: NaiveDate::from_ymd(3000, 01, 01),
            due_date: DueDate::Date(NaiveDate::from_ymd(3000, 01, 03)),
            id: None,
        };

        assert_eq!(task_date.into_iter().count(), 3);

        let task_asap = Task{
            category: "".to_string(),
            finished: "O".to_string(),
            task_name: "".to_string(),
            _time_budgeted: 0,
            time_needed: 0,
            time_used: 0,
            notes: "".to_string(),
            date_added: NaiveDate::from_ymd(3000, 01, 01),
            next_action_date: NaiveDate::from_ymd(3000, 01, 01),
            due_date: DueDate::ASAP,
            id: None,
        };

        assert_eq!(task_asap.into_iter().count(), 1);

        let task_none = Task{
            category: "".to_string(),
            finished: "O".to_string(),
            task_name: "".to_string(),
            _time_budgeted: 0,
            time_needed: 0,
            time_used: 0,
            notes: "".to_string(),
            date_added: NaiveDate::from_ymd(3000, 01, 01),
            next_action_date: NaiveDate::from_ymd(3000, 01, 01),
            due_date: DueDate::NONE,
            id: None,
        };

        assert_eq!(task_none.into_iter().count(), 0);
    }
}
