use feos_core::python::parameter::*;
use feos_core::{Contributions, Verbosity};
use feos_uvtheory::python::*;
use feos_uvtheory::Perturbation;
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use quantity::python::__PYO3_PYMODULE_DEF_QUANTITY;

mod eos;
use eos::__PYO3_PYMODULE_DEF_EOS;

#[pymodule]
pub fn feos_uvtheory(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyIdentifier>()?;
    m.add_class::<Verbosity>()?;
    m.add_class::<Contributions>()?;
    m.add_class::<Perturbation>()?;

    m.add_class::<PyUVRecord>()?;
    m.add_class::<PyPureRecord>()?;
    m.add_class::<PyUVParameters>()?;

    m.add_wrapped(wrap_pymodule!(eos))?;
    m.add_wrapped(wrap_pymodule!(quantity))?;
    py.run(
        "\
import sys
sys.modules['feos_uvtheory.eos'] = eos
quantity.SINumber.__module__ = 'feos_uvtheory.si'
quantity.SIArray1.__module__ = 'feos_uvtheory.si'
quantity.SIArray2.__module__ = 'feos_uvtheory.si'
quantity.SIArray3.__module__ = 'feos_uvtheory.si'
quantity.SIArray4.__module__ = 'feos_uvtheory.si'
sys.modules['feos_uvtheory.si'] = quantity
    ",
        None,
        Some(m.dict()),
    )?;
    Ok(())
}
