use crate::parameters::{NoRecord, UVBinaryRecord, UVParameters, UVRecord};
use feos_core::parameter::{
    Identifier, IdentifierOption, Parameter, ParameterError, PureRecord, SegmentRecord,
};
use feos_core::python::parameter::{
    PyBinaryRecord, PyBinarySegmentRecord, PyChemicalRecord, PyIdentifier,
};
use feos_core::*;
use ndarray::Array2;
use numpy::PyArray2;
use pyo3::prelude::*;
use std::convert::TryFrom;
use std::rc::Rc;

/// Create a set of UV Theory parameters from records.
#[pyclass(name = "NoRecord", unsendable)]
#[derive(Clone)]
struct PyNoRecord(NoRecord);

/// Create a set of UV Theory parameters from records.
#[pyclass(name = "UVRecord", unsendable)]
#[pyo3(text_signature = "(rep, att, sigma, epsilon_k)")]
#[derive(Clone)]
pub struct PyUVRecord(UVRecord);

#[pymethods]
impl PyUVRecord {
    #[new]
    fn new(rep: f64, att: f64, sigma: f64, epsilon_k: f64) -> Self {
        Self(UVRecord::new(rep, att, sigma, epsilon_k))
    }
}

#[pyproto]
impl pyo3::class::basic::PyObjectProtocol for PyUVRecord {
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.to_string())
    }
}

impl_json_handling!(PyUVRecord);

/// Create a set of UV Theory parameters from records.
///
/// Parameters
/// ----------
/// pure_records : List[PureRecord]
///     pure substance records.
/// binary_records : List[BinarySubstanceRecord], optional
///     binary saft parameter records
/// substances : List[str], optional
///     The substances to use. Filters substances from `pure_records` according to
///     `search_option`.
///     When not provided, all entries of `pure_records` are used.
/// search_option : {'Name', 'Cas', 'Inchi', 'IupacName', 'Formula', 'Smiles'}, optional, defaults to 'Name'.
///     Identifier that is used to search substance.
#[pyclass(name = "UVParameters", unsendable)]
#[pyo3(
    text_signature = "(pure_records, binary_records=None, substances=None, search_option='Name')"
)]
#[derive(Clone)]
pub struct PyUVParameters(pub Rc<UVParameters>);

#[pymethods]
impl PyUVParameters {
    /// Create a set of UV Theory parameters from lists.
    ///
    /// Parameters
    /// ----------
    /// rep : List[float]
    ///     repulsive exponents
    /// att : List[float]
    ///     attractive exponents
    /// sigma : List[float]
    ///     Mie diameter in units of Angstrom
    /// epsilon_k : List[float]
    ///     Mie energy parameter in units of Kelvin
    ///
    /// Returns
    /// -------
    /// UVParameters
    #[pyo3(text_signature = "(rep, att, sigma, epsilon_k)")]
    #[staticmethod]
    fn from_lists(rep: Vec<f64>, att: Vec<f64>, sigma: Vec<f64>, epsilon_k: Vec<f64>) -> Self {
        let n = rep.len();
        let pure_records = (0..n)
            .map(|i| {
                let identifier =
                    Identifier::new(format!("{}", i).as_str(), None, None, None, None, None);
                let model_record = UVRecord::new(rep[i], att[i], sigma[i], epsilon_k[i]);
                PureRecord::new(identifier, 1.0, model_record, None)
            })
            .collect();
        let binary = Array2::from_shape_fn((n, n), |(_, _)| UVBinaryRecord { k_ij: 0.0 });
        Self(Rc::new(UVParameters::from_records(pure_records, binary)))
    }
}

impl_pure_record!(UVRecord, PyUVRecord, NoRecord, PyNoRecord);
impl_parameter!(UVParameters, PyUVParameters);
