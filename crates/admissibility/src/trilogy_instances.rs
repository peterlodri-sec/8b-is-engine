//! trilogy_instances — Executable witnesses for Flyxion's October 2026 Trilogy:
//!
//! 1. *The Relief of Explanation: Nietzsche, Formal Necessity, and the Feeling
//!    of Understanding* (`ROE`) — Definitions 3–4 (Continuation-Sufficiency and
//!    Collisions), Cracked-Wall simplification collision vs. productive
//!    complication, Flagpole/Shadow causal asymmetry, and Common-Cause
//!    observational equivalence vs. interventional `do(X = x)` divergence.
//! 2. *Toward an Evolutionary History of Terrestrial Oxygen: Impact Mixing,
//!    Reservoir Formation, and the Limits of Historical Reconstruction* (`TOH`)
//!    — The 9-Hypothesis Ledger (`H0a`–`H7`), the 5 classes of geochemical
//!    discriminants, the Xu et al. (2026) 20× extrusive-to-total LIP volume
//!    convention, and the electron-equivalent oxidizing-power budget Ω.
//! 3. *Distributed Material Archives: Multidimensional Optical Encoding,
//!    Embedded Manufacture, and the Physical Replication of Public Knowledge*
//!    (`DMA`) — Observational Equivalence zero-error capacity (`Prop. 3.1`),
//!    Fixed-Power Gaussian channel saturation (`Prop. 7.1`, Table 3), the
//!    8-state synchronous Reader FSM (`Sec. 9.4`, Table 6), Common-Mode
//!    replication floor (`Prop. 15.2`), Delta-Chain decay (`Prop. 15.3`), and
//!    Birth-Death persistence (`Prop. 16.1`).

use std::collections::HashSet;

use crate::Structure;

// ============================================================================
// I. THE RELIEF OF EXPLANATION (ROE)
// ============================================================================

/// Underlying structural states of the cracked-wall diagnostic case (`ROE` §1, §4, §12–13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WallSituation {
    /// Mortar failed a strength test; foundation is stable.
    WeakMortarOnly,
    /// Foundation is settling differentially, concentrating stress at the window corner.
    FoundationSettlement,
}

/// Admissible repair interventions for the cracked wall (`ROE` §12–13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallRepair {
    /// Repoint/replace mortar and wait (`ROE` §12: holds only when foundation is stable).
    RepointMortarOnly,
    /// Underpin foundation, correct drainage, and redistribute opening loads (`ROE` §13).
    UnderpinAndDrain,
}

/// Coarse account `A_coarse` (`ROE` §1, §4, §12): both situations are labeled
/// `"weak mortar"`. For the repair question `Q_repair`, the two situations have
/// disjoint durable repair continuations, producing a collision (`CT-000⁺`).
pub fn cracked_wall_coarse_structure() -> Structure<&'static str, WallRepair> {
    let obs = vec!["weak mortar", "weak mortar"];
    let adm = vec![
        HashSet::from([WallRepair::RepointMortarOnly]),
        HashSet::from([WallRepair::UnderpinAndDrain]),
    ];
    Structure::new(obs, adm)
}

/// Refined account `A_refined` (`ROE` §11–13, Productive Complication):
/// splits the `"weak mortar"` class by checking diagonal window-corner stress
/// concentration and foundation plumb/moisture history.
pub fn cracked_wall_refined_structure() -> Structure<&'static str, WallRepair> {
    let obs = vec![
        "mortar_crumble_level_foundation",
        "diagonal_window_corner_foundation_settlement",
    ];
    let adm = vec![
        HashSet::from([WallRepair::RepointMortarOnly]),
        HashSet::from([WallRepair::UnderpinAndDrain]),
    ];
    Structure::new(obs, adm)
}

/// Population least-squares slope `β_{Y|X}` under linear common-cause confounding
/// `X = a Z + ε_X`, `Y = b Z + ε_Y` (`ROE` §9, Eq. 3).
pub fn common_cause_ols_slope(a: f64, b: f64, var_z: f64, var_eps_x: f64) -> f64 {
    let denom = a * a * var_z + var_eps_x;
    assert!(denom > 0.0, "Var(X) must be strictly positive");
    (a * b * var_z) / denom
}

/// Models two observationally equivalent data-generating processes (`ROE` §9):
/// - World 0: Direct causal influence `X -> Y` with structural coefficient `β_{Y|X}`
/// - World 1: Pure common cause `X <- Z -> Y` with zero structural effect of `X` on `Y`
///
/// When `for_intervention` is `false` (forecasting `Y` from passively observed `X = x`),
/// both worlds admit the same OLS predictor (`no_safe_action_witness() == None`).
/// When `for_intervention` is `true` (predicting `Y` under `do(X = x)`, `ROE` Eq. 4),
/// their continuations diverge (`no_safe_action_witness() == Some((0, 1))`).
pub fn observational_equivalence_vs_intervention(
    for_intervention: bool,
) -> Structure<&'static str, &'static str> {
    let obs = vec!["fitted_ols_slope_beta_yx", "fitted_ols_slope_beta_yx"];
    let adm = if for_intervention {
        vec![
            HashSet::from(["shift_y_by_beta_times_x"]),
            HashSet::from(["y_distribution_unchanged_under_do_x"]),
        ]
    } else {
        vec![
            HashSet::from(["forecast_y_via_conditional_expectation"]),
            HashSet::from(["forecast_y_via_conditional_expectation"]),
        ]
    };
    Structure::new(obs, adm)
}

// ============================================================================
// II. TERRESTRIAL OXYGEN HISTORY (TOH)
// ============================================================================

/// The 9 competing hypotheses in the ledger for oxidized mantle reservoirs (`TOH` §6, Table 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OxygenHypothesis {
    /// H0a: Analytical or EMPA matrix/calibration artifact.
    H0aAnalyticalArtifact,
    /// H0b: Real high-pressure capacity confined to PtO2 laboratory conditions.
    H0bLaboratoryCapacityOnly,
    /// H1: Ordinary Fe2+/Fe3+ redox without independent non-stoichiometric oxygen.
    H1OrdinaryIronRedox,
    /// H2: Primitive mantle oxygen excess retained from accretion (Andrault et al. 2018).
    H2PrimitiveAccretionExcess,
    /// H3: Recycled surface-oxidized material delivered by subduction.
    H3SubductedRecycledCrust,
    /// H4: Lower-mantle Fe2+ disproportionation with metal segregation to the core (Frost et al. 2008).
    H4IronDisproportionation,
    /// H5: Deep water + iron forming FeO2/FeOOH with H2 escape (Hu et al. 2016).
    H5DeepPeroxideHydrogenLoss,
    /// H6: Pre-Moon ocean volatiles vaporized in the giant impact/synestia with H2 loss.
    H6PreMoonOceanGiantImpact,
    /// H7: Subsurface radiolysis of water with sustained product separation (Lin et al. 2005).
    H7WaterRadiolysis,
}

impl OxygenHypothesis {
    pub const ALL: [OxygenHypothesis; 9] = [
        Self::H0aAnalyticalArtifact,
        Self::H0bLaboratoryCapacityOnly,
        Self::H1OrdinaryIronRedox,
        Self::H2PrimitiveAccretionExcess,
        Self::H3SubductedRecycledCrust,
        Self::H4IronDisproportionation,
        Self::H5DeepPeroxideHydrogenLoss,
        Self::H6PreMoonOceanGiantImpact,
        Self::H7WaterRadiolysis,
    ];
}

/// Under the laboratory quenched-glass observation alone (`TOH` §2), all 9 ledger
/// hypotheses remain in the same coarse observational class: committing to a
/// single unique historical narrative produces a collision (`CT-000⁺`).
pub fn oxygen_ledger_lab_only_structure() -> Structure<&'static str, OxygenHypothesis> {
    let obs = vec!["pto2_quenched_glass_5_2_wt_pct_at_7gpa"; 9];
    let adm = OxygenHypothesis::ALL
        .iter()
        .map(|&h| HashSet::from([h]))
        .collect();
    Structure::new(obs, adm)
}

/// Five-tuple of discriminating observable classes from `TOH` §7:
/// `(natural_excess_o, iron_valence_exceeded, isotope_dh_o, primordial_noble_gas, budget_class)`.
pub type GeochemicalDiscriminants = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

/// Joining the 5 classes of discriminating observations (`TOH` §7) refines the
/// partition across `H0a`–`H7` so that every hypothesis class is separated.
pub fn oxygen_ledger_discriminated_structure() -> Structure<GeochemicalDiscriminants, OxygenHypothesis>
{
    let obs = vec![
        // H0a: disappears under independent O analysis & controlled fugacity
        ("lab_residual_vanishes", "none", "baseline", "baseline", "zero"),
        // H0b: lab residual real, but absent in natural glasses/melt inclusions
        ("lab_only_no_natural_excess", "fe_only", "baseline", "baseline", "lab_endmember"),
        // H1: natural redox tracks Fe2+/Fe3+ stoichiometry exclusively
        ("natural_tracks_fe3_stoich", "fe_balanced", "mantle_normal", "degassed", "omega_fe_only"),
        // H2: ancient unhomogenized domains with primordial noble gas (3He/22Ne, 129Xe)
        ("natural_excess_present", "exceeds_fe3", "chondritic", "primordial_unhomogenized", "accretion_budget"),
        // H3: oxygen-rich plumes carrying recycled crustal isotopic signatures
        ("natural_excess_present", "exceeds_fe3", "recycled_crustal", "degassed", "subduction_flux"),
        // H4: core-mantle disproportionation signature without D/H fractionation
        ("natural_excess_present", "exceeds_fe3", "unfractionated_h", "degassed", "core_segregation_fe0"),
        // H5: deep lower-mantle FeO2/FeOOH phase stability + H-depleted signature
        ("natural_excess_present", "exceeds_fe3", "deep_peroxide_dh", "degassed", "pyrite_phase_h2_loss"),
        // H6: pre-Moon impact steam-atmosphere H2 hydrodynamic escape + deep >5 GPa return
        ("natural_excess_present", "exceeds_fe3", "impact_steam_escape_dh", "synestia_mixed", "giant_impact_ocean_h2_loss"),
        // H7: radiolytic H2/oxidant yield bounded by integrated radiogenic energy
        ("natural_excess_present", "exceeds_fe3", "radiolytic_dh", "crustal_radiogenic", "radiogenic_budget"),
    ];
    let adm = OxygenHypothesis::ALL
        .iter()
        .map(|&h| HashSet::from([h]))
        .collect();
    Structure::new(obs, adm)
}

/// Bayesian posterior weight `w(H | O) = μ(A_H ∩ C_O) / μ(C_O)` (`TOH` Appendix A).
pub fn posterior_hypothesis_weight(mu_ah_and_co: f64, mu_co: f64) -> f64 {
    assert!(mu_co > 0.0, "observation region must have positive prior measure");
    mu_ah_and_co / mu_co
}

/// Audits the Xu et al. (2026) Large Igneous Province (LIP) excess-oxygen scaling
/// (`TOH` §2 & Appendix B).
///
/// - `quoted_extrusive_km3`: 7.2e6 km³ (41 provinces × 0.18e6 km² × 1 km)
/// - `extrusive_ratio`: `None` uses the raw 7.2e6 km³; `Some((0.05, true))` treats
///   extrusive as 5% of total (144e6 km³); `Some((0.05, false))` treats extrusive
///   as 5% of intrusive (151.2e6 km³).
pub fn lip_released_oxygen_mass_kg(
    quoted_extrusive_km3: f64,
    extrusive_ratio: Option<(f64, bool)>,
    density_kg_m3: f64,
    excess_oxygen_wt_frac: f64,
) -> f64 {
    let total_km3 = match extrusive_ratio {
        None => quoted_extrusive_km3,
        Some((ratio, extrusive_is_fraction_of_total)) => {
            if extrusive_is_fraction_of_total {
                quoted_extrusive_km3 / ratio
            } else {
                quoted_extrusive_km3 * (1.0 + 1.0 / ratio)
            }
        }
    };
    let total_m3 = total_km3 * 1.0e9;
    let total_magma_mass_kg = total_m3 * density_kg_m3;
    total_magma_mass_kg * excess_oxygen_wt_frac
}

/// Electron-equivalent oxidizing power supplied by losing `n_h2_moles` of H₂
/// (`TOH` Appendix B: `ΔΩ_residue = 2 * n_{H2}`).
pub fn oxidizing_power_from_h2_loss_moles_e(n_h2_moles: f64) -> f64 {
    2.0 * n_h2_moles
}

/// Maximum electron-equivalents consumable by oxidizing mantle Fe²⁺ to Fe³⁺
/// (`TOH` Appendix B: `Ω_Fe^max = M * x_Fe / m_Fe`).
pub fn max_ferrous_iron_sink_moles_e(
    mantle_mass_kg: f64,
    fe2_mass_fraction: f64,
    molar_mass_fe_kg_per_mol: f64,
) -> f64 {
    (mantle_mass_kg * fe2_mass_fraction) / molar_mass_fe_kg_per_mol
}

// ============================================================================
// III. DISTRIBUTED MATERIAL ARCHIVES (DMA)
// ============================================================================

/// Zero-error capacity `⌊log₂ |ω(U)|⌋` of a configuration space `U` under a
/// deterministic readout map `ω` (`DMA` Definition 3.1 & Proposition 3.1).
pub fn zero_error_capacity_bits<Y: Eq + std::hash::Hash>(readout_observations: &[Y]) -> u32 {
    let distinct: HashSet<&Y> = readout_observations.iter().collect();
    assert!(!distinct.is_empty(), "configuration space must be non-empty");
    distinct.len().ilog2()
}

/// Capacity `C(d) = (d / 2) * log₂(1 + P / (d * N))` of `d` parallel real Gaussian
/// channels sharing a fixed total signal-to-noise budget `x = P / N` (`DMA` Eq. 5, Prop. 7.1).
pub fn gaussian_fixed_power_capacity_bits(d: usize, snr_total_linear: f64) -> f64 {
    assert!(d >= 1, "mode count d must be at least 1");
    assert!(snr_total_linear > 0.0, "P/N must be positive");
    let d_f = d as f64;
    0.5 * d_f * (1.0 + snr_total_linear / d_f).log2()
}

/// Asymptotic saturation limit `lim_{d -> ∞} C(d) = (P / N) / (2 ln 2)` (`DMA` Prop. 7.1).
pub fn gaussian_fixed_power_limit_bits(snr_total_linear: f64) -> f64 {
    snr_total_linear / (2.0 * std::f64::consts::LN_2)
}

/// The 8 states of the synchronous Reader Controller FSM (`DMA` §9.4, Table 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReaderState {
    Idle,
    Align,
    Calibrate,
    Scan,
    Decode,
    Verify,
    Done,
    Fail,
}

/// Control inputs to the synchronous Reader FSM (`DMA` §9.4, Table 6).
#[derive(Debug, Clone, Copy, Default)]
pub struct ReaderInputs {
    pub start: bool,
    pub aligned: bool,
    pub retry: bool,
    pub calibrated: bool,
    pub scan_done: bool,
    pub decoded: bool,
    pub more: bool,
    pub enough: bool,
    pub reset: bool,
}

impl ReaderState {
    /// Deterministic next-state transition implementing `DMA` Table 6 and the
    /// one-hot flip-flop equations of §9.4.
    pub fn step(self, inp: ReaderInputs) -> Self {
        if inp.reset && matches!(self, Self::Done | Self::Fail) {
            return Self::Idle;
        }
        match self {
            Self::Idle => {
                if inp.start {
                    Self::Align
                } else {
                    Self::Idle
                }
            }
            Self::Align => {
                if inp.aligned {
                    Self::Calibrate
                } else if inp.retry {
                    Self::Align
                } else {
                    Self::Fail
                }
            }
            Self::Calibrate => {
                if inp.calibrated {
                    Self::Scan
                } else {
                    Self::Fail
                }
            }
            Self::Scan => {
                if inp.scan_done {
                    Self::Decode
                } else {
                    Self::Scan
                }
            }
            Self::Decode => {
                if inp.decoded {
                    Self::Verify
                } else {
                    Self::Decode
                }
            }
            Self::Verify => {
                if inp.enough {
                    Self::Done
                } else if inp.more {
                    Self::Scan
                } else {
                    Self::Fail
                }
            }
            Self::Done => Self::Done,
            Self::Fail => Self::Fail,
        }
    }
}

/// Probability of total archive loss `q + (1 - q) * p^r` with common-mode failure
/// probability `q` and `r` copies failing independently with probability `p`
/// (`DMA` Proposition 15.2).
pub fn common_mode_archive_loss_prob(p: f64, q: f64, r: u32) -> f64 {
    q + (1.0 - q) * p.powi(r as i32)
}

/// Availability `a^v` of a delta chain of length `v` where each version has `m`
/// independent copies surviving with probability `s` (`DMA` Proposition 15.3).
pub fn delta_chain_availability(s: f64, m: u32, v: u32) -> (f64, f64) {
    let single_version_avail = 1.0 - (1.0 - s).powi(m as i32);
    let chain_avail = single_version_avail.powi(v as i32);
    (single_version_avail, chain_avail)
}

/// Extinction probability `min(1, μ / β)^{N_0}` of a perishable carrier lineage
/// under linear birth rate `β` and loss rate `μ` (`DMA` Proposition 16.1).
pub fn perishable_archive_extinction_prob(mu: f64, beta: f64, n0: u32) -> f64 {
    assert!(beta > 0.0 && mu >= 0.0, "rates must be non-negative with beta > 0");
    let single = (mu / beta).min(1.0);
    single.powi(n0 as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roe_cracked_wall_collision_and_productive_complication() {
        let coarse = cracked_wall_coarse_structure();
        coarse.assert_ct000();
        // Under "weak mortar" alone, WeakMortarOnly (0) and FoundationSettlement (1)
        // collide on Q_repair: no single repair guarantees durability.
        assert_eq!(coarse.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(coarse.guaranteeing_policy(), None);

        // Productive complication splits the equivalence class along the
        // window-corner diagonal stress & foundation-settlement observation.
        let refined = cracked_wall_refined_structure();
        refined.assert_ct000();
        assert_eq!(refined.no_safe_action_witness(), None);
        assert_eq!(
            refined.guaranteeing_policy(),
            Some(vec![
                ("mortar_crumble_level_foundation", WallRepair::RepointMortarOnly),
                (
                    "diagonal_window_corner_foundation_settlement",
                    WallRepair::UnderpinAndDrain
                ),
            ])
        );
    }

    #[test]
    fn roe_common_cause_forecasting_vs_do_intervention() {
        // ROE Eq. 3: β_{Y|X} = a b Var(Z) / (a^2 Var(Z) + Var(ε_X))
        let beta = common_cause_ols_slope(2.0, 3.0, 4.0, 8.0);
        // Numerator: 2 * 3 * 4 = 24; Denominator: 4 * 4 + 8 = 24 => β = 1.0
        assert!((beta - 1.0).abs() < 1e-12);

        // Adequate for forecasting (passive conditional P(Y | X = x)):
        let forecast = observational_equivalence_vs_intervention(false);
        forecast.assert_ct000();
        assert_eq!(forecast.no_safe_action_witness(), None);
        assert!(forecast.guaranteeing_policy().is_some());

        // Inadequate (collides) when the question shifts to manipulation do(X = x):
        let manipulate = observational_equivalence_vs_intervention(true);
        manipulate.assert_ct000();
        assert_eq!(manipulate.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(manipulate.guaranteeing_policy(), None);
    }

    #[test]
    fn toh_ledger_underdetermination_and_five_class_discrimination() {
        // Under the lab quenched-glass result alone, all 9 hypotheses collide:
        let lab_only = oxygen_ledger_lab_only_structure();
        lab_only.assert_ct000();
        assert_eq!(lab_only.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(lab_only.guaranteeing_policy(), None);

        // Joining the 5 observable classes separates all 9 hypotheses cleanly:
        let discriminated = oxygen_ledger_discriminated_structure();
        discriminated.assert_ct000();
        assert_eq!(discriminated.no_safe_action_witness(), None);
        let policy = discriminated.guaranteeing_policy().unwrap();
        assert_eq!(policy.len(), 9);
    }

    #[test]
    fn toh_xu2026_lip_volume_convention_and_oxidizing_budget() {
        // Unscaled 7.2e6 km³ extrusive volume gives ~5.85e16 kg O2 (20× below 1.2e18 kg)
        let raw_o2 = lip_released_oxygen_mass_kg(7.2e6, None, 3010.0, 0.0027);
        assert!((raw_o2 - 5.851_44e16).abs() < 1e13);

        // Convention A: extrusive is 5% of total (144e6 km³) -> ~1.17e18 kg O2
        let conv_total = lip_released_oxygen_mass_kg(7.2e6, Some((0.05, true)), 3010.0, 0.0027);
        assert!((conv_total - 1.170_288e18).abs() < 1e14);

        // Convention B: extrusive is 5% of intrusive (151.2e6 km³) -> ~1.23e18 kg O2
        let conv_intr = lip_released_oxygen_mass_kg(7.2e6, Some((0.05, false)), 3010.0, 0.0027);
        assert!((conv_intr - 1.228_802_4e18).abs() < 1e14);

        // Electron-equivalent budget: 1 mol H2 lost -> 2 mol e- oxidizing power
        assert_eq!(oxidizing_power_from_h2_loss_moles_e(50.0), 100.0);
        let fe_max = max_ferrous_iron_sink_moles_e(4.0e24, 0.06, 0.055845);
        assert!(fe_max > 4.2e24 && fe_max < 4.4e24);
    }

    #[test]
    fn dma_observational_equivalence_and_gaussian_power_saturation() {
        // 16 control settings that map onto only 4 distinct observations -> 2 bits (not 4 bits)
        let readout = [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3];
        assert_eq!(zero_error_capacity_bits(&readout), 2);

        // DMA Table 3: P/N = 100 (20 dB)
        let expected_table_3 = [
            (1, 3.33),
            (4, 9.40),
            (16, 22.86),
            (64, 43.44),
            (256, 60.89),
            (1024, 68.83),
        ];
        for &(d, expected) in &expected_table_3 {
            let cap = gaussian_fixed_power_capacity_bits(d, 100.0);
            assert!(
                (cap - expected).abs() < 0.01,
                "d={d}: got {cap:.4}, expected {expected:.2}"
            );
        }
        let lim = gaussian_fixed_power_limit_bits(100.0);
        assert!((lim - 72.134_752).abs() < 1e-5);
    }

    #[test]
    fn dma_reader_fsm_early_verify_termination_and_bounded_retry() {
        // Happy path with 1 alignment retry and 2 block scans until `enough` holds
        let mut st = ReaderState::Idle;
        st = st.step(ReaderInputs { start: true, ..Default::default() });
        assert_eq!(st, ReaderState::Align);

        // First alignment fails with retry=true -> stays in Align
        st = st.step(ReaderInputs { aligned: false, retry: true, ..Default::default() });
        assert_eq!(st, ReaderState::Align);

        // Second alignment succeeds -> Calibrate -> Scan -> Decode -> Verify
        st = st.step(ReaderInputs { aligned: true, ..Default::default() });
        assert_eq!(st, ReaderState::Calibrate);
        st = st.step(ReaderInputs { calibrated: true, ..Default::default() });
        assert_eq!(st, ReaderState::Scan);
        st = st.step(ReaderInputs { scan_done: true, ..Default::default() });
        assert_eq!(st, ReaderState::Decode);
        st = st.step(ReaderInputs { decoded: true, ..Default::default() });
        assert_eq!(st, ReaderState::Verify);

        // First block verified, not yet `enough` (k blocks), but `more` unread blocks remain -> Scan
        st = st.step(ReaderInputs { enough: false, more: true, ..Default::default() });
        assert_eq!(st, ReaderState::Scan);

        // Second block reaches `enough` -> Done (skips remaining damaged regions!)
        st = st.step(ReaderInputs { scan_done: true, ..Default::default() });
        st = st.step(ReaderInputs { decoded: true, ..Default::default() });
        st = st.step(ReaderInputs { enough: true, more: true, ..Default::default() });
        assert_eq!(st, ReaderState::Done);

        // Reset returns to Idle
        st = st.step(ReaderInputs { reset: true, ..Default::default() });
        assert_eq!(st, ReaderState::Idle);
    }

    #[test]
    fn dma_common_mode_floor_delta_chains_and_perishable_persistence() {
        // DMA Prop. 15.2 numbers: p = 0.1, q = 1e-3
        assert!((common_mode_archive_loss_prob(0.1, 1e-3, 1) - 0.1009).abs() < 1e-6);
        assert!((common_mode_archive_loss_prob(0.1, 1e-3, 2) - 0.01099).abs() < 1e-6);
        assert!((common_mode_archive_loss_prob(0.1, 1e-3, 4) - 0.001_099_9).abs() < 1e-7);
        assert!((common_mode_archive_loss_prob(0.1, 1e-3, 6) - 0.001_000_999).abs() < 1e-9);

        // DMA Prop. 15.3: a = 0.99 -> a^20 ≈ 0.8179, a^100 ≈ 0.3660
        let (a, chain20) = delta_chain_availability(0.9, 2, 20);
        assert!((a - 0.99).abs() < 1e-12);
        assert!((chain20 - 0.817_906_9).abs() < 1e-6);
        let (_, chain100) = delta_chain_availability(0.9, 2, 100);
        assert!((chain100 - 0.366_032_3).abs() < 1e-6);

        // DMA Prop. 16.1 ("weaponized manna" birth-death persistence):
        assert!((perishable_archive_extinction_prob(0.5, 1.0, 3) - 0.125).abs() < 1e-12);
        assert_eq!(perishable_archive_extinction_prob(1.2, 1.0, 5), 1.0);
    }
}
