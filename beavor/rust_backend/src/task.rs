use pyo3::prelude::{
    pyclass,
    pymethods,
};

use chrono::naive::NaiveDate;

use crate::date::{
    DueDate,
    PyDueDate,
};

#[derive(Clone)]
#[pyclass]
pub struct Task{
    #[pyo3(get, set)]
    pub category:         String,
    #[pyo3(get, set)]
    pub finished:         String,
    #[pyo3(get, set)]
    pub task_name:        String,
    pub _time_budgeted:   i32,
    #[pyo3(get, set)]
    pub time_needed:      i32,
    #[pyo3(get, set)]
    pub time_used:        i32,
    #[pyo3(get, set)]
    pub next_action_date: NaiveDate,
    #[pyo3(get, set)]
    pub notes:            String,
    pub date_added:       NaiveDate,
    #[pyo3(get)]
    pub id:               Option<i32>,
    pub due_date:         DueDate,
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
