// This file should contain only the public API, i.e. the pymodule exports,
// and the use / mod lines required for that
use pyo3::{
    wrap_pyfunction,
    prelude::{
        pymodule,
        PyResult,
        PyModule,
        Python
    }
};

mod database;
use database::PyDatabaseManager;

mod due_date;
use due_date::{
    DueDate,
    PyDueDate,
    PyDueDateType,
    ParseDateError
};

mod task;
use task::Task;

mod utils;
use utils::{
    green_red_scale,
    today_date,
    today_string,
    format_date,
    parse_date,
};

mod schedule;
use schedule::Schedule;

#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_date, m)?)?;
    m.add_function(wrap_pyfunction!(green_red_scale, m)?)?;
    m.add_function(wrap_pyfunction!(parse_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_string, m)?)?;
    m.add_class::<Task>().unwrap();
    m.add_class::<PyDueDate>().unwrap();
    m.add_class::<PyDueDateType>().unwrap();
    m.add_class::<PyDatabaseManager>().unwrap();
    m.add_class::<Schedule>().unwrap();
    Ok(())
}
