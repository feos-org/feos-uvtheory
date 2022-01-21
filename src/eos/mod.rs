use crate::parameters::UVParameters;
use feos_core::{parameter::Parameter, EquationOfState, HelmholtzEnergy};
use ndarray::Array1;
use std::f64::consts::FRAC_PI_6;
use std::rc::Rc;

pub(crate) mod attractive_perturbation_bh;
pub(crate) mod attractive_perturbation_wca;
pub(crate) mod hard_sphere_bh;
pub(crate) mod hard_sphere_wca;
pub(crate) mod reference_perturbation_bh;
pub(crate) mod reference_perturbation_wca;
use attractive_perturbation_bh::AttractivePerturbationBH;
use attractive_perturbation_wca::AttractivePerturbationWCA;
use hard_sphere_bh::HardSphere;
use hard_sphere_wca::HardSphereWCA;
use reference_perturbation_bh::ReferencePerturbationBH;
use reference_perturbation_wca::ReferencePerturbationWCA;
#[derive(Copy, Clone)]
pub enum Perturbation {
    BarkerHenderson,
    WeeksChandlerAndersen,
}

#[derive(Copy, Clone)]
pub struct UVTheoryOptions {
    pub max_eta: f64,
    pub perturbation: Perturbation,
}

impl Default for UVTheoryOptions {
    fn default() -> Self {
        Self {
            max_eta: 0.5,
            perturbation: Perturbation::WeeksChandlerAndersen,
        }
    }
}

pub struct UVTheory {
    parameters: Rc<UVParameters>,
    options: UVTheoryOptions,
    contributions: Vec<Box<dyn HelmholtzEnergy>>,
}

impl UVTheory {
    pub fn new(parameters: Rc<UVParameters>) -> Self {
        Self::with_options(parameters, UVTheoryOptions::default())
    }

    pub fn with_options(parameters: Rc<UVParameters>, options: UVTheoryOptions) -> Self {
        let mut contributions: Vec<Box<dyn HelmholtzEnergy>> = Vec::with_capacity(3);

        match options.perturbation {
            Perturbation::BarkerHenderson => {
                contributions.push(Box::new(HardSphere {
                    parameters: parameters.clone(),
                }));
                contributions.push(Box::new(ReferencePerturbationBH {
                    parameters: parameters.clone(),
                }));
                contributions.push(Box::new(AttractivePerturbationBH {
                    parameters: parameters.clone(),
                }));
            }
            Perturbation::WeeksChandlerAndersen => {
                contributions.push(Box::new(HardSphereWCA {
                    parameters: parameters.clone(),
                }));
                contributions.push(Box::new(ReferencePerturbationWCA {
                    parameters: parameters.clone(),
                }));
                contributions.push(Box::new(AttractivePerturbationWCA {
                    parameters: parameters.clone(),
                }));
            }
        }

        Self {
            parameters: parameters.clone(),
            options,
            contributions,
        }
    }
}

impl EquationOfState for UVTheory {
    fn components(&self) -> usize {
        self.parameters.pure_records.len()
    }

    fn subset(&self, component_list: &[usize]) -> Self {
        Self::with_options(
            Rc::new(self.parameters.subset(component_list)),
            self.options,
        )
    }

    fn compute_max_density(&self, moles: &Array1<f64>) -> f64 {
        self.options.max_eta * moles.sum()
            / (FRAC_PI_6 * self.parameters.sigma.mapv(|v| v.powi(3)) * moles).sum()
    }

    fn residual(&self) -> &[Box<dyn HelmholtzEnergy>] {
        &self.contributions
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parameters::*;
    use approx::assert_relative_eq;
    use feos_core::parameter::{Identifier, Parameter, PureRecord};
    use feos_core::{Contributions, State};
    use ndarray::arr1;
    use quantity::si::{ANGSTROM, KELVIN, MOL, NAV, RGAS};

    #[test]
    fn helmholtz_energy() {
        let eps_k = 150.03;
        let sig = 3.7039;
        let r = UVRecord::new(24.0, 6.0, sig, eps_k);
        let i = Identifier::new("1", None, None, None, None, None);
        let pr = PureRecord::new(i, 1.0, r, None);
        let parameters = UVParameters::new_pure(pr);
        let eos = Rc::new(UVTheory::new(Rc::new(parameters)));

        let reduced_temperature = 4.0;
        let reduced_density = 1.0;
        let temperature = reduced_temperature * eps_k * KELVIN;
        let moles = arr1(&vec![2.0]) * MOL;
        let volume = (sig * ANGSTROM).powi(3) / reduced_density * NAV * 2.0 * MOL;
        let s = State::new_nvt(&eos, temperature, volume, &moles).unwrap();
        let a = s
            .molar_helmholtz_energy(Contributions::Residual)
            .to_reduced(RGAS * temperature)
            .unwrap();

        assert_relative_eq!(a, 2.972986567516, max_relative = 1e-12) //wca
                                                                     // bh: assert_relative_eq!(a, 2.993577305779432, max_relative = 1e-12)
    }
}
