use feos_uvtheory::python::feos_uvtheory;
use pyo3::prelude::*;

#[pymodule]
pub fn build_wheel(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    feos_uvtheory(py, m)
}
