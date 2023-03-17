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
pub struct Task{
    #[pyo3(get, set)]
    pub name:          String,
    #[pyo3(get, set)]
    pub finished:      String,
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
}

#[pyclass]
#[derive(Clone)]
pub struct External{
    #[pyo3(get, set)]
    name: String,
    #[pyo3(get, set)]
    link: String, // this should maybe be a more specific type, like a URL or somesuch
}

#[pyclass]
#[derive(Clone)]
pub struct Deliverable{
    #[pyo3(get, set)]
    name:      String,
    due:       DueDate,
    #[pyo3(get, set)]
    notes:     String,
    #[pyo3(get, set)]
    tasks:     Vec<Task>,
    #[pyo3(get, set)]
    externals: Vec<External>,
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
    name: String,
    #[pyo3(get, set)]
    deliverables: Vec<Deliverable>,
}

#[pyclass]
#[derive(Clone)]
pub struct Category{
    #[pyo3(get, set)]
    name:     String,
    #[pyo3(get, set)]
    projects: Vec<Project>,
}
