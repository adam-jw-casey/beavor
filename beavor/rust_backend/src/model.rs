use pyo3::prelude::{
    pyclass,
    pymethods
};

use crate::date::{
    Availability,
    PyAvailability,
    DueDate,
    PyDueDate,
};

#[derive(Clone)]
#[pyclass]
enum TaskStatus{
    Open,
    Complete,
    Archived,
}

#[derive(Debug)]
struct ParseTaskStatusError;

impl TryFrom<&String> for TaskStatus{
    type Error = ParseTaskStatusError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str(){
            "Open"     => Ok(TaskStatus::Open),
            "Complete" => Ok(TaskStatus::Complete),
            "Archived" => Ok(TaskStatus::Archived),
            _ => Err(ParseTaskStatusError)
        }
    }
}

impl From<&TaskStatus> for String{
    fn from(value: &TaskStatus) -> Self {
        match value{
            TaskStatus::Open => "Open",
            TaskStatus::Complete => "Complete",
            TaskStatus::Archived => "Archived",
        }.to_string()
    }
}

#[derive(Clone)]
#[pyclass]
pub struct Task{
    #[pyo3(get, set)]
    pub name:          String,
    #[pyo3(get, set)]
    pub status:        TaskStatus,
    #[pyo3(get, set)]
    pub time_needed:   i32,
    #[pyo3(get, set)]
    pub time_used:     i32,
    pub available:     Availability,
    #[pyo3(get, set)]
    pub notes:         String,
    #[pyo3(get)]
    pub id:            Option<i64>,
}

#[pymethods]
impl Task{
    #[getter]
    fn get_available(&self) -> PyAvailability{
        (&self.available).into()
    }

    #[setter]
    fn set_available(&mut self, availability: PyAvailability){
        self.available = (&availability).into();
    }

    #[staticmethod]
    pub fn default() -> Task{
        Task{
            name:             "".into(),
            status:           TaskStatus::Open,
            time_needed:      0,
            time_used:        0,
            available:        Availability::Any,
            notes:            "".into(),
            id:               None,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct External{
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub link: String, // this should maybe be a more specific type, like a URL or somesuch
    pub id:   Option<i64>,
}

#[pyclass]
#[derive(Clone)]
pub struct Deliverable{
    #[pyo3(get, set)]
    pub name:      String,
    pub due:       DueDate,
    #[pyo3(get, set)]
    pub notes:     String,
    #[pyo3(get, set)]
    pub tasks:     Vec<Task>,
    #[pyo3(get, set)]
    pub externals: Vec<External>,
    pub id:        Option<i64>,
}

#[pymethods]
impl Deliverable{
    #[getter]
    fn get_due(&self) -> PyDueDate{
        (&self.due).into()
    }

    #[setter]
    fn set_due(&mut self, due: PyDueDate){
        self.due = (&due).into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Project{
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub deliverables: Vec<Deliverable>,
    pub id:           Option<i64>,
}

#[pyclass]
#[derive(Clone)]
pub struct Category{
    #[pyo3(get, set)]
    pub name:     String,
    #[pyo3(get, set)]
    pub projects: Vec<Project>,
    pub id:       Option<i64>,
}
