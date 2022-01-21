use super::hard_sphere_wca::{diameter_q_wca, diameter_wca};
use crate::parameters::*;
use feos_core::{HelmholtzEnergyDual, StateHD};
use ndarray::Array1;
use num_dual::DualNum;
use std::{f64::consts::PI, fmt, rc::Rc};

const C_WCA: [[f64; 6]; 6] = [
    [
        -0.2622378162,
        0.6585817423,
        5.5318022309,
        0.6902354794,
        -3.6825190645,
        -1.7263213318,
    ],
    [
        -0.1899241690,
        -0.5555205158,
        9.1361398949,
        0.7966155658,
        -6.1413017045,
        4.9553415149,
    ],
    [
        0.1169786415,
        -0.2216804790,
        -2.0470861617,
        -0.3742261343,
        0.9568416381,
        10.1401796764,
    ],
    [
        0.5852642702,
        2.0795520346,
        19.0711829725,
        -2.3403594600,
        2.5833371420,
        432.3858674425,
    ],
    [
        -0.6084232211,
        -7.2376034572,
        19.0412933614,
        3.2388986513,
        75.4442555789,
        -588.3837110653,
    ],
    [
        0.0512327656,
        6.6667943569,
        47.1109947616,
        -0.5011125797,
        -34.8918383146,
        189.5498636006,
    ],
];

/// Constants for WCA u-fraction.
const CU_WCA: [f64; 3] = [1.4419, 1.1169, 16.8810];

/// Constants for WCA effective inverse reduced temperature.
const C2: [[f64; 2]; 3] = [
    [1.45805207053190E-03, 3.57786067657446E-02],
    [1.25869266841313E-04, 1.79889086453277E-03],
    [0.0, 0.0],
];

#[derive(Debug, Clone)]
pub struct AttractivePerturbationWCA {
    pub parameters: Rc<UVParameters>,
}

impl fmt::Display for AttractivePerturbationWCA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Attractive Perturbation")
    }
}

impl<D: DualNum<f64>> HelmholtzEnergyDual<D> for AttractivePerturbationWCA {
    /// Helmholtz energy for attractive perturbation, eq. 52
    fn helmholtz_energy(&self, state: &StateHD<D>) -> D {
        let p = &self.parameters;
        let x = &state.molefracs;
        let t = state.temperature;
        let density = state.partial_density.sum();

        // vdw effective one fluid properties
        let (rep_x, att_x, sigma_x, weighted_sigma3_ij, epsilon_k_x, d_x, q_x) =
            one_fluid_properties(p, x, t);
        let t_x = state.temperature / epsilon_k_x;
        let rho_x = density * sigma_x.powi(3);
        let rm_x = (rep_x / att_x).powd((rep_x - att_x).recip());
        let mean_field_constant_x = mean_field_constant(rep_x, att_x, rm_x);

        let i_wca =
            correlation_integral_wca(rho_x, mean_field_constant_x, rep_x, att_x, d_x, q_x, rm_x);

        let delta_a1u = state.partial_density.sum() / t_x * i_wca * 2.0 * PI * weighted_sigma3_ij;

        //                 state.partial_density.sum() / t_x * i_wca * 2.0 * PI * weighted_sigma3_ij;
        let u_fraction_wca = u_fraction_wca(
            rep_x,
            density * (x * &p.sigma.mapv(|s| s.powi(3))).sum(),
            t_x.recip(),
        );

        let b21u = delta_b12u(t_x, mean_field_constant_x, weighted_sigma3_ij, q_x, rm_x);
        let b2bar = residual_virial_coefficient(p, x, state.temperature);

        state.moles.sum() * (delta_a1u + (-u_fraction_wca + 1.0) * (b2bar - b21u) * density)
    }
}

// (S43) & (S53)
fn delta_b12u<D: DualNum<f64>>(
    t_x: D,
    mean_field_constant_x: D,
    weighted_sigma3_ij: D,
    q_x: D,
    rm_x: D,
) -> D {
    (-mean_field_constant_x - (rm_x.powi(3) - q_x.powi(3)) * 1.0 / 3.0) / t_x
        * 2.0
        * PI
        * weighted_sigma3_ij
}

fn residual_virial_coefficient<D: DualNum<f64>>(p: &UVParameters, x: &Array1<D>, t: D) -> D {
    let mut delta_b2bar = D::zero();
    let q = diameter_q_wca(&p, t);
    for i in 0..p.ncomponents {
        let xi = x[i];
        for j in 0..p.ncomponents {
            let q_ij = (q[i] / p.sigma[i] + q[j] / p.sigma[j]) * 0.5;
            // Recheck mixing rule!
            delta_b2bar += xi
                * x[j]
                * p.sigma_ij[[i, j]].powi(3)
                * delta_b2(
                    t / p.eps_k_ij[[i, j]],
                    p.rep_ij[[i, j]],
                    p.att_ij[[i, j]],
                    q_ij,
                );
        }
    }
    delta_b2bar
}

fn correlation_integral_wca<D: DualNum<f64>>(
    rho_x: D,
    mean_field_constant_x: D,
    rep_x: D,
    att_x: D,
    d_x: D,
    q_x: D,
    rm_x: D,
) -> D {
    let c = coefficients_WCA(rep_x, att_x, d_x);
    (q_x.powi(3) - rm_x.powi(3)) * 1.0 / 3.0 - mean_field_constant_x
        + mie_prefactor(rep_x, att_x) * (c[0] * rho_x + c[1] * rho_x.powi(2) + c[2] * rho_x.powi(3))
            / (c[3] * rho_x + c[4] * rho_x.powi(2) + c[5] * rho_x.powi(3) + 1.0)
}

/// U-fraction according to Barker-Henderson division.
/// Eq. 15
fn u_fraction_wca<D: DualNum<f64>>(rep_x: D, reduced_density: D, one_fluid_beta: D) -> D {
    (reduced_density * CU_WCA[0]
        + reduced_density.powi(2) * (rep_x.recip() * CU_WCA[2] + CU_WCA[1]))
        .tanh()
}

fn one_fluid_properties<D: DualNum<f64>>(
    p: &UVParameters,
    x: &Array1<D>,
    t: D,
) -> (D, D, D, D, D, D, D) {
    let d = diameter_wca(p, t) / &p.sigma;
    let q = diameter_q_wca(p, t) / &p.sigma;
    let mut epsilon_k = D::zero();
    let mut weighted_sigma3_ij = D::zero();
    let mut rep = D::zero();
    let mut att = D::zero();
    for i in 0..p.ncomponents {
        let xi = x[i];
        for j in 0..p.ncomponents {
            let _y = xi * x[j] * p.sigma_ij[[i, j]].powi(3);
            weighted_sigma3_ij += _y;
            epsilon_k += _y * p.eps_k_ij[[i, j]];
            rep += xi * x[j] * p.rep_ij[[i, j]];
            att += xi * x[j] * p.att_ij[[i, j]];
        }
    }
    let dx = (x * &d.mapv(|v| v.powi(3))).sum().powf(1.0 / 3.0);
    let qx = (x * &q.mapv(|v| v.powi(3))).sum().powf(1.0 / 3.0);
    (
        rep,
        att,
        (x * &p.sigma.mapv(|v| v.powi(3))).sum().powf(1.0 / 3.0),
        weighted_sigma3_ij,
        epsilon_k / weighted_sigma3_ij,
        dx,
        qx,
    )
}
// Coefficients for IWCA from eq. (S55)
fn coefficients_WCA<D: DualNum<f64>>(rep: D, att: D, d: D) -> [D; 6] {
    let rep_inv = rep.recip();
    let rs_x = (rep / att).powd((rep - att).recip());
    let tau_x = -d + rs_x;
    let c1 = rep_inv.powi(2) * C_WCA[0][2]
        + C_WCA[0][0]
        + rep_inv * C_WCA[0][1]
        + (rep_inv.powi(2) * C_WCA[0][5] + rep_inv * C_WCA[0][4] + C_WCA[0][3]) * tau_x;
    let c2 = rep_inv.powi(2) * C_WCA[1][2]
        + C_WCA[1][0]
        + rep_inv * C_WCA[1][1]
        + (rep_inv.powi(2) * C_WCA[1][5] + rep_inv * C_WCA[1][4] + C_WCA[1][3]) * tau_x;
    let c3 = rep_inv.powi(2) * C_WCA[2][2]
        + C_WCA[2][0]
        + rep_inv * C_WCA[2][1]
        + (rep_inv.powi(2) * C_WCA[2][5] + rep_inv * C_WCA[2][4] + C_WCA[2][3]) * tau_x;
    let c4 = rep_inv.powi(2) * C_WCA[3][2]
        + C_WCA[3][0]
        + rep_inv * C_WCA[3][1]
        + (rep_inv.powi(2) * C_WCA[3][5] + rep_inv * C_WCA[3][4] + C_WCA[3][3]) * tau_x;
    let c5 = rep_inv.powi(2) * C_WCA[4][2]
        + C_WCA[4][0]
        + rep_inv * C_WCA[4][1]
        + (rep_inv.powi(2) * C_WCA[4][5] + rep_inv * C_WCA[4][4] + C_WCA[4][3]) * tau_x;
    let c6 = rep_inv.powi(2) * C_WCA[5][2]
        + C_WCA[5][0]
        + rep_inv * C_WCA[5][1]
        + (rep_inv.powi(2) * C_WCA[5][5] + rep_inv * C_WCA[5][4] + C_WCA[5][3]) * tau_x;

    [c1, c2, c3, c4, c5, c6]
}

fn delta_b2<D: DualNum<f64>>(reduced_temperature: D, rep: f64, att: f64, q: D) -> D {
    let rm = (rep / att).powf(1.0 / (rep - att)); // Check mixing rule!!
    let rc = 5.0;
    let alpha = mean_field_constant(rep, att, rc);
    let beta = reduced_temperature.recip();
    let y = beta.exp() - 1.0;
    let yeff = y_eff(reduced_temperature, rep, att);
    -(yeff * (rc.powi(3) - rm.powi(3)) / 3.0 + y * (-q.powi(3) + rm.powi(3)) / 3.0 + beta * alpha)
        * 2.0
        * PI
}

fn y_eff<D: DualNum<f64>>(reduced_temperature: D, rep: f64, att: f64) -> D {
    // optimize: move this part to parameter initialization
    let rc = 5.0;
    let rs = (rep / att).powf(1.0 / (rep - att));
    let c0 = 1.0
        - 3.0 * (mean_field_constant(rep, att, rs) - mean_field_constant(rep, att, rc))
            / (rc.powi(3) - rs.powi(3));
    let c1 = C2[0][0] + C2[0][1] / rep;
    let c2 = C2[1][0] + C2[1][1] / rep;
    let c3 = C2[2][0] + C2[2][1] / rep;

    //exponents
    let a = 1.05968091375869;
    let b = 3.41106168592999;
    let c = 0.0;
    // (S58)
    let beta = reduced_temperature.recip();
    let beta_eff = beta
        * (-(beta.powf(a) * c1 + beta.powf(b) * c2 + beta.powf(c) * c3 + 1.0).recip() * c0 + 1.0);
    beta_eff.exp() - 1.0
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parameters::utils::methane_parameters;
    use approx::assert_relative_eq;
    use ndarray::arr1;

    #[test]
    fn test_attractive_perturbation() {
        // m = 24, t = 4.0, rho = 1.0
        let moles = arr1(&[2.0]);
        let reduced_temperature = 4.0;
        let reduced_density = 1.0;
        let reduced_volume = moles[0] / reduced_density;

        let p = methane_parameters(24.0, 6.0);
        let pt = AttractivePerturbationWCA {
            parameters: Rc::new(p.clone()),
        };
        let state = StateHD::new(
            reduced_temperature * p.epsilon_k[0],
            reduced_volume * p.sigma[0].powi(3),
            moles.clone(),
        );
        let x = &state.molefracs;

        let (rep_x, att_x, sigma_x, weighted_sigma3_ij, epsilon_k_x, d_x, q_x) =
            one_fluid_properties(&p, &state.molefracs, state.temperature);
        let t_x = state.temperature / epsilon_k_x;
        let rho_x = state.partial_density.sum() * sigma_x.powi(3);
        let rm_x = (rep_x / att_x).powd((rep_x - att_x).recip());
        let mean_field_constant_x = mean_field_constant(rep_x, att_x, rm_x);
        dbg!(q_x);
        dbg!(rm_x);
        let b21u = delta_b12u(t_x, mean_field_constant_x, weighted_sigma3_ij, q_x, rm_x)
            / p.sigma[0].powi(3);
        //assert!(b21u.re() == -1.02233216);
        assert_relative_eq!(b21u.re(), -1.02233215790525, epsilon = 1e-12);

        let i_wca =
            correlation_integral_wca(rho_x, mean_field_constant_x, rep_x, att_x, d_x, q_x, rm_x);
        dbg!(i_wca);
        let delta_a1u = state.partial_density.sum() / t_x * i_wca * 2.0 * PI * weighted_sigma3_ij;

        //dbg!(delta_a1u);
        //assert!(delta_a1u.re() == -1.1470186919354);
        assert_relative_eq!(delta_a1u.re(), -1.52406840346272, epsilon = 1e-6);

        let u_fraction_wca = u_fraction_wca(
            rep_x,
            state.partial_density.sum() * (x * &p.sigma.mapv(|s| s.powi(3))).sum(),
            t_x.recip(),
        );

        let b2bar = residual_virial_coefficient(&p, x, state.temperature) / p.sigma[0].powi(3);
        dbg!(b2bar);
        assert_relative_eq!(b2bar.re(), -1.09102560732964, epsilon = 1e-12);
        dbg!(u_fraction_wca);
        //assert!(u_fraction_WCA.re() == 0.743451055308332);
        assert_relative_eq!(u_fraction_wca.re(), 0.997069754340431, epsilon = 1e-5);

        //assert!(b2bar.re() ==-1.00533412744652);

        let a_test = delta_a1u
            + (-u_fraction_wca + 1.0)
                * (b2bar - b21u)
                * p.sigma[0].powi(3)
                * state.partial_density.sum();
        dbg!(a_test);
        dbg!(state.moles.sum());
        let a = pt.helmholtz_energy(&state) / moles[0];
        dbg!(a.re());
        //assert!(-1.16124062615291 == a.re())
        assert_relative_eq!(-1.5242697155023, a.re(), epsilon = 1e-5);
    }
}