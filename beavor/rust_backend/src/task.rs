use pyo3::prelude::{
    pyclass,
    pymethods,
};

use chrono::NaiveDate;

use crate::today_date;

use crate::due_date::{
    DueDate,
    PyDueDate,
};

#[derive(Clone)]
#[pyclass]
pub struct Task{
    #[pyo3(get, set)]
    pub category:         String,
    #[pyo3(get, set)]
    pub finished:         bool,
    #[pyo3(get, set)]
    pub name:             String,
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
}

#[pymethods]
impl Task{
    #[staticmethod]
    pub fn default() -> Self{
        Task{
            category:         "Work".into(),
            finished:         false,
            name:             "".into(),
            _time_budgeted:   0,
            time_needed:      0,
            time_used:        0,
            next_action_date: today_date(),
            due_date:         DueDate::Date(today_date()),
            notes:            "".into(),
            id:               None,
            date_added:       today_date(),
        }
    }

    #[getter]
    fn get_due_date(&self) -> PyDueDate{
        self.due_date.into()
    }

    #[setter]
    fn set_due_date(&mut self, due_date: PyDueDate){
        self.due_date = (&due_date).into()
    }
}
