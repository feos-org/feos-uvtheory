use feos_core::python::*;
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
pub mod eos;
pub mod parameters;
use eos::*;
use parameters::*;
use quantity::python::PyInit_quantity;

#[pymodule]
pub fn feos_uvtheory(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVerbosity>()?;
    m.add_class::<PyContributions>()?;
    m.add_class::<PyUVParameters>()?;

    m.add_wrapped(wrap_pymodule!(eos))?;
    m.add_wrapped(wrap_pymodule!(quantity))?;

    py.run(
        "\
import sys
sys.modules['feos_uvtheory.eos'] = eos
sys.modules['feos_uvtheory.eos.utils'] = eos.utils
quantity.SINumber.__module__ = 'feos_pcsaft.si'
quantity.SIArray1.__module__ = 'feos_pcsaft.si'
quantity.SIArray2.__module__ = 'feos_pcsaft.si'
quantity.SIArray3.__module__ = 'feos_pcsaft.si'
quantity.SIArray4.__module__ = 'feos_pcsaft.si'
sys.modules['feos_uvtheory.si'] = quantity
    ",
        None,
        Some(m.dict()),
    )?;
    Ok(())
}
