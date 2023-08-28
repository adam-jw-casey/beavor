use pyo3::prelude::{
    pyclass,
    pymethods,
};

use chrono::{
    NaiveDate,
};

use std::cmp::max;

use crate::due_date::{
    DueDate,
    PyDueDate,
};

use crate::calendar::work_days_from;
use crate::utils::today_date;

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
}
