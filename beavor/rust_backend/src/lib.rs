use pyo3::prelude::{
    pyfunction,
    pymodule,
    PyResult,
    PyModule,
    Python
};

use pyo3::wrap_pyfunction;

mod date;
use date::{
    PyDueDate,
    PyDueDateType,
    PyAvailability,
    PyAvailabilityType,
    today_date,
    parse_date,
    format_date,
    today_str,
};

mod model;
use model::{
    Task,
    Deliverable,
    Project,
    Category,
    External,
};

mod database;
use database::DatabaseManager;

#[pyfunction]
fn green_red_scale(low: f32, high: f32, val: f32) -> String {
    let frac = f32::max(0.0,f32::min(1.0,(val-low)/(high-low)));

    let red: u8;
    let green: u8;

    if frac > 0.5{
        red = 255;
        green = ((2.0-2.0*frac) * 255.0) as u8;
    }else{
        red = ((2.0*frac) * 255.0) as u8;
        green = 255
    }

    format!("#{red:02X}{green:02X}00")
}

#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_date, m)?)?;
    m.add_function(wrap_pyfunction!(green_red_scale, m)?)?;
    m.add_function(wrap_pyfunction!(parse_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_str, m)?)?;
    m.add_class::<Task>().unwrap();
    m.add_class::<Deliverable>().unwrap();
    m.add_class::<Project>().unwrap();
    m.add_class::<Category>().unwrap();
    m.add_class::<External>().unwrap();
    m.add_class::<PyDueDate>().unwrap();
    m.add_class::<PyDueDateType>().unwrap();
    m.add_class::<DatabaseManager>().unwrap();
    m.add_class::<PyAvailability>().unwrap();
    m.add_class::<PyAvailabilityType>().unwrap();
    Ok(())
}
