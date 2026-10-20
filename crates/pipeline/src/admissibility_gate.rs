//! admissibility_gate.rs — Central Theorem (`CT-000` / `CT-003` / `CT-015` / `CT-016`)
//! gate for the Layer-1 ingestion and tick pipeline.
//!
//! Connects the engine spine (`seed → GAIA → cast → keeper fold → spherepop →
//! Seed 001 stock-vs-flow ledger → durable stage`) before any asset is staged:
//!
//! 1. **Seed & GAIA verification**: checks that the ternary board contains only
//!    `{-1, 0, +1}` trits, that `tick >= manifest.gaia.t` (`CT-003` monotonicity),
//!    and that `gaia_state(brief, tick)` agrees with the manifest's constants.
//! 2. **Cast & Keeper Fold (`CT-000` / `CT-016`)**: projects the zone's bestiary
//!    cast through `admissibility::keeper_instance::admissible_actions`, verifies
//!    the multi-actor `Structure` satisfies `assert_ct000()` with a guaranteeing
//!    policy, adjudicates the cast on a live `world_core::fold::Keeper`, and pins
//!    `CT-016` (a poison refusal owns a ledger sequence slot without moving the
//!    material zone fold).
//! 3. **Spherepop Witness (`CT-000` / `CT-015` / `CT-016`)**: builds a zone-seeded
//!    nested bubble expression, witnesses that an outer-first refusal shifts the
//!    history `digest()` while preserving the visible `render()` frontier, verifies
//!    `structure_from_spheres` under `assert_ct000()`, and evaluates `pop_all()`.
//! 4. **Seed 001 Stock-vs-Flow Ledger (`CT-000` / `CT-000⁺`)**: confirms that
//!    `flow_only_ledger()` blurs idle reserve with decay (`no_safe_action_witness`)
//!    while `seed001_maintenance_ledger()` guarantees an admissible allocation
//!    continuation (`HoldAndMaintain`).
//! 5. **Golden Hash Attestation**: seals all witness outputs into a deterministic
//!    SHA-256 `golden_hash` on [`AdmissibilityReceipt`].

use std::collections::HashSet;

use admissibility::keeper_instance::admissible_actions;
use admissibility::seed001_instance::{
    flow_only_ledger, seed001_maintenance_ledger, StockAndMaintenanceObs,
};
use admissibility::spherepop_instance::structure_from_spheres;
use admissibility::Structure;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spherepop::{Refusal, Sphere};
use world_core::fold::{Delta, Keeper, Verdict};
use world_core::gaia::gaia_state;

use crate::gdd_parser::ZoneManifest;

/// Deterministic witness receipt emitted by the pipeline's admissibility gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissibilityReceipt {
    pub brief: String,
    pub seed_line: String,
    pub tick: u64,
    pub gaia_memory: String,
    pub keeper_admitted: u64,
    pub keeper_refused: u64,
    pub keeper_seq: u64,
    pub sphere_expr: String,
    pub sphere_value: i64,
    pub sphere_digest: String,
    pub seed001_action: String,
    pub ct000_verified: bool,
    pub golden_hash: String,
}

/// Map a ternary board slice at `actor_idx` into attested needs `(h, r, s)` in `[0.0, 1.0]`.
///
/// `h` is always placed in `{0.20, 0.30, 0.34}` (`<= 0.35`) so every actor in the
/// zone's seeded cast has at least one entitled continuation (`"forage the field"`),
/// while `r` (`<= 0.30` when trit `<= 0`) and `s` (`<= 0.40` when trit `<= 0`) open
/// additional admissible continuations (`"sleep in the tent"`, `"flee to the gate"`).
fn cast_needs_from_board(board: &[i8], actor_idx: usize) -> (f64, f64, f64) {
    let len = board.len();
    let th = board[(actor_idx * 3) % len];
    let tr = board[(actor_idx * 3 + 1) % len];
    let ts = board[(actor_idx * 3 + 2) % len];

    let h = match th {
        -1 => 0.20,
        0 => 0.30,
        _ => 0.34,
    };
    let r = if tr <= 0 { 0.25 } else { 0.85 };
    let s = if ts <= 0 { 0.35 } else { 0.85 };
    (h, r, s)
}

/// Gate a [`ZoneManifest`] at `tick`, verifying all Central Theorem (`CT-000`,
/// `CT-003`, `CT-015`, `CT-016`) invariants across GAIA, Keeper, Spherepop, and
/// Seed 001 before emitting a deterministic [`AdmissibilityReceipt`].
pub fn gate_zone_tick(manifest: &ZoneManifest, tick: u64) -> Result<AdmissibilityReceipt, String> {
    // 1. Seed, ternary wire, and GAIA checks
    if manifest.brief.trim().is_empty() {
        return Err("admissibility gate: empty brief".into());
    }
    if manifest.seed_line.trim().is_empty() {
        return Err("admissibility gate: empty seed_line".into());
    }
    if manifest.board.is_empty() {
        return Err("admissibility gate: empty ternary board".into());
    }
    for (idx, &trit) in manifest.board.iter().enumerate() {
        if !(-1..=1).contains(&trit) {
            return Err(format!(
                "admissibility gate: corrupted ternary board at tile {idx}: trit {trit} outside {{-1, 0, +1}}"
            ));
        }
    }
    if !manifest.gaia.gravity.is_finite() || manifest.gaia.gravity <= 0.0 {
        return Err("admissibility gate: invalid GAIA gravity".into());
    }
    if !(0.0..=1.0).contains(&manifest.gaia.light) {
        return Err("admissibility gate: invalid GAIA light outside [0, 1]".into());
    }
    if tick < manifest.gaia.t {
        return Err(format!(
            "admissibility gate: tick {tick} precedes manifest birth tick {} (CT-003)",
            manifest.gaia.t
        ));
    }

    let expected_at_birth = gaia_state(&manifest.brief, manifest.gaia.t);
    if manifest.gaia != expected_at_birth {
        return Err(
            "admissibility gate: manifest.gaia diverges from deterministic gaia_state(brief, t)"
                .into(),
        );
    }

    let gaia_now = gaia_state(&manifest.brief, tick);
    if gaia_now.entropy < manifest.gaia.entropy {
        return Err("admissibility gate: GAIA entropy is not monotone".into());
    }
    if (gaia_now.gravity - manifest.gaia.gravity).abs() > 1e-9 {
        return Err("admissibility gate: GAIA gravity drifted across ticks".into());
    }

    // 2. Cast -> Keeper Fold & CT-000 multi-actor structure
    let actor_names: Vec<&str> = if manifest.archetypes.is_empty() {
        vec!["sentinel"]
    } else {
        manifest
            .archetypes
            .iter()
            .map(|a| a.name.as_str())
            .collect()
    };

    let mut obs_vec: Vec<String> = Vec::with_capacity(actor_names.len());
    let mut adm_vec: Vec<HashSet<&'static str>> = Vec::with_capacity(actor_names.len());
    let mut needs_vec: Vec<(f64, f64, f64)> = Vec::with_capacity(actor_names.len());

    for (idx, _) in actor_names.iter().enumerate() {
        let (h, r, s) = cast_needs_from_board(&manifest.board, idx);
        let obs = format!("needs({h:.2},{r:.2},{s:.2})");
        let adm: HashSet<&'static str> = admissible_actions(h, r, s).into_iter().collect();
        if adm.is_empty() {
            return Err(format!(
                "admissibility gate: actor {idx} has empty admissible set at ({h}, {r}, {s})"
            ));
        }
        obs_vec.push(obs);
        adm_vec.push(adm);
        needs_vec.push((h, r, s));
    }

    let cast_structure = Structure::new(obs_vec.clone(), adm_vec);
    cast_structure.assert_ct000();
    if cast_structure.no_safe_action_witness().is_some() {
        return Err("admissibility gate: cast structure has blurred observation class (CT-000+)".into());
    }
    let cast_policy = cast_structure
        .guaranteeing_policy()
        .ok_or_else(|| "admissibility gate: cast guaranteeing_policy returned None".to_string())?;

    let mut keeper = Keeper::default();
    let base_tick = tick + 1;
    for (idx, &actor) in actor_names.iter().enumerate() {
        let (h, r, s) = needs_vec[idx];
        // Birth at base_tick
        let birth = Delta {
            t: base_tick,
            h,
            r,
            s,
            action: None,
            asleep: false,
        };
        if keeper.adjudicate(actor, &birth) != Verdict::Admitted {
            return Err(format!("admissibility gate: keeper refused birth for {actor}"));
        }

        // Entitled continuation selected by the CT-000 guaranteeing policy at base_tick + 1
        let chosen_action = cast_policy
            .iter()
            .find(|(o, _)| o == &obs_vec[idx])
            .map(|(_, a)| *a)
            .ok_or_else(|| "admissibility gate: missing policy entry for actor class".to_string())?;
        let step_delta = Delta {
            t: base_tick + 1,
            h,
            r,
            s,
            action: Some(chosen_action.to_string()),
            asleep: false,
        };
        if keeper.adjudicate(actor, &step_delta) != Verdict::Admitted {
            return Err(format!(
                "admissibility gate: keeper refused guaranteed action {chosen_action} for {actor}"
            ));
        }

        // CT-016 witness: a poison action (attesting comfort while foraging) at base_tick + 2
        // must be refused durable, advancing seq while leaving the zone fold untouched.
        let poison = Delta {
            t: base_tick + 2,
            h: 0.95,
            r: 0.95,
            s: 0.95,
            action: Some("forage the field".to_string()),
            asleep: false,
        };
        if keeper.adjudicate(actor, &poison) != Verdict::Refused("action without need") {
            return Err(format!(
                "admissibility gate: keeper failed to refuse poison delta for {actor} (CT-016)"
            ));
        }
        if keeper.zone.get(actor).copied() != Some([h, r, s]) {
            return Err("admissibility gate: refused poison delta mutated keeper zone state".into());
        }
    }

    // 3. Spherepop innermost-first witness (CT-000 / CT-015 / CT-016)
    let b0 = (manifest.board[0] as i64) + 3;
    let b1 = (manifest.board[1 % manifest.board.len()] as i64) + 3;
    let b2 = (manifest.board[2 % manifest.board.len()] as i64) + 3;
    let b3 = ((tick % 7) as i64) + 1;
    let sphere_expr = format!("((({b0} + {b1}) * {b2}) + {b3})");

    let mut s_clean = Sphere::parse(&sphere_expr).map_err(|e| e.to_string())?;
    let mut s_refused = Sphere::parse(&sphere_expr).map_err(|e| e.to_string())?;

    // Outer-first attempt on s_refused must record Refusal::InnerFirst { blocking: 1 } (CT-016)
    if s_refused.pop(0) != Err(Refusal::InnerFirst { blocking: 1 }) {
        return Err("admissibility gate: spherepop outer-first pop did not refuse".into());
    }
    // Pop innermost bubble 2 on both spheres
    let inner_clean = s_clean.pop(2).map_err(|e| e.to_string())?;
    let inner_refused = s_refused.pop(2).map_err(|e| e.to_string())?;
    if inner_clean != inner_refused || s_clean.render() != s_refused.render() {
        return Err("admissibility gate: spherepop render diverged after innermost pop".into());
    }
    if s_clean.digest() == s_refused.digest() {
        return Err("admissibility gate: spherepop history digest ignored CT-016 refusal".into());
    }

    let sp_structure = structure_from_spheres(&[s_clean.clone(), s_refused], |s| s.render());
    sp_structure.assert_ct000();
    if sp_structure.guaranteeing_policy().is_none() {
        return Err("admissibility gate: spherepop frontier structure lacks policy".into());
    }

    let sphere_value = s_clean.pop_all().map_err(|e| e.to_string())?;
    let sphere_digest = format!("{:016x}", s_clean.digest());

    // 4. Seed 001 stock-vs-flow maintenance ledger witness (CT-000 / CT-000+)
    let flow_only = flow_only_ledger();
    flow_only.assert_ct000();
    if flow_only.no_safe_action_witness().is_none() {
        return Err("admissibility gate: flow_only_ledger failed to witness CT-000+ blur".into());
    }

    let maint = seed001_maintenance_ledger();
    maint.assert_ct000();
    let maint_policy = maint
        .guaranteeing_policy()
        .ok_or_else(|| "admissibility gate: seed001_maintenance_ledger lacks policy".to_string())?;
    let target_obs = if tick == 0 {
        StockAndMaintenanceObs::HeldMaintainedReserve
    } else {
        StockAndMaintenanceObs::HeldMaintainedActive
    };
    let seed001_action = maint_policy
        .into_iter()
        .find(|(obs, _)| *obs == target_obs)
        .map(|(_, act)| format!("{act:?}"))
        .ok_or_else(|| "admissibility gate: missing Seed 001 action for observation".to_string())?;

    // 5. Seal the deterministic receipt with a SHA-256 golden hash
    let golden_hash = compute_golden_hash(
        &manifest.brief,
        &manifest.seed_line,
        tick,
        &gaia_now.memory,
        keeper.admitted,
        keeper.refused,
        keeper.seq,
        &sphere_expr,
        sphere_value,
        &sphere_digest,
        &seed001_action,
    );

    Ok(AdmissibilityReceipt {
        brief: manifest.brief.clone(),
        seed_line: manifest.seed_line.clone(),
        tick,
        gaia_memory: gaia_now.memory,
        keeper_admitted: keeper.admitted,
        keeper_refused: keeper.refused,
        keeper_seq: keeper.seq,
        sphere_expr,
        sphere_value,
        sphere_digest,
        seed001_action,
        ct000_verified: true,
        golden_hash,
    })
}

#[allow(clippy::too_many_arguments)]
fn compute_golden_hash(
    brief: &str,
    seed_line: &str,
    tick: u64,
    gaia_memory: &str,
    keeper_admitted: u64,
    keeper_refused: u64,
    keeper_seq: u64,
    sphere_expr: &str,
    sphere_value: i64,
    sphere_digest: &str,
    seed001_action: &str,
) -> String {
    let payload = format!(
        "v1|{brief}|{seed_line}|{tick}|{gaia_memory}|{keeper_admitted}|{keeper_refused}|{keeper_seq}|{sphere_expr}|{sphere_value}|{sphere_digest}|{seed001_action}"
    );
    let bytes = Sha256::digest(payload.as_bytes());
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Re-run the admissibility gate on `manifest` at `receipt.tick` and verify
/// that every witness field and the `golden_hash` match byte-for-byte.
pub fn verify_receipt(
    manifest: &ZoneManifest,
    receipt: &AdmissibilityReceipt,
) -> Result<(), String> {
    if !receipt.ct000_verified {
        return Err("receipt has ct000_verified == false".into());
    }
    let expected = gate_zone_tick(manifest, receipt.tick)?;
    if &expected != receipt {
        return Err(format!(
            "admissibility receipt mismatch: expected golden_hash {}, got {}",
            expected.golden_hash, receipt.golden_hash
        ));
    }
    Ok(())
}

/// Adjudicate an explicit sequence of actor tick deltas through a fresh [`Keeper`],
/// rejecting the trace on the first inadmissible delta (`CT-003` non-monotonic tick,
/// out-of-range needs, unknown action, or unentitled action poisoning).
pub fn adjudicate_tick_trace(actor: &str, deltas: &[Delta]) -> Result<Keeper, String> {
    if deltas.is_empty() {
        return Err("empty tick trace: at least birth delta required".into());
    }
    let mut keeper = Keeper::default();
    for (idx, d) in deltas.iter().enumerate() {
        if !(0.0..=1.0).contains(&d.h)
            || !(0.0..=1.0).contains(&d.r)
            || !(0.0..=1.0).contains(&d.s)
        {
            return Err(format!("tick trace[{idx}] at t={}: needs out of range", d.t));
        }
        match keeper.adjudicate(actor, d) {
            Verdict::Admitted => {}
            Verdict::Refused(reason) => {
                return Err(format!("tick trace[{idx}] at t={} refused: {reason}", d.t));
            }
        }
    }
    Ok(keeper)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gdd_parser::{DeterministicExpander, GddExpander};

    const SAMPLE_BRIEF: &str = "the painted forest at dawn, ZEN wired";

    #[test]
    fn gate_zone_tick_is_deterministic_and_verifies_receipt() {
        let manifest = DeterministicExpander.expand(SAMPLE_BRIEF).unwrap();
        let r1 = gate_zone_tick(&manifest, 0).expect("admissible at birth tick 0");
        let r2 = gate_zone_tick(&manifest, 0).expect("admissible on replay");
        assert_eq!(r1, r2, "same manifest + tick must yield identical receipt");
        assert!(r1.ct000_verified);
        assert_eq!(r1.seed001_action, "HoldAndMaintain");
        assert_eq!(r1.golden_hash.len(), 64);
        assert_eq!(r1.keeper_admitted, (manifest.archetypes.len() as u64) * 2);
        assert_eq!(r1.keeper_refused, manifest.archetypes.len() as u64);
        assert_eq!(r1.keeper_seq, (manifest.archetypes.len() as u64) * 3);
        assert_eq!(verify_receipt(&manifest, &r1), Ok(()));

        // Advancing the tick produces a distinct receipt with monotone GAIA state
        let r_later = gate_zone_tick(&manifest, 3600).expect("admissible at tick 3600");
        assert_ne!(r1.golden_hash, r_later.golden_hash);
        assert_eq!(verify_receipt(&manifest, &r_later), Ok(()));
    }

    #[test]
    fn gate_rejects_corrupted_ternary_board_and_forged_gaia() {
        let mut bad_board = DeterministicExpander.expand(SAMPLE_BRIEF).unwrap();
        bad_board.board[42] = 2; // outside {-1, 0, +1}
        let err = gate_zone_tick(&bad_board, 0).unwrap_err();
        assert!(
            err.contains("corrupted ternary board at tile 42"),
            "unexpected error: {err}"
        );

        let mut forged_gaia = DeterministicExpander.expand(SAMPLE_BRIEF).unwrap();
        forged_gaia.gaia.memory = "deadbeef".into();
        let err = gate_zone_tick(&forged_gaia, 0).unwrap_err();
        assert!(
            err.contains("manifest.gaia diverges"),
            "unexpected error: {err}"
        );

        let mut future_birth = DeterministicExpander.expand(SAMPLE_BRIEF).unwrap();
        future_birth.gaia = gaia_state(&future_birth.brief, 100);
        let err = gate_zone_tick(&future_birth, 50).unwrap_err();
        assert!(
            err.contains("precedes manifest birth tick 100 (CT-003)"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn verify_receipt_rejects_tampered_golden_hash_or_sphere_value() {
        let manifest = DeterministicExpander.expand(SAMPLE_BRIEF).unwrap();
        let mut receipt = gate_zone_tick(&manifest, 0).unwrap();
        receipt.sphere_value += 1;
        let err = verify_receipt(&manifest, &receipt).unwrap_err();
        assert!(err.contains("admissibility receipt mismatch"), "got: {err}");

        let mut unverified = gate_zone_tick(&manifest, 0).unwrap();
        unverified.ct000_verified = false;
        assert!(verify_receipt(&manifest, &unverified).is_err());
    }

    #[test]
    fn adjudicate_tick_trace_accepts_valid_and_rejects_corrupted_ticks() {
        let valid_trace = vec![
            Delta {
                t: 1,
                h: 0.25,
                r: 0.90,
                s: 0.90,
                action: None,
                asleep: false,
            },
            Delta {
                t: 2,
                h: 0.25,
                r: 0.90,
                s: 0.90,
                action: Some("forage the field".into()),
                asleep: false,
            },
            Delta {
                t: 3,
                h: 0.80,
                r: 0.20,
                s: 0.90,
                action: Some("sleep in the tent".into()),
                asleep: true,
            },
        ];
        let keeper = adjudicate_tick_trace("cube-wolf", &valid_trace).expect("valid trace");
        assert_eq!(keeper.admitted, 3);
        assert_eq!(keeper.refused, 0);

        // Non-monotonic tick replay (CT-003) must be rejected
        let mut replay_trace = valid_trace.clone();
        replay_trace[2].t = 2;
        let err = adjudicate_tick_trace("cube-wolf", &replay_trace).unwrap_err();
        assert!(err.contains("tick not monotonic"), "got: {err}");

        // Poison action (foraging while comfortable) must be rejected
        let mut poison_trace = valid_trace.clone();
        poison_trace[1].h = 0.90;
        let err = adjudicate_tick_trace("cube-wolf", &poison_trace).unwrap_err();
        assert!(err.contains("action without need"), "got: {err}");

        // Out-of-range needs at birth or continuation must be rejected
        let mut oob_trace = valid_trace;
        oob_trace[0].h = 1.5;
        let err = adjudicate_tick_trace("cube-wolf", &oob_trace).unwrap_err();
        assert!(err.contains("needs out of range"), "got: {err}");
    }
}
