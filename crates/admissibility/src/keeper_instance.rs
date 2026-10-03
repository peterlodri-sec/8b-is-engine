//! The zone keeper as witness instance of the Central Theorem.
//!
//! `world_core::fold::Keeper` is the single-zone operational witness:
//! an event is admissible only if it can be a continuation of the actor's
//! attested history. This module pins that the keeper's admission kernel
//! agrees, bit for bit, with the theorem's admissible-action structure —
//! and that a refusal is a committed record (CT-016), not an invisible
//! event.
//!
//! ```text
//! MANIFEST: CT-000 (structure witness), CT-003 (commitment collapse —
//!           tick monotonicity, witness), CT-016 (durable refusal owns a
//!           ledger position).
//! ```
//!
//! Guardrail: the keeper's semantics are bit-exact by construction — the
//! vocabulary, thresholds, and refusal reasons below are copied from
//! `world-core/src/fold.rs`. Nothing in this module feeds back into the
//! keeper; the agreement is asserted in tests, so a semantic drift on
//! either side fails CI.

use world_core::fold::{Delta, Keeper, Verdict};

/// The keeper's action vocabulary, keyed to its need type — bit-exact to
/// `fold.rs` `ACTIONS` ("forage the field" ↔ h, "sleep in the tent" ↔ r,
/// "flee to the gate" ↔ s).
pub const ACTIONS: [(&'static str, char); 3] = [
    ("forage the field", 'h'),
    ("sleep in the tent", 'r'),
    ("flee to the gate", 's'),
];

/// The need threshold at/below which the action is entitled — bit-exact to
/// `fold.rs` `THRESH` (h: 0.35, r: 0.30, s: 0.40). The keeper refuses when
/// the attested need is strictly above the threshold; admissible is the
/// complement: the attested need is at or below it.
pub const THRESH: [(char, f64); 3] = [('h', 0.35), ('r', 0.30), ('s', 0.40)];

/// CT-000, one actor's attested world: the actions entitled by the
/// attested needs `(h, r, s)`. Mirrors the keeper's pure admissibility
/// check exactly:
///
/// 1. the attested needs are in range (all of `[0, 1]`);
/// 2. the action is in the three-action vocabulary;
/// 3. the need the action draws on is attested at or below its threshold —
///    acting while attesting comfort is poisoning, and is refused.
///
/// Tick monotonicity (CT-003) is a separate channel: it is a property of
/// the (actor, tick) sequence, not of the attested world, and is witnessed
/// against the real keeper in the tests below.
pub fn admissible_actions(h: f64, r: f64, s: f64) -> Vec<&'static str> {
    let in_range = (0.0..=1.0).contains(&h) && (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&s);
    if !in_range {
        return Vec::new();
    }
    let attested = |n: char| match n {
        'h' => h,
        'r' => r,
        _ => s,
    };
    ACTIONS
        .iter()
        .filter(|(_action, need)| {
            let threshold = THRESH
                .iter()
                .find(|(n, _)| n == need)
                .map(|(_, v)| *v)
                .unwrap();
            attested(*need) <= threshold
        })
        .map(|(action, _)| *action)
        .collect()
}

/// one delta, at the keeper's wire: attested needs + an optional action
fn delta(t: u64, h: f64, r: f64, s: f64, action: Option<&str>) -> Delta {
    Delta {
        t,
        h,
        r,
        s,
        action: action.map(str::to_string),
        asleep: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the attested-need grid, with the thresholds bracketed: 0.34/0.36
    /// straddle the 0.35 forage line, 0.28/0.32 straddle the 0.30 sleep
    /// line, 0.38/0.42 straddle the 0.40 flee line
    const NEEDS: [f64; 8] = [0.0, 0.1, 0.2, 0.3, 0.34, 0.36, 0.5, 0.9];

    #[test]
    fn the_keeper_matches_the_ct000_structure() {
        // For every attested world in range and every action in the
        // vocabulary: a fresh keeper births the actor at that world, then
        // adjudicates the action at the next tick — admitted if and only
        // if the theorem's structure says the action is admissible there.
        // 8^3 = 512 worlds × 3 actions = 1536 adjudications, each on a
        // fresh keeper so the attested world is exactly the grid point.
        for h in NEEDS {
            for r in NEEDS {
                for s in NEEDS {
                    let expected = admissible_actions(h, r, s);
                    for &(action, _) in ACTIONS.iter() {
                        let mut k = Keeper::default();
                        // birth — the first attested state (always
                        // admitted; the actor's attestation of existence)
                        assert_eq!(
                            k.adjudicate("actor", &delta(1, h, r, s, None)),
                            Verdict::Admitted,
                            "birth at ({h}, {r}, {s}) must be admitted"
                        );
                        let verdict = k.adjudicate("actor", &delta(2, h, r, s, Some(action)));
                        let entitled = expected.iter().any(|a| a == &action);
                        match (entitled, &verdict) {
                            (true, Verdict::Admitted) => {}
                            (false, Verdict::Refused(reason)) => {
                                assert!(
                                    matches!(*reason, "action without need" | "needs out of range"),
                                    "unexpected refusal {reason:?} at ({h}, {r}, {s}) / {action}"
                                );
                            }
                            other => panic!(
                                "keeper/structure drift at ({h}, {r}, {s}) / {action}: {other:?}"
                            ),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn an_out_of_range_world_admits_nothing() {
        // out of range: the structure admits no action, and the keeper
        // refuses every action at the next tick with the range reason
        for &(h, r, s) in &[(-0.1, 0.5, 0.5), (0.5, 1.2, 0.5), (0.5, 0.5, 2.0)] {
            assert!(admissible_actions(h, r, s).is_empty());
            for &(action, _) in ACTIONS.iter() {
                let mut k = Keeper::default();
                // the keeper admits the birth even out of range — birth is
                // the attestation of existence; range only governs the
                // continuation
                assert_eq!(k.adjudicate("actor", &delta(1, h, r, s, None)), Verdict::Admitted);
                assert_eq!(
                    k.adjudicate("actor", &delta(2, h, r, s, Some(action))),
                    Verdict::Refused("needs out of range")
                );
            }
        }
    }

    #[test]
    fn tick_monotonicity_is_the_ct003_channel() {
        // commitment collapse, witnessed: a replayed tick cannot continue
        // the attested history — it is refused durable, and the world does
        // not move
        let mut k = Keeper::default();
        assert_eq!(k.adjudicate("actor", &delta(10, 0.2, 0.9, 0.9, None)), Verdict::Admitted);
        assert_eq!(
            k.adjudicate("actor", &delta(10, 0.2, 0.9, 0.9, None)),
            Verdict::Refused("tick not monotonic — cannot continue its past")
        );
        assert_eq!(k.zone["actor"], [0.2, 0.9, 0.9]);
        assert_eq!(k.last_tick["actor"], 10);
        assert_eq!(k.admitted, 1);
        assert_eq!(k.refused, 1);
    }

    #[test]
    fn a_refusal_owns_a_ledger_position() {
        // CT-016 — commit enforcement vs. visible state, at the keeper
        // level: a poison input (foraging while attesting comfort) is
        // refused, yet it still owns the next ledger position. The
        // material fold did not move; the record did.
        let mut k = Keeper::default();
        // birth with a hungry need — forage would be entitled, but the
        // poison delta re-attests comfort
        assert_eq!(
            k.adjudicate("actor", &delta(1, 0.2, 0.9, 0.9, None)),
            Verdict::Admitted
        );
        assert_eq!(k.seq, 1);

        // the poison: attested h = 0.9 > 0.35 while foraging
        assert_eq!(
            k.adjudicate("actor", &delta(2, 0.9, 0.9, 0.9, Some("forage the field"))),
            Verdict::Refused("action without need")
        );
        assert_eq!(k.seq, 2, "a refusal owns a ledger position");
        assert_eq!(k.refused, 1);
        assert_eq!(k.admitted, 1);
        // the material fold did not move: the zone still carries the
        // birth attestation, not the poison's
        assert_eq!(k.zone["actor"], [0.2, 0.9, 0.9]);
        assert_eq!(k.last_tick["actor"], 1);

        // the next admission continues from where the log stood
        assert_eq!(
            k.adjudicate("actor", &delta(3, 0.2, 0.9, 0.9, Some("forage the field"))),
            Verdict::Admitted
        );
        assert_eq!(k.seq, 3);
        assert_eq!(k.zone["actor"], [0.2, 0.9, 0.9]);
    }
}
