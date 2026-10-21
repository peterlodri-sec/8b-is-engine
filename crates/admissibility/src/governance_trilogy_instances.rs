//! governance_trilogy_instances — Executable witnesses for Flyxion's October 2026
//! Governance, Autonomy & Bounded-Charity Trilogy:
//!
//! 1. *Continuous Microsurgery: The Ethics of Never Putting the Knife Down* (`CMS`)
//!    — Hepastitial redundancy vs. common-cause ceiling (`Prop. 1`), Umbilicus mass
//!    balance & atrophy trap (`Prop. 2`), Skeletal strain & asymmetric return (`§3.3`),
//!    Micro-repair bot damage floor (`§3.4`), Nerode trajectory equivalence vs.
//!    local-rule inadequacy (`Prop. 3`, `CT-000⁺` / `CT-015`), Competing risks
//!    cumulative incidence (`§5.3`), Concave option price of lock-in (`Prop. 5`),
//!    Greedy over-authorization ratchet (`Prop. 6`), Damage-balance grace period &
//!    alternation duty cycle (`Prop. 7`, `§12.3`), Portability/independence/rebuild
//!    horizons (`Prop. 8`), Endogenous preferences & veto bound (`Prop. 9`),
//!    Closed-loop trait control (`Prop. 10`), Granovetter threshold pressure without
//!    a coercer (`Prop. 11`), Autonomy enlargement inclusion (`Prop. 12`), and
//!    Independence of `(P1, P2, P3)` & Governability `(E, M, C)` (`Props. 13–14`).
//! 2. *The Surveillance State After Scarcity: Information Integration, Institutional
//!    Limits, and the Mathematics of Ubiquitous Observation* (`SAS`)
//!    — Observational equivalence non-identifiability (`Prop. 1`), Retention risk
//!    monotonicity (`Prop. 2`), Defeasible authority ceiling & warrant revocation
//!    (`Prop. 3`), Recorded dependency graph `Reach_G(r_0)` vs. unrecorded
//!    over-inclusion (`Props. 4–5`), Privacy loss vs. auxiliary information (`Prop. 6`),
//!    Incompatibility of subject erasure and exact correction vs. pseudonymous
//!    dependency tokens (`Prop. 7`, `CT-000⁺`), and Reversibility-scaled evidence
//!    thresholds (`Prop. 8`).
//! 3. *The Charity Trap: Trust, Attribution, Emotional Dysregulation, and the Hidden
//!    Costs of Explaining Other People's Behavior* (`TCT`)
//!    — Possibility is not justification (`Prop. 1`, Table 1 `CT-000⁺`), Silent
//!    threshold of default trust & Beta goodwill exchange rate (`Prop. 2`, `Cor. 1`),
//!    Excuse inflation: geometric updating under disciplined charity vs. non-updating
//!    post-selection (`Prop. 3`), Cause-equivalence of conduct-based decisions
//!    (`Prop. 4`), and The Ratchet of Accommodation submartingale bound vs. calm-period
//!    cap (`Prop. 5`, `Cor. 2`).

use std::collections::{HashMap, HashSet, VecDeque};

use crate::Structure;

// ============================================================================
// I. CONTINUOUS MICROSURGERY (CMS)
// ============================================================================

/// Unmaintained viability horizon `t_ε(N) = (1 / μ) * ln(1 / (1 - ε^{1/N}))`
/// for a `k = 1` hepastitial network of `N` independent nodes (`CMS` Prop. 1(a)).
pub fn hepastitial_unmaintained_horizon_days(n: u32, mu_per_day: f64, eps: f64) -> f64 {
    assert!(n >= 1 && mu_per_day > 0.0 && eps > 0.0 && eps < 1.0);
    let eps_root = eps.powf(1.0 / (n as f64));
    (1.0 / mu_per_day) * (1.0 / (1.0 - eps_root)).ln()
}

/// Common-cause failure ceiling `t_ε^{cc} = (1 / μ_c) * ln(1 / (1 - ε))`
/// bounding viability for every `N` and `k` (`CMS` Prop. 1(b)).
pub fn hepastitial_common_cause_ceiling_days(mu_c_per_day: f64, eps: f64) -> f64 {
    assert!(mu_c_per_day > 0.0 && eps > 0.0 && eps < 1.0);
    (1.0 / mu_c_per_day) * (1.0 / (1.0 - eps)).ln()
}

/// Steady-state native clearance/repair capacity `ρ_ss(u)` under delegated share
/// `u ∈ [0, 1]` (`CMS` Eq. 16):
/// `ρ_ss(u) = ρ_max - (ρ_max - ρ_min) * (α u) / (β (1 - u) + α u)`.
pub fn native_capacity_steady_state(
    u: f64,
    rho_min: f64,
    rho_max: f64,
    alpha: f64,
    beta: f64,
) -> f64 {
    assert!((0.0..=1.0).contains(&u));
    assert!(rho_max >= rho_min && alpha > 0.0 && beta > 0.0);
    let denom = beta * (1.0 - u) + alpha * u;
    let w = (alpha * u) / denom;
    rho_max - (rho_max - rho_min) * w
}

/// Umbilicus mass-balance margin `g(θ) = θ J_max + ε ρ_ss(s θ) - L` (`CMS` Eq. 3)
/// and derivative at the origin `g'(0) = J_max - s ε (ρ_max - ρ_min) α / β` (`CMS` Prop. 2).
pub fn umbilicus_mass_balance(
    theta: f64,
    s_delegated: f64,
    j_max: f64,
    eps_clearance: f64,
    load_l: f64,
    rho_min: f64,
    rho_max: f64,
    alpha: f64,
    beta: f64,
) -> (f64, f64) {
    let rho = native_capacity_steady_state(s_delegated * theta, rho_min, rho_max, alpha, beta);
    let g_theta = theta * j_max + eps_clearance * rho - load_l;
    let g_prime_0 = j_max - s_delegated * eps_clearance * (rho_max - rho_min) * (alpha / beta);
    (g_theta, g_prime_0)
}

/// Feasible skeletal lengthening rate `r ≤ min(r_bone, min_i ε̇_i L_i)` (`CMS` Eq. 4)
/// and return-to-forward time ratio `r_+ / r_-` (`CMS` §3.3).
pub fn skeletal_feasible_rate_and_return_ratio(
    r_bone_mm_per_day: f64,
    segments_rest_mm_and_tolerable_strain: &[(f64, f64)],
    r_plus_mm_per_day: f64,
    r_minus_mm_per_day: f64,
) -> (f64, f64) {
    let min_soft = segments_rest_mm_and_tolerable_strain
        .iter()
        .map(|&(l_i, eps_dot_i)| l_i * eps_dot_i)
        .fold(r_bone_mm_per_day, f64::min);
    let r_max = r_bone_mm_per_day.min(min_soft);
    let return_ratio = r_plus_mm_per_day / r_minus_mm_per_day;
    (r_max, return_ratio)
}

/// Micro-repair bot useful damage floor `D_floor = h K / (r - h)` and required
/// bot population `n(D̄)` to hold target damage `D̄ > D_floor` (`CMS` Eqs. 5–6).
pub fn micro_repair_floor_and_population(
    r_max: f64,
    h_collateral: f64,
    k_half: f64,
    g0: f64,
    g1: f64,
    rho: f64,
    d_target: f64,
) -> (f64, Option<f64>) {
    assert!(r_max > h_collateral && h_collateral >= 0.0 && k_half > 0.0);
    let d_floor = (h_collateral * k_half) / (r_max - h_collateral);
    let deficit = g0 + (g1 - rho) * d_target;
    let net_per_bot = (r_max * d_target) / (k_half + d_target) - h_collateral;
    let pop = if d_target > d_floor && deficit > 0.0 && net_per_bot > 0.0 {
        Some(deficit / net_per_bot)
    } else {
        None
    };
    (d_floor, pop)
}

/// Actions in a longitudinal trajectory subject to a history-dependent exit-cost
/// or cumulative-exposure cap (`CMS` §4.2, Proposition 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrajectoryAction {
    /// Dependency-increasing microsurgical step `b_intervene` (admissible only
    /// when the cumulative exposure / exit-cost cap has not yet been reached).
    InterveneStep,
    /// Null or restoring maintenance step `b_hold_or_restore` (admissible only
    /// when the trajectory is at the cap and must not exceed it).
    HoldOrRestoreOnly,
}

/// `CMS` Proposition 3 (`CT-000⁺` / `CT-015`):
/// Two histories `h` (0 dependency steps taken) and `h'` (`N` dependency steps
/// taken, at the cap) end in the exact same current physiological state `x`.
/// A local rule `σ: X -> 2^A` sees only `x`, so the two histories fall into the
/// same observation class with disjoint admissible continuation obligations.
pub fn cms_local_rule_nerode_collision() -> Structure<&'static str, TrajectoryAction> {
    let obs = vec!["physiological_state_x", "physiological_state_x"];
    let adm = vec![
        HashSet::from([TrajectoryAction::InterveneStep]),
        HashSet::from([TrajectoryAction::HoldOrRestoreOnly]),
    ];
    Structure::new(obs, adm)
}

/// `CMS` Proposition 3 (Converse):
/// Augmenting the current physiological state `x` with the Nerode equivalence
/// class `[h]_C` of the history separates the two prefixes using the minimal
/// number of memory states.
pub fn cms_nerode_augmented_structure() -> Structure<(&'static str, &'static str), TrajectoryAction>
{
    let obs = vec![
        ("physiological_state_x", "nerode_class_below_cap"),
        ("physiological_state_x", "nerode_class_at_cap"),
    ];
    let adm = vec![
        HashSet::from([TrajectoryAction::InterveneStep]),
        HashSet::from([TrajectoryAction::HoldOrRestoreOnly]),
    ];
    Structure::new(obs, adm)
}

/// Competing risks cumulative incidence `F_1(t) = (h_1 / (h_1 + h_2)) * (1 - e^{-(h_1+h_2)t})`
/// versus the naive Kaplan-Meier complement `1 - e^{-h_1 t}` (`CMS` §5.3, Eq. 10).
pub fn competing_risks_cif_vs_km(h1: f64, h2: f64, t: f64) -> (f64, f64) {
    let total = h1 + h2;
    let cif = (h1 / total) * (1.0 - (-total * t).exp());
    let km_naive = 1.0 - (-h1 * t).exp();
    (cif, km_naive)
}

/// Price of lock-in `Π(κ) = m e^{-c_0/m} (1 - e^{-κ/m})` and marginal price
/// `Π'(κ) = S(c_0 + κ) = e^{-(c_0+κ)/m}` for exponential discontinuation benefit
/// `d ~ Exp(1/m)` (`CMS` Proposition 5 & Example 3).
pub fn exponential_lock_in_option_price(m: f64, c0: f64, kappa: f64) -> (f64, f64, f64) {
    assert!(m > 0.0 && c0 >= 0.0 && kappa >= 0.0);
    let w0 = m * (-c0 / m).exp();
    let pi_kappa = w0 * (1.0 - (-kappa / m).exp());
    let marginal = (-(c0 + kappa) / m).exp();
    (w0, pi_kappa, marginal)
}

/// Greedy over-authorization ratchet (`CMS` Proposition 6 & Example 4):
/// Step `k` (`k = 1..=n`) has benefit `B`, own medical risk `R_k = r / (1 + k)`,
/// and dependence burden `D_k = λ k`. Returns `(u_k, N(n), authorized_by_local_rule)`.
pub fn greedy_ratchet_trajectory(b: f64, r: f64, lambda: f64, n: usize) -> (Vec<f64>, f64, bool) {
    let mut increments = Vec::with_capacity(n);
    let mut h_sum = 0.0;
    let mut all_authorized = true;
    for k in 1..=n {
        let r_k = r / (1.0 + k as f64);
        if r_k > b {
            all_authorized = false;
        }
        let u_k = b - r_k - lambda * (k as f64);
        increments.push(u_k);
        h_sum += 1.0 / (1.0 + k as f64);
    }
    let nf = n as f64;
    let net_n = nf * b - r * h_sum - 0.5 * lambda * nf * (nf + 1.0);
    (increments, net_n, all_authorized)
}

/// Grace period `T(a)` after support is lost at held damage `D̄ < D_m` with net
/// endogenous clearance `a = ρ - g_1` (`CMS` Proposition 7).
/// Returns `None` when `a >= g_0 / D_m` (`T(a) = ∞`, physiologically independent).
pub fn grace_period_days(g0: f64, a: f64, d_bar: f64, d_m: f64) -> Option<f64> {
    assert!(g0 > 0.0 && d_bar > 0.0 && d_m > d_bar);
    if a >= g0 / d_m {
        return None;
    }
    if a.abs() < 1e-12 {
        Some((d_m - d_bar) / g0)
    } else {
        Some(((g0 - a * d_bar) / (g0 - a * d_m)).ln() / a)
    }
}

/// Engaged maintenance load `R_on(θ) = max(0, g_0 + (g_1 - ρ_ss(s θ)) D̄) / θ`
/// (`CMS` Eq. 17) and planned-withdrawal biological rebuild time
/// `τ_bio(t) = (1 / β) ln((ρ_max - ρ(t)) / (ρ_max - ρ*))` (`CMS` Proposition 8(c)).
pub fn alternation_load_and_rebuild_days(
    theta: f64,
    s_delegated: f64,
    rho_min: f64,
    rho_max: f64,
    alpha: f64,
    beta: f64,
    g0: f64,
    g1: f64,
    d_bar: f64,
    d_m: f64,
) -> (f64, Option<f64>, f64, Option<f64>) {
    let rho_ss = native_capacity_steady_state(s_delegated * theta, rho_min, rho_max, alpha, beta);
    let a = rho_ss - g1;
    let grace = grace_period_days(g0, a, d_bar, d_m);
    let r_on = if theta <= 0.0 {
        0.0
    } else {
        (g0 + (g1 - rho_ss) * d_bar).max(0.0) / theta
    };
    let rho_star = g1 + g0 / d_m;
    let tau_bio = if rho_ss >= rho_star {
        Some(0.0)
    } else if rho_max > rho_star {
        Some(((rho_max - rho_ss) / (rho_max - rho_star)).ln() / beta)
    } else {
        None
    };
    (rho_ss, grace, r_on, tau_bio)
}

/// Endogenous preference shift (`CMS` Proposition 9 & Example 5):
/// Returns `(k_star_unanimity, initial_self_utilities_u0_to_un)`.
pub fn endogenous_preference_unanimity_steps(
    delta0: f64,
    delta_shift: f64,
    max_steps: usize,
) -> (usize, Vec<f64>) {
    assert!(delta_shift > 1.0 && delta0 > 0.5);
    let k_star = (delta0 - 0.5).ceil() as usize;
    let u_initial = (0..=max_steps)
        .map(|n| {
            let diff = (n as f64) - delta0;
            -(diff * diff)
        })
        .collect();
    (k_star, u_initial)
}

/// Closed-loop trait autoregression under external target `g` (`CMS` Proposition 10):
/// Returns `(carry_over_coeff_n, stationary_mean, stationary_variance)`.
pub fn closed_loop_trait_dynamics(
    a: f64,
    b: f64,
    k: f64,
    g: f64,
    sigma_sq: f64,
    n: u32,
) -> (f64, f64, f64) {
    let phi = a - b * k;
    assert!(phi.abs() < 1.0, "|a - b*k| must be < 1 for stability");
    let carry = phi.powi(n as i32);
    let mean = (b * k * g) / (1.0 - a + b * k);
    let var = sigma_sq / (1.0 - phi * phi);
    (carry, mean, var)
}

/// Standard normal CDF approximation (Abramowitz & Stegun 7.1.26, error `< 1.5e-7`).
fn std_normal_cdf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let z = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.3275911 * z);
    let poly = t
        * (0.254829592
            + t * (-0.284496736
                + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    let erf = sign * (1.0 - poly * (-z * z).exp());
    0.5 * (1.0 + erf)
}

/// Granovetter threshold equilibrium `f*(s)` for `v ~ N(mean, std^2)` (`CMS` Proposition 11 & Example 6):
/// Iterates `f^{(j+1)} = 1 - F(-s f^{(j)})` from `f^{(0)} = f_0 = 1 - F(0)`.
/// Returns `(f0, f_star, excess)`.
pub fn granovetter_threshold_equilibrium(mean: f64, std: f64, s: f64) -> (f64, f64, f64) {
    let cdf = |v: f64| std_normal_cdf((v - mean) / std);
    let f0 = 1.0 - cdf(0.0);
    let mut f = f0;
    for _ in 0..500 {
        let next = 1.0 - cdf(-s * f);
        if (next - f).abs() < 1e-12 {
            f = next;
            break;
        }
        f = next;
    }
    (f0, f, f - f0)
}

// ============================================================================
// II. THE SURVEILLANCE STATE AFTER SCARCITY (SAS)
// ============================================================================

/// Candidate institutional actions in the municipal warrant-flag deployment (`SAS` §6.9–6.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstitutionalAction {
    /// Low-loss, reversible administrative review (`ℓ_rev ≤ θ / p`).
    ReversibleAdminReview,
    /// High-loss, irreversible detention or deprivation (`ℓ_det = k * ℓ_rev > θ / p`).
    IrreversibleDetention,
    /// Withhold coercive action when person is not the warrant subject.
    ReleaseUnflagged,
}

/// `SAS` Propositions 1 & 8(b) (`CT-000⁺`):
/// Two trajectories `x ∈ W_a` (flagged person is NOT the warrant subject;
/// `IrreversibleDetention` is wrong with loss `ℓ_det > θ / p`) and `x' ∉ W_a`
/// (flagged person IS the warrant subject) are observationally equivalent under
/// a coarse sensor suite (`obs[0] == obs[1]`). When the decision space forces
/// choosing between `IrreversibleDetention` (required in `x'`) and `ReleaseUnflagged`
/// (required in `x`), no volume of retained history `H_T` can make the action
/// admissible (`no_safe_action_witness() == Some((0, 1))`).
pub fn sas_observational_equivalence_irreversible_collision(
) -> Structure<&'static str, InstitutionalAction> {
    let obs = vec!["coarse_biometric_name_match", "coarse_biometric_name_match"];
    let adm = vec![
        HashSet::from([InstitutionalAction::ReleaseUnflagged]),
        HashSet::from([InstitutionalAction::IrreversibleDetention]),
    ];
    Structure::new(obs, adm)
}

/// `SAS` Propositions 1 & 8(a):
/// Either restricting the action under coarse observations to `ReversibleAdminReview`
/// (where `E[L(a, X) | H_T] = p * ℓ_rev ≤ θ` holds across the equivalence class)
/// or adding a discriminating cryptographic/sensor modality resolves the collision.
pub fn sas_reversibility_scaled_or_refined_structure(
    discriminating_sensor: bool,
) -> Structure<&'static str, InstitutionalAction> {
    if discriminating_sensor {
        let obs = vec![
            "verified_non_match_exoneration",
            "verified_warrant_subject_match",
        ];
        let adm = vec![
            HashSet::from([InstitutionalAction::ReleaseUnflagged]),
            HashSet::from([InstitutionalAction::IrreversibleDetention]),
        ];
        Structure::new(obs, adm)
    } else {
        let obs = vec!["coarse_biometric_name_match", "coarse_biometric_name_match"];
        let adm = vec![
            HashSet::from([
                InstitutionalAction::ReversibleAdminReview,
                InstitutionalAction::ReleaseUnflagged,
            ]),
            HashSet::from([
                InstitutionalAction::ReversibleAdminReview,
                InstitutionalAction::IrreversibleDetention,
            ]),
        ];
        Structure::new(obs, adm)
    }
}

/// Fellegi-Sunter base-rate posterior `P(true match | flagged) = s π / (s π + α (1 - π))`
/// (`SAS` §3.1 & §6.10) and maximum tolerable posterior error `θ / ℓ_a` (`SAS` Prop. 8(a)).
pub fn record_linkage_posterior_and_threshold(
    prevalence_pi: f64,
    sensitivity_s: f64,
    false_match_alpha: f64,
    risk_tolerance_theta: f64,
    action_loss_ell: f64,
) -> (f64, f64, bool) {
    let tp = sensitivity_s * prevalence_pi;
    let fp = false_match_alpha * (1.0 - prevalence_pi);
    let posterior_true_match = tp / (tp + fp);
    let posterior_wrong_qa = 1.0 - posterior_true_match;
    let max_error = risk_tolerance_theta / action_loss_ell;
    (posterior_true_match, max_error, posterior_wrong_qa <= max_error)
}

/// Exact BFS reachability `Reach_G(r_0)` in a recorded dependency graph `Ĝ` (`SAS` Proposition 4).
pub fn dependency_reach_set(edges: &[(usize, usize)], r0: usize) -> Vec<usize> {
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(u, v) in edges {
        adj.entry(u).or_default().push(v);
    }
    let mut visited = HashSet::new();
    let mut q = VecDeque::new();
    visited.insert(r0);
    q.push_back(r0);
    while let Some(u) = q.pop_front() {
        if let Some(nbrs) = adj.get(&u) {
            for &v in nbrs {
                if visited.insert(v) {
                    q.push_back(v);
                }
            }
        }
    }
    let mut out: Vec<usize> = visited.into_iter().collect();
    out.sort_unstable();
    out
}

/// `SAS` Proposition 7 (`CT-000⁺`):
/// In the 2-subject swap (`Λ: r_1 -> s_1, r_2 -> s_2` vs. `Λ': r_1 -> s_2, r_2 -> s_1`),
/// `Λ ~ Λ'` differ only in subject-identifying content, so irreversible erasure
/// forces `e(Λ) == e(Λ')`. Yet exact subject-specific correction for `s_1` requires
/// revising `{d_1}` in `Λ` and `{d_2}` in `Λ'`, producing a `CT-000⁺` collision.
pub fn sas_erasure_vs_subject_correction_collision() -> Structure<&'static str, &'static str> {
    let obs = vec![
        "erased_state_e_lambda_anonymized",
        "erased_state_e_lambda_anonymized",
    ];
    let adm = vec![
        HashSet::from(["revise_decision_d1_for_s1"]),
        HashSet::from(["revise_decision_d2_for_s1"]),
    ];
    Structure::new(obs, adm)
}

/// `SAS` Proposition 7 (Pseudonymous Dependency Token Resolution):
/// When dependency edges and correction queries are keyed by a pseudonymous
/// dependency token `τ_1` held outside the erasure boundary, `D_{τ_1}(Λ) == D_{τ_1}(Λ') == {d_1}`
/// is invariant across erasure-equivalent histories `Λ ~ Λ'`.
pub fn sas_pseudonymous_token_correction_structure() -> Structure<&'static str, &'static str> {
    let obs = vec![
        "erased_state_with_pseudonymous_token_tau1",
        "erased_state_with_pseudonymous_token_tau1",
    ];
    let adm = vec![
        HashSet::from(["revise_decision_d1_via_token_tau1"]),
        HashSet::from(["revise_decision_d1_via_token_tau1"]),
    ];
    Structure::new(obs, adm)
}

// ============================================================================
// III. THE CHARITY TRAP (TCT)
// ============================================================================

/// Observer responses to a surprising, costly interpersonal act (`TCT` §4, §7, §11–12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObserverResponse {
    /// Absorb the cost unconditionally and ratify the accusation/demand (`J = 1`).
    AbsorbUnconditionally,
    /// Validate the feeling, state the factual record plainly, and hold a calm
    /// conduct-based boundary (`TCT` §11–12, Proposition 4).
    ValidateAndHoldBoundary,
    /// Enforce a pre-announced calm boundary without absorbing the unjustified cost (`J = 0`).
    EnforcePreannouncedLimit,
}

/// `TCT` Proposition 1 & Table 1 (`CT-000⁺`):
/// Under a surprising act `a`, two hidden circumstances `c_1` (justifying, `J(a, c_1) = 1`)
/// and `c_2` (unjustified, `J(a, c_2) = 0`) produce the same observed act `a`.
/// An observer who collapses Intelligibility/Sincerity into Justification (`adm[0] = {Absorb}`,
/// `adm[1] = {EnforcePreannouncedLimit}`) faces a `CT-000⁺` collision (`no_safe_action_witness() == Some((0, 1))`),
/// and the posterior `P(J = 1 | a, H) = ε` can be arbitrarily small (`Prop. 1`).
pub fn tct_collapsed_attribution_collision() -> Structure<&'static str, ObserverResponse> {
    let obs = vec!["surprising_costly_act_a", "surprising_costly_act_a"];
    let adm = vec![
        HashSet::from([ObserverResponse::AbsorbUnconditionally]),
        HashSet::from([ObserverResponse::EnforcePreannouncedLimit]),
    ];
    Structure::new(obs, adm)
}

/// `TCT` Proposition 4 (Cause-Equivalence of Conduct-Based Decisions) & §11–12:
/// When two causal hypotheses `M_1, M_2` induce the same predictive law `F_1 = F_2`
/// for future conduct `X` and the observer's policy separates the four questions of
/// Table 1 (validating the feeling while declining false facts and holding a
/// pre-announced conduct limit), `ValidateAndHoldBoundary` is jointly admissible
/// across the entire observation class (`no_safe_action_witness() == None`).
pub fn tct_conduct_based_bounded_charity_structure() -> Structure<&'static str, ObserverResponse> {
    let obs = vec!["surprising_costly_act_a", "surprising_costly_act_a"];
    let adm = vec![
        HashSet::from([
            ObserverResponse::ValidateAndHoldBoundary,
            ObserverResponse::AbsorbUnconditionally,
        ]),
        HashSet::from([
            ObserverResponse::ValidateAndHoldBoundary,
            ObserverResponse::EnforcePreannouncedLimit,
        ]),
    ];
    Structure::new(obs, adm)
}

/// Posterior probability `P(J = 1 | a, H)` under `TCT` Proposition 1 when
/// `c_1` (`J = 1`, prior `ε`) and `c_2` (`J = 0`, prior `1 - ε`) have likelihoods
/// `l1 = P(a | c_1, H)` and `l2 = P(a | c_2, H)`.
pub fn justification_posterior(eps_prior: f64, l1: f64, l2: f64) -> f64 {
    let num = eps_prior * l1;
    let den = num + (1.0 - eps_prior) * l2;
    assert!(den > 0.0);
    num / den
}

/// Silent threshold of default trust `p* = 1 - v / L` (`TCT` Proposition 2) and
/// Beta(α, β) goodwill threshold `n*(m) = (v(α + m) - (L - v)β) / (L - v)` with
/// exchange rate slope `v / (L - v)` (`TCT` Corollary 1).
pub fn silent_trust_threshold_and_goodwill(
    v_verify_cost: f64,
    l_loss: f64,
    alpha: f64,
    beta: f64,
    m_honored: u32,
) -> (f64, f64, f64) {
    assert!(v_verify_cost > 0.0 && l_loss > v_verify_cost);
    let p_star = 1.0 - v_verify_cost / l_loss;
    let exchange_slope = v_verify_cost / (l_loss - v_verify_cost);
    let n_star = (v_verify_cost * (alpha + m_honored as f64)
        - (l_loss - v_verify_cost) * beta)
        / (l_loss - v_verify_cost);
    (p_star, exchange_slope, n_star)
}

/// Excuse inflation (`TCT` Proposition 3 & Table 2):
/// Compares disciplined charity (`q_t / u_t = ρ < 1` fixed in advance) against
/// undisciplined post-selected charity (`q̃_t / u_t = r ≥ 1` chosen after seeing `v_t`).
/// Returns `(omega_disciplined_n, omega_undisciplined_n, steps_to_reach_threshold_omega)`.
pub fn excuse_inflation_comparison(
    omega0: f64,
    rho_disciplined: f64,
    r_undisciplined: f64,
    n_events: u32,
    threshold_omega: f64,
) -> (f64, f64, u32) {
    assert!(omega0 > 0.0 && rho_disciplined > 0.0 && rho_disciplined < 1.0 && r_undisciplined >= 1.0);
    let om_disc = omega0 * rho_disciplined.powi(n_events as i32);
    let om_undisc = omega0 * r_undisciplined.powi(n_events as i32);
    let steps = if omega0 <= threshold_omega {
        0
    } else {
        ((omega0 / threshold_omega).ln() / (1.0 / rho_disciplined).ln()).ceil() as u32
    };
    (om_disc, om_undisc, steps)
}

/// Ratchet of Accommodation (`TCT` Proposition 5 & Corollary 2):
/// Returns the submartingale upper bound `(C + m - E_0) / (p * δ)` on expected
/// capacity-exhaustion time `E[T]` under yielding (`A_t = D_t`), and simulates a
/// demand trajectory under both yielding (`A_t = D_t`) and a calm-period cap
/// (`A_t = min(D_t, a_star)` with `a_star < C`).
pub fn accommodation_ratchet_bound_and_trace(
    e0: f64,
    capacity_c: f64,
    m_max_jump: f64,
    p_exceed: f64,
    delta_step: f64,
    a_star_cap: f64,
    demand_increments: &[f64],
) -> (f64, Option<usize>, Vec<f64>, Vec<f64>) {
    assert!(capacity_c > e0 && a_star_cap >= e0 && a_star_cap < capacity_c);
    assert!(m_max_jump >= delta_step && delta_step > 0.0 && p_exceed > 0.0);
    let expected_t_bound = (capacity_c + m_max_jump - e0) / (p_exceed * delta_step);

    let mut e_yield = e0;
    let mut e_capped = e0;
    let mut yield_trace = Vec::with_capacity(demand_increments.len());
    let mut capped_trace = Vec::with_capacity(demand_increments.len());
    let mut exhaustion_step = None;

    for (idx, &inc) in demand_increments.iter().enumerate() {
        let clamped_inc = inc.clamp(0.0, m_max_jump);
        let d_t_yield = e_yield + clamped_inc;
        e_yield = e_yield.max(d_t_yield);
        yield_trace.push(e_yield);
        if exhaustion_step.is_none() && e_yield >= capacity_c {
            exhaustion_step = Some(idx + 1);
        }

        let d_t_cap = e_capped + clamped_inc;
        let a_t_cap = d_t_cap.min(a_star_cap);
        e_capped = e_capped.max(a_t_cap);
        capped_trace.push(e_capped);
    }

    (expected_t_bound, exhaustion_step, yield_trace, capped_trace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cms_subsystems_hepastitium_umbilicus_skeleton_and_repair_bots() {
        // CMS Example 1: μ = 0.01/day, ε = 1e-3, N = 10, 100, 1000 -> 69.6, 270.7, 497.9 days
        let t10 = hepastitial_unmaintained_horizon_days(10, 0.01, 1e-3);
        let t100 = hepastitial_unmaintained_horizon_days(100, 0.01, 1e-3);
        let t1000 = hepastitial_unmaintained_horizon_days(1000, 0.01, 1e-3);
        assert!((t10 - 69.56).abs() < 0.05);
        assert!((t100 - 270.67).abs() < 0.05);
        assert!((t1000 - 497.87).abs() < 0.05);
        let t_cc = hepastitial_common_cause_ceiling_days(0.001, 1e-3);
        assert!((t_cc - 1.0005).abs() < 1e-3);

        // CMS Example 2: Umbilicus atrophy trap (ρ_max=0.20, ρ_min=0.02, α=0.04, β=0.02, ε=1, L=0.25, J_max=0.30)
        let (g0, gp0) = umbilicus_mass_balance(0.0, 1.0, 0.30, 1.0, 0.25, 0.02, 0.20, 0.04, 0.02);
        assert!((g0 - (-0.05)).abs() < 1e-12);
        assert!((gp0 - (-0.06)).abs() < 1e-12); // g'(0) = 0.30 - 0.36 = -0.06 < 0
        let (g01, _) = umbilicus_mass_balance(0.1, 1.0, 0.30, 1.0, 0.25, 0.02, 0.20, 0.04, 0.02);
        assert!((g01 - (-0.052_727_273)).abs() < 1e-6);
        assert!(g01 < g0); // Light engagement worsens mass balance!
        let (g063, _) = umbilicus_mass_balance(0.630_602, 1.0, 0.30, 1.0, 0.25, 0.02, 0.20, 0.04, 0.02);
        assert!(g063.abs() < 1e-4);
        let (g10, _) = umbilicus_mass_balance(1.0, 1.0, 0.30, 1.0, 0.25, 0.02, 0.20, 0.04, 0.02);
        assert!((g10 - 0.07).abs() < 1e-12);

        // CMS §3.3: Skeletal strain rate at 300 mm & asymmetric return
        let (r_max, ratio) =
            skeletal_feasible_rate_and_return_ratio(3.0, &[(300.0, 0.008)], 2.4, 1.0);
        assert!((r_max - 2.4).abs() < 1e-12);
        assert!((ratio - 2.4).abs() < 1e-12);

        // CMS §3.4: Micro-repair bots D_floor = 1.25 for r=1, h=0.2, K=5
        let (d_floor, pop_above) =
            micro_repair_floor_and_population(1.0, 0.2, 5.0, 1.0, 0.05, 0.10, 2.0);
        assert!((d_floor - 1.25).abs() < 1e-12);
        assert!(pop_above.unwrap() > 0.0);
        let (_, pop_below) =
            micro_repair_floor_and_population(1.0, 0.2, 5.0, 1.0, 0.05, 0.10, 1.0);
        assert!(pop_below.is_none());
    }

    #[test]
    fn cms_nerode_local_rule_collision_and_history_augmentation() {
        let local = cms_local_rule_nerode_collision();
        local.assert_ct000();
        assert_eq!(local.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(local.guaranteeing_policy(), None);

        let augmented = cms_nerode_augmented_structure();
        augmented.assert_ct000();
        assert_eq!(augmented.no_safe_action_witness(), None);
        assert_eq!(
            augmented.guaranteeing_policy(),
            Some(vec![
                (
                    ("physiological_state_x", "nerode_class_below_cap"),
                    TrajectoryAction::InterveneStep
                ),
                (
                    ("physiological_state_x", "nerode_class_at_cap"),
                    TrajectoryAction::HoldOrRestoreOnly
                ),
            ])
        );
    }

    #[test]
    fn cms_competing_risks_option_price_greedy_ratchet_and_worked_example() {
        // CMS §5.3: h1 = 0.10, h2 = 0.05, t = 5 -> CIF = 0.3518 vs KM = 0.3935
        let (cif, km) = competing_risks_cif_vs_km(0.10, 0.05, 5.0);
        assert!((cif - 0.351_755).abs() < 1e-5);
        assert!((km - 0.393_469).abs() < 1e-5);
        assert!(cif < km);

        // CMS Example 3: κ = m destroys 1 - e^{-1} ≈ 63.21% of the option
        let (w0, pi_m, _) = exponential_lock_in_option_price(10.0, 0.0, 10.0);
        assert!(((pi_m / w0) - (1.0 - (-1.0_f64).exp())).abs() < 1e-12);

        // CMS Example 4: B = 5, r = 2, λ = 1 -> peaks at n* = 4 (N(4) ≈ 7.43), N(10) ≈ -9.04
        let (_, n4, auth4) = greedy_ratchet_trajectory(5.0, 2.0, 1.0, 4);
        let (inc10, n10, auth10) = greedy_ratchet_trajectory(5.0, 2.0, 1.0, 10);
        assert!(auth4 && auth10);
        assert!((n4 - 7.433_333).abs() < 1e-5);
        assert!((n10 - (-9.039_755)).abs() < 1e-5);
        assert!(inc10[3] > 0.0 && inc10[4] < 0.0); // u_4 = +0.6, u_5 = -0.333

        // CMS §7.3 & §12.3 worked example table
        assert!((grace_period_days(1.0, 0.05, 2.0, 10.0).unwrap() - 11.755_733).abs() < 1e-5);
        assert!((grace_period_days(1.0, 0.0, 2.0, 10.0).unwrap() - 8.0).abs() < 1e-12);
        assert!((grace_period_days(1.0, -0.05, 2.0, 10.0).unwrap() - 6.203_099).abs() < 1e-5);

        let (rho50, grace50, ron50, rebuild50) =
            alternation_load_and_rebuild_days(0.50, 1.0, 0.02, 0.20, 0.04, 0.02, 1.0, 0.05, 2.0, 10.0);
        assert!((rho50 - 0.080).abs() < 1e-12);
        assert!((grace50.unwrap() - 9.826_653).abs() < 1e-5);
        assert!((ron50 - 1.88).abs() < 1e-12);
        assert!((rebuild50.unwrap() - 43.773_436).abs() < 1e-5);
    }

    #[test]
    fn cms_preferences_trait_control_and_granovetter_pressure() {
        // CMS Example 5: Δ_0 = 0.6, δ = 1.5 -> k* = 1, u(1; θ_0) = -0.16, u(2; θ_0) = -1.96
        let (k_star, u_init) = endogenous_preference_unanimity_steps(0.6, 1.5, 2);
        assert_eq!(k_star, 1);
        assert!((u_init[0] - (-0.36)).abs() < 1e-12);
        assert!((u_init[1] - (-0.16)).abs() < 1e-12);
        assert!((u_init[2] - (-1.96)).abs() < 1e-12);

        // CMS Prop. 10: bk = a eliminates dependence on initial condition in 1 step
        let (carry, mean, _) = closed_loop_trait_dynamics(0.98, 1.0, 0.98, 5.0, 1.0, 1);
        assert!(carry.abs() < 1e-15);
        assert!((mean - 4.90).abs() < 1e-12);

        // CMS Example 6: v ~ N(-0.2, 1), s = 1.50 -> f0 = 0.4207, f* = 0.863, excess = 0.443
        let (f0, f_star, excess) = granovetter_threshold_equilibrium(-0.2, 1.0, 1.50);
        assert!((f0 - 0.420_740).abs() < 1e-4);
        assert!((f_star - 0.863_3).abs() < 1e-3);
        assert!((excess - 0.442_6).abs() < 1e-3);
    }

    #[test]
    fn sas_observational_equivalence_reversibility_and_erasure_compatibility() {
        // SAS Props. 1 & 8(b): Irreversible detention under observational equivalence collides
        let irrev = sas_observational_equivalence_irreversible_collision();
        irrev.assert_ct000();
        assert_eq!(irrev.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(irrev.guaranteeing_policy(), None);

        // Restricting to reversible administrative review or adding a discriminating sensor succeeds
        let rev_only = sas_reversibility_scaled_or_refined_structure(false);
        rev_only.assert_ct000();
        assert_eq!(rev_only.no_safe_action_witness(), None);
        assert_eq!(
            rev_only.guaranteeing_policy(),
            Some(vec![(
                "coarse_biometric_name_match",
                InstitutionalAction::ReversibleAdminReview
            )])
        );

        let refined = sas_reversibility_scaled_or_refined_structure(true);
        refined.assert_ct000();
        assert_eq!(refined.no_safe_action_witness(), None);

        // SAS §3.1 & §6.10: Base-rate fallacy in record linkage (π = 1e-4, s = 0.99, α = 1e-3)
        // P(match | flagged) ≈ 9.01%, so q_a(H) ≈ 90.99%: passes reversible review (ℓ=1, θ=0.95),
        // fails irreversible detention (ℓ=100, θ=0.95 -> max_err = 0.0095).
        let (p_match, _, ok_rev) =
            record_linkage_posterior_and_threshold(1e-4, 0.99, 1e-3, 0.95, 1.0);
        let (_, _, ok_det) =
            record_linkage_posterior_and_threshold(1e-4, 0.99, 1e-3, 0.95, 100.0);
        assert!((p_match - 0.090_090).abs() < 1e-5);
        assert!(ok_rev && !ok_det);

        // SAS Prop. 4: BFS reachability in recorded dependency graph
        let reach = dependency_reach_set(&[(1, 2), (2, 4), (1, 3), (5, 6)], 1);
        assert_eq!(reach, vec![1, 2, 3, 4]);

        // SAS Prop. 7: Erasure vs. subject-specific correction collision & pseudonymous token fix
        let erasure_col = sas_erasure_vs_subject_correction_collision();
        erasure_col.assert_ct000();
        assert_eq!(erasure_col.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(erasure_col.guaranteeing_policy(), None);

        let token_ok = sas_pseudonymous_token_correction_structure();
        token_ok.assert_ct000();
        assert_eq!(token_ok.no_safe_action_witness(), None);
        assert!(token_ok.guaranteeing_policy().is_some());
    }

    #[test]
    fn tct_bounded_charity_excuse_inflation_and_accommodation_ratchet() {
        // TCT Prop. 1 & Table 1: Collapsing intelligibility into justification collides
        let collapsed = tct_collapsed_attribution_collision();
        collapsed.assert_ct000();
        assert_eq!(collapsed.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(collapsed.guaranteeing_policy(), None);

        // Equal likelihoods with ε = 0.02 -> P(J = 1 | a, H) = 0.02 even though c_1 is possible
        assert!((justification_posterior(0.02, 0.5, 0.5) - 0.02).abs() < 1e-12);

        // TCT Prop. 4: Conduct-based bounded charity resolves the collision
        let bounded = tct_conduct_based_bounded_charity_structure();
        bounded.assert_ct000();
        assert_eq!(bounded.no_safe_action_witness(), None);
        assert_eq!(
            bounded.guaranteeing_policy(),
            Some(vec![(
                "surprising_costly_act_a",
                ObserverResponse::ValidateAndHoldBoundary
            )])
        );

        // TCT Prop. 2 & Cor. 1: v = 2, L = 10 -> p* = 0.80, exchange slope = 0.25 (4 honored per 1 broken)
        let (p_star, slope, n_star) = silent_trust_threshold_and_goodwill(2.0, 10.0, 4.0, 1.0, 12);
        assert!((p_star - 0.80).abs() < 1e-12);
        assert!((slope - 0.25).abs() < 1e-12);
        assert!((n_star - 3.0).abs() < 1e-12);

        // TCT Prop. 3: Disciplined (ρ = 0.5) vs. Undisciplined post-selected (r = 1.2)
        let (om_d, om_u, steps) = excuse_inflation_comparison(10.0, 0.5, 1.2, 5, 1.0);
        assert!((om_d - 0.3125).abs() < 1e-12);
        assert!((om_u - 24.8832).abs() < 1e-12);
        assert_eq!(steps, 4); // 10 * 0.5^4 = 0.625 <= 1.0

        // TCT Prop. 5 & Cor. 2: Accommodation ratchet vs. calm-period cap a* < C
        let increments = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let (bound, exhausted_at, yield_tr, cap_tr) =
            accommodation_ratchet_bound_and_trace(1.0, 5.0, 1.5, 0.5, 1.0, 3.5, &increments);
        assert!((bound - 11.0).abs() < 1e-12);
        assert_eq!(exhausted_at, Some(4)); // E_0 = 1 -> 2 -> 3 -> 4 -> 5 >= C
        assert_eq!(yield_tr, vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        assert!(cap_tr.iter().all(|&e| e <= 3.5 && e < 5.0)); // T = ∞ under cap!
    }
}
