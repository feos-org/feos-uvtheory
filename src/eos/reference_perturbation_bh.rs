use super::hard_sphere_bh::{
    diameter_bh, packing_fraction, packing_fraction_a, packing_fraction_b,
};
use crate::parameters::*;
use feos_core::{HelmholtzEnergyDual, StateHD};
use itertools::Itertools;
use num_dual::DualNum;
use std::fmt;
use std::{f64::consts::PI, rc::Rc};

#[derive(Debug, Clone)]
pub struct ReferencePerturbationBH {
    pub parameters: Rc<UVParameters>,
}

impl fmt::Display for ReferencePerturbationBH {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reference Perturbation")
    }
}

impl<D: DualNum<f64>> HelmholtzEnergyDual<D> for ReferencePerturbationBH {
    /// Helmholtz energy for perturbation reference (Mayer-f), eq. 29
    fn helmholtz_energy(&self, state: &StateHD<D>) -> D {
        let p = &self.parameters;
        let n = p.sigma.len();
        let x = &state.molefracs;
        let d = diameter_bh(&p, state.temperature);
        let eta = packing_fraction(&state.partial_density, &d);
        let eta_a = packing_fraction_a(p, &d, eta);
        let eta_b = packing_fraction_b(p, &d, eta);
        let mut a = D::zero();
        for i in 0..n {
            for j in 0..n {
                let d_ij = (d[i] + d[j]) * 0.5; // (d[i] * p.sigma[i] + d[j] * p.sigma[j]) * 0.5;
                a += x[i]
                    * x[j]
                    * (((-eta_a[[i, j]] * 0.5 + 1.0) / (-eta_a[[i, j]] + 1.0).powi(3))
                        - ((-eta_b[[i, j]] * 0.5 + 1.0) / (-eta_b[[i, j]] + 1.0).powi(3)))
                    * (-d_ij.powi(3) + p.sigma_ij[[i, j]].powi(3))
            }
        }
        -a * state.moles.sum().powi(2) * 2.0 / 3.0 / state.volume * PI
    }
}

/// Boltzmann factor of the Mie potential.
///
/// m: repulsive exponent
/// reduced_temperature: T / epsilon_k
/// reduced_separation: r / sigma
fn boltzmann_factor<D: DualNum<f64>>(m: f64, reduced_temperature: D, reduced_separation: D) -> D {
    (-reduced_temperature.recip()
        * mie_prefactor(m, 6.0)
        * (reduced_separation.powf(-m) - reduced_separation.powf(-6.0)))
    .exp()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parameters::utils::{methane_parameters, test_parameters};
    use approx::assert_relative_eq;
    use ndarray::{arr1, Array1};

    fn state_from_reduced<D: DualNum<f64>>(
        reduced_temperature: D,
        reduced_density: D,
        moles: &Array1<D>,
        sigma: f64,
        epsilon_k: f64,
    ) -> StateHD<D> {
        let temperature = reduced_temperature * epsilon_k;
        let reduced_volume = moles.sum() / reduced_density;
        let volume = reduced_volume * sigma.powi(3);
        StateHD::new(temperature, volume, moles.clone())
    }

    #[test]
    fn test_delta_a0_bh() {
        let moles = arr1(&[2.0]);

        // m = 12.0, t = 4.0, rho = 1.0
        let sigma = 2.0;
        let reduced_temperature = 4.0;
        let reduced_density = 1.0;
        let reduced_volume = moles[0] / reduced_density;

        let p = test_parameters(24.0, 6.0, 1.0, 1.0);
        let pt = ReferencePerturbationBH {
            parameters: Rc::new(p),
        };
        let state = StateHD::new(reduced_temperature, reduced_volume, moles.clone());
        let a = pt.helmholtz_energy(&state) / moles[0];
        assert_relative_eq!(a, -0.0611105573289734, epsilon = 1e-10);
    }
}