//! Spherepop as a witness instance of the Central Theorem.
//!
//! `spherepop::Sphere` evaluates nested bubbles inside-out: only an unpopped
//! bubble with no unpopped children is in `Sphere::admissible()`. Popping
//! outside that frontier does not mutate the visible expression (`render()`),
//! yet appends a `Event::Refuse` entry to the history (`digest()`).
//!
//! This module wires `spherepop::Sphere` directly into `admissibility::Structure`:
//!
//! ```text
//! MANIFEST: CT-000 (frontier overlap structure witness),
//!           CT-000⁺ (coarse observation blurs which inner bubble popped),
//!           CT-015 (admissibility factors through render() despite distinct
//!                   pop-order or refusal histories),
//!           CT-016 (refusal owns a history position and shifts digest()
//!                   while leaving render() and admissible() unchanged).
//! ```

use std::collections::HashSet;
use std::hash::Hash;

use spherepop::{BubbleId, Sphere};

use crate::Structure;

/// Build a Central Theorem [`Structure`] over a finite family of [`Sphere`]
/// worlds using `observe` as the observation map and `Sphere::admissible()`
/// as each world's admissible-action set.
pub fn structure_from_spheres<Obs, F>(spheres: &[Sphere], mut observe: F) -> Structure<Obs, BubbleId>
where
    Obs: Eq + Hash + Clone + std::fmt::Debug,
    F: FnMut(&Sphere) -> Obs,
{
    let obs: Vec<Obs> = spheres.iter().map(&mut observe).collect();
    let adm: Vec<HashSet<BubbleId>> = spheres
        .iter()
        .map(|s| s.admissible().into_iter().collect())
        .collect();
    Structure::new(obs, adm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spherepop::{Event, Refusal};

    #[test]
    fn render_observation_satisfies_ct000_and_ct015_minimal_history() {
        // Four spheres over the same source `((1 + 2) + (3 + 4))`:
        // - w0: popped 1 then 2 (history len 2)
        // - w1: popped 2 then 1 (history len 2, different digest)
        // - w2: refused 0 first, then popped 1 and 2 (history len 3, CT-016)
        // - w3: fresh (neither child popped yet)
        let src = "((1 + 2) + (3 + 4))";

        let mut w0 = Sphere::parse(src).unwrap();
        assert_eq!(w0.pop(1), Ok(3));
        assert_eq!(w0.pop(2), Ok(7));

        let mut w1 = Sphere::parse(src).unwrap();
        assert_eq!(w1.pop(2), Ok(7));
        assert_eq!(w1.pop(1), Ok(3));

        let mut w2 = Sphere::parse(src).unwrap();
        assert_eq!(w2.pop(0), Err(Refusal::InnerFirst { blocking: 1 }));
        assert_eq!(w2.pop(1), Ok(3));
        assert_eq!(w2.pop(2), Ok(7));

        let w3 = Sphere::parse(src).unwrap();

        // History is identity: w0, w1, w2 have pairwise distinct digests.
        assert_ne!(w0.digest(), w1.digest());
        assert_ne!(w0.digest(), w2.digest());
        assert_ne!(w1.digest(), w2.digest());

        // Yet w0, w1, w2 project to the same visible render "(3 + 7)" and share
        // the exact same admissible frontier `{0}`.
        let worlds = [w0, w1, w2, w3];
        let st = structure_from_spheres(&worlds, |s| s.render());
        st.assert_ct000();
        assert_eq!(st.no_safe_action_witness(), None);

        // Deterministic guaranteeing policy picks bubble 0 for "(3 + 7)" and
        // the smallest innermost bubble 1 for "((1 + 2) + (3 + 4))".
        let policy = st.guaranteeing_policy().unwrap();
        assert_eq!(
            policy,
            vec![
                ("(3 + 7)".to_string(), 0),
                ("((1 + 2) + (3 + 4))".to_string(), 1),
            ]
        );

        // CT-015: admissibility factors through the visible render representative
        // `p = [0, 0, 0, 3]` even though w0, w1, w2 have distinct event histories.
        let p = [0, 0, 0, 3];
        for i in 0..worlds.len() {
            assert_eq!(st.adm[i], st.adm[p[i]]);
        }
        let projected = st.projected_through(&p);
        projected.assert_ct000();
        assert_eq!(st.guaranteeing_policy(), projected.guaranteeing_policy());
    }

    #[test]
    fn coarse_depth_observation_blurs_frontier_and_witnesses_ct000_plus() {
        // Two spheres over `((1 + 2) + (3 + 4))` where exactly one inner bubble
        // has popped:
        // - left_popped: bubble 1 popped -> admissible frontier is `{2}`
        // - right_popped: bubble 2 popped -> admissible frontier is `{1}`
        // Observing only "1 of 3 bubbles popped" blurs which child remains,
        // so the admissible sets `{2}` and `{1}` are disjoint under one observation.
        let src = "((1 + 2) + (3 + 4))";

        let mut left_popped = Sphere::parse(src).unwrap();
        assert_eq!(left_popped.pop(1), Ok(3));

        let mut right_popped = Sphere::parse(src).unwrap();
        assert_eq!(right_popped.pop(2), Ok(7));

        let worlds = [left_popped.clone(), right_popped.clone()];
        let coarse = structure_from_spheres(&worlds, |s| {
            s.bubbles().iter().filter(|b| b.is_popped()).count()
        });
        coarse.assert_ct000();
        assert_eq!(coarse.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(coarse.guaranteeing_policy(), None);

        // CT-016: if a policy blindly tries bubble 1 on `left_popped`, the
        // sphere refuses with `AlreadyPopped`, records the refusal in history,
        // and leaves `render()` untouched.
        let before_render = left_popped.render();
        let before_digest = left_popped.digest();
        assert_eq!(left_popped.pop(1), Err(Refusal::AlreadyPopped));
        assert_eq!(left_popped.render(), before_render);
        assert_ne!(left_popped.digest(), before_digest);
        assert_eq!(
            left_popped.history().last(),
            Some(&Event::Refuse {
                id: 1,
                reason: Refusal::AlreadyPopped,
            })
        );
    }
}
