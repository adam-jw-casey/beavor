use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
fn surround(mut inner: String, mut outer: String) -> PyResult<String> {
    inner.push_str(&outer);
    outer.push_str(&inner);
    Ok(outer)
}

/// A Python module implemented in Rust.
#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(surround, m)?)?;
    Ok(())
}
