use feos_core::*;
use feos_uvtheory::python::PyUVParameters;
use feos_uvtheory::{UVTheory, UVTheoryOptions, Perturbation};
use numpy::convert::ToPyArray;
use numpy::{PyArray1, PyArray2};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use quantity::python::*;
use quantity::si::*;
use std::collections::HashMap;
use std::rc::Rc;


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
    fn new(parameters: PyUVParameters, max_eta: f64, perturbation: Option<Perturbation>) -> Self {
        let options = match perturbation {
            Some(p) => UVTheoryOptions {
                max_eta,
                perturbation: p,
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
impl_phase_equilibrium!(UVTheory, PyUVTheory);

#[pymodule]
pub fn eos(_: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUVTheory>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyPhaseDiagram>()?;
    m.add_class::<PyPhaseEquilibrium>()
}
