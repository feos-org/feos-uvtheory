use super::parameters::PyUVParameters;
use crate::eos::{Perturbation, UVTheory, UVTheoryOptions};
use feos_core::python::{PyContributions, PyVerbosity};
use feos_core::utils::{
    DataSet, EquilibriumLiquidDensity, Estimator, LiquidDensity, VaporPressure,
};
use feos_core::*;
use numpy::convert::ToPyArray;
use numpy::{PyArray1, PyArray2};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use quantity::python::*;
use quantity::si::*;
use std::collections::HashMap;
use std::rc::Rc;

/// Possible ways to define a perturbation potential.
#[pyclass(name = "Perturbation")]
#[derive(Copy, Clone)]
pub struct PyPerturbation(pub Perturbation);

#[pymethods]
impl PyPerturbation {
    /// Use Barker-Handerson division for perturbation potential.
    #[classattr]
    #[allow(non_snake_case)]
    pub fn BarkerHenderson() -> Self {
        Self(Perturbation::BarkerHenderson)
    }

    /// Use Weeks-Chandler-Andersen division for perturbation potential.
    #[classattr]
    #[allow(non_snake_case)]
    pub fn WeeksChandlerAndersen() -> Self {
        Self(Perturbation::WeeksChandlerAndersen)
    }
}

/// Initialize UV Theory equation of state.
///
/// Parameters
/// ----------
/// parameters : UVTheoryParameters
///     The parameters of the UV Theory equation of state to use.
/// max_eta : float, optional
///     Maximum packing fraction. Defaults to 0.5.
///
/// Returns
/// -------
/// UVTheory
///     The UV Theory equation of state that can be used to compute thermodynamic
///     states.
#[pyclass(name = "UVTheory", unsendable)]
#[pyo3(text_signature = "(parameters, max_eta, perturbation)")]
#[derive(Clone)]
pub struct PyUVTheory(pub Rc<UVTheory>);

#[pymethods]
impl PyUVTheory {
    #[new]
    #[args(max_eta = "0.5")]
    fn new(parameters: PyUVParameters, max_eta: f64, perturbation: Option<PyPerturbation>) -> Self {
        let options = match perturbation {
            Some(p) => UVTheoryOptions {
                max_eta,
                perturbation: p.0,
            },
            None => UVTheoryOptions {
                max_eta,
                perturbation: Perturbation::WeeksChandlerAndersen,
            },
        };
        Self(Rc::new(UVTheory::with_options(
            parameters.0.clone(),
            options,
        )))
    }
}

impl_equation_of_state!(PyUVTheory);
impl_virial_coefficients!(PyUVTheory);

impl_state!(UVTheory, PyUVTheory);
impl_vle_state!(UVTheory, PyUVTheory);
// impl_estimator!(UVTheory, PyUVTheory);

#[pymodule]
pub fn eos(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUVTheory>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyPerturbation>()?;
    m.add_class::<PyPhaseDiagramPure>()?;
    m.add_class::<PyPhaseDiagramBinary>()?;
    m.add_class::<PyPhaseDiagramHetero>()?;
    m.add_class::<PyPhaseEquilibrium>()?;

    let utils = PyModule::new(py, "utils")?;
    m.add_submodule(utils)?;
    Ok(())
}
