use pyo3::prelude::{
    pyclass,
    pymethods,
};

use chrono::naive::NaiveDate;

use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use crate::date::{
    DueDate,
    PyDueDate,
    ParseDateError,
    parse_date,
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

impl TryFrom<SqliteRow> for Task{
    type Error = ParseDateError;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(Task{
            category:                     row.get::<String, &str>("Category"),
            finished:                     row.get::<String, &str>("O"),
            task_name:                    row.get::<String, &str>("Task"),
            _time_budgeted:               row.get::<i32,    &str>("Budget"),
            time_needed:                  row.get::<i32,    &str>("Time"),
            time_used:                    row.get::<i32,    &str>("Used"),
            next_action_date: parse_date(&row.get::<String, &str>("NextAction"))?,
            due_date:                     row.get::<String, &str>("DueDate").try_into()?,
            notes:                        row.get::<String, &str>("Notes"),
            id:                           row.get::<Option<i32>, &str>("rowid"),
            date_added:       parse_date(&row.get::<String, &str>("DateAdded"))?,
        })
    }
}
