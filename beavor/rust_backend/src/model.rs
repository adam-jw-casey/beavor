use pyo3::prelude::{
    pyclass,
    pymethods
};

use crate::date::{
    Availability,
    PyAvailability,
};

#[derive(Clone)]
#[pyclass]
pub struct Task{
    #[pyo3(get, set)]
    pub task_name:        String,
    #[pyo3(get, set)]
    pub finished:         String,
    #[pyo3(get, set)]
    pub time_needed:      i32,
    #[pyo3(get, set)]
    pub time_used:        i32,
    pub available:        Availability,
    #[pyo3(get, set)]
    pub notes:            String,
    #[pyo3(get)]
    pub id:               Option<i64>,
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
