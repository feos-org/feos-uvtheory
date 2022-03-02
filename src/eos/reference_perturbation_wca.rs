use super::hard_sphere_wca::{
    diameter_q_wca, diameter_wca, packing_fraction, packing_fraction_a, packing_fraction_b,
};
use crate::parameters::*;
use feos_core::{HelmholtzEnergyDual, StateHD};
use num_dual::DualNum;
use std::fmt;
use std::{f64::consts::PI, rc::Rc};

#[derive(Debug, Clone)]
pub struct ReferencePerturbationWCA {
    pub parameters: Rc<UVParameters>,
}

impl fmt::Display for ReferencePerturbationWCA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reference Perturbation")
    }
}

impl<D: DualNum<f64>> HelmholtzEnergyDual<D> for ReferencePerturbationWCA {
    /// Helmholtz energy for perturbation reference (Mayer-f), eq. 29
    fn helmholtz_energy(&self, state: &StateHD<D>) -> D {
        let p = &self.parameters;
        let n = p.sigma.len();
        let x = &state.molefracs;
        let d = diameter_wca(&p, state.temperature);
        let q = diameter_q_wca(&p, state.temperature);
        let eta = packing_fraction(&state.partial_density, &d);
        let eta_a = packing_fraction_a(p, eta, state.temperature);
        let eta_b = packing_fraction_b(p, eta, state.temperature);
        let mut a = D::zero();

        for i in 0..n {
            for j in 0..n {
                let rs_ij = ((p.rep[i] / p.att[i]).powf(1.0 / (p.rep[i] - p.att[i]))
                    + (p.rep[j] / p.att[j]).powf(1.0 / (p.rep[j] - p.att[j])))
                    * 0.5; // MIXING RULE not clear!!!
                let d_ij = (d[i] + d[j]) * 0.5; // (d[i] * p.sigma[i] + d[j] * p.sigma[j]) * 0.5;
                let q_ij = (q[i] + q[j]) * 0.5;

                a += x[i]
                    * x[j]
                    * ((-eta_a[[i, j]] * 0.5 + 1.0) / (-eta_a[[i, j]] + 1.0).powi(3)
                        * (-q_ij.powi(3) + (rs_ij * p.sigma_ij[[i, j]]).powi(3))
                        - ((-eta_b[[i, j]] * 0.5 + 1.0) / (-eta_b[[i, j]] + 1.0).powi(3))
                            * (-d_ij.powi(3) + (rs_ij * p.sigma_ij[[i, j]]).powi(3)))
            }
        }

        -a * state.moles.sum().powi(2) * 2.0 / 3.0 / state.volume * PI
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parameters::utils::{test_parameters, test_parameters_mixture};
    use approx::assert_relative_eq;
    use ndarray::arr1;

    #[test]
    fn test_delta_a0_wca_pure() {
        let moles = arr1(&[2.0]);

        // m = 12.0, t = 4.0, rho = 1.0

        let reduced_temperature = 4.0;
        let reduced_density = 1.0;
        let reduced_volume = moles[0] / reduced_density;

        let p = test_parameters(24.0, 6.0, 1.0, 1.0);
        let pt = ReferencePerturbationWCA {
            parameters: Rc::new(p),
        };
        let state = StateHD::new(reduced_temperature, reduced_volume, moles.clone());
        let a = pt.helmholtz_energy(&state) / moles[0];
        assert_relative_eq!(a, 0.258690311450425, epsilon = 1e-10);
    }
    #[test]
    fn test_delta_a0_wca_mixture() {
        let moles = arr1(&[1.7, 0.3]);
        let reduced_temperature = 4.0;
        let reduced_density = 1.0;
        let reduced_volume = (moles[0] + moles[1]) / reduced_density;

        let p = test_parameters_mixture(
            arr1(&[24.0, 24.0]),
            arr1(&[6.0, 6.0]),
            arr1(&[1.0, 1.0]),
            arr1(&[1.0, 1.0]),
        );
        let d = diameter_wca(&p, reduced_temperature);
        let q = diameter_q_wca(&p, reduced_temperature);

        let pt = ReferencePerturbationWCA {
            parameters: Rc::new(p),
        };
        let state = StateHD::new(reduced_temperature, reduced_volume, moles.clone());
        let a = pt.helmholtz_energy(&state) / (moles[0] + moles[1]);
        assert_relative_eq!(a, 0.258690311450425, epsilon = 1e-10);
    }
}
