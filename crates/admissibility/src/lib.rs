//! admissibility — the Central Theorem (Epistemic Overlap), as types and
//! invariants pinned by tests.
//!
//! A world is a geometry of admissible continuation. The theorem: a
//! deterministic policy can guarantee an admissible global action if and
//! only if all worlds epistemically indistinguishable under the available
//! observations share at least one common admissible action. If local
//! observations blur state distinctions necessary for global validity, no
//! safe action can be guaranteed.
//!
//! One source, three views — this is the operational Rust view. The
//! canonical claim set and formal statements live at
//! `standardgalactic/alphabet: central-theorem/` (branch `central-theorem`,
//! PR #13): `SPEC.md` (registry) and `theorem.tex` (monograph). The Lean
//! kernel view lands in `8b-is/claimshift: CentralTheorem.lean`
//! (PR #1).
//!
//! ```text
//! MANIFEST: CT-000 (epistemic overlap, both directions), CT-000⁺
//!           (no-safe-action corollary), CT-015 (minimal-history premise),
//!           CT-016 (commit enforcement vs. visible state — witness).
//! ```

pub mod keeper_instance;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;

/// A finite world-structure: the observation map and the admissible-action
/// map of the Central Theorem (CT-000), made concrete over index sets.
///
/// `obs[i]` is what world `i` looks like from the available observations;
/// `adm[i]` is the set of actions admissible in world `i` — continuations
/// of that world's attested history, independent of any valuation of the
/// outcome (CT-001).
#[derive(Debug, Clone)]
pub struct Structure<Obs, A>
where
    Obs: Clone,
    A: Clone,
{
    /// the available observation map, one entry per world
    pub obs: Vec<Obs>,
    /// the admissible-action map, one set per world
    pub adm: Vec<HashSet<A>>,
}

impl<Obs, A> Structure<Obs, A>
where
    Obs: Eq + Hash + Clone + std::fmt::Debug,
    A: Eq + Hash + Clone + PartialOrd + std::fmt::Debug,
{
    /// build — the two maps must agree on the world count
    pub fn new(obs: Vec<Obs>, adm: Vec<HashSet<A>>) -> Self {
        assert_eq!(
            obs.len(),
            adm.len(),
            "the observation map and the admissible-action map must cover the same worlds"
        );
        Self { obs, adm }
    }

    /// the number of worlds
    pub fn worlds(&self) -> usize {
        self.obs.len()
    }

    /// CT-000 — the epistemic overlap class of world `i`: every world
    /// indistinguishable from it under the available observations.
    pub fn class_of(&self, i: usize) -> Vec<usize> {
        let o = &self.obs[i];
        (0..self.obs.len())
            .filter(|&j| &self.obs[j] == o)
            .collect()
    }

    /// CT-000 — the admissible actions shared by the entire class of `i`
    /// (the intersection over all epistemically indistinguishable worlds),
    /// in `PartialOrd` order. Empty means the class carries no common
    /// admissible action.
    pub fn common_admissible(&self, i: usize) -> Vec<A> {
        let class = self.class_of(i);
        let mut common: HashSet<A> = self.adm[class[0]].clone();
        for &j in &class[1..] {
            common.retain(|a| self.adm[j].contains(a));
        }
        let mut out: Vec<A> = common.into_iter().collect();
        out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        out
    }

    /// CT-000 (⇒) with the No-Safe-Action corollary (CT-000⁺) pinned in one
    /// call: a guaranteeing policy exists if and only if every observation
    /// class has a common admissible action. Deterministic: the smallest
    /// common action per class. `None` is the corollary — some class blurs
    /// an admissibility-relevant distinction, and no policy can guarantee.
    pub fn guaranteeing_policy(&self) -> Option<Vec<(Obs, A)>> {
        let mut out: Vec<(Obs, A)> = Vec::new();
        let mut seen: HashSet<Obs> = HashSet::new();
        for i in 0..self.obs.len() {
            if !seen.insert(self.obs[i].clone()) {
                continue;
            }
            // the smallest common admissible action of the class; a class
            // with an empty intersection aborts the guarantee (CT-000⁺)
            let common = self.common_admissible(i);
            let a = common
                .into_iter()
                .next()?;
            out.push((self.obs[i].clone(), a));
        }
        Some(out)
    }

    /// CT-000⁺ — a witness for the no-safe-action corollary: the first pair
    /// `(i, j)` of epistemically indistinguishable worlds whose admissible
    /// sets are disjoint. `None` means every class overlaps — the premise
    /// of the theorem holds.
    pub fn no_safe_action_witness(&self) -> Option<(usize, usize)> {
        for i in 0..self.obs.len() {
            for j in (i + 1)..self.obs.len() {
                if self.obs[i] == self.obs[j]
                    && self.adm[i].intersection(&self.adm[j]).next().is_none()
                {
                    return Some((i, j));
                }
            }
        }
        None
    }

    /// the Central Theorem, pinned as an invariant rather than a hope:
    /// a guaranteeing policy exists exactly when no blurred class exists,
    /// and the policy's action is a member of every world's admissible set
    /// in its class. Panics with a readable message on violation.
    pub fn assert_ct000(&self) {
        let guarantee = self.guaranteeing_policy().is_some();
        let no_blur = self.no_safe_action_witness().is_none();
        assert_eq!(
            guarantee, no_blur,
            "CT-000 broken: guaranteeing policy {} but blurred class {}",
            if guarantee { "exists" } else { "absent" },
            if no_blur { "absent" } else { "present" }
        );
        if let Some(policy) = self.guaranteeing_policy() {
            for (o, a) in &policy {
                for j in 0..self.obs.len() {
                    if self.obs[j] == *o {
                        assert!(
                            self.adm[j].contains(a),
                            "CT-000 broken: policy action {a:?} is not admissible in world {j} of its class"
                        );
                    }
                }
            }
        }
    }

    /// CT-015 — the minimal-history premise, made operational: the same
    /// structure with world `i`'s admissible set read from its prefix
    /// representative `p[i]`. When admissibility actually factors through
    /// the prefix — `∀i, adm[i] == adm[p[i]]` — the guarantee property is
    /// unchanged: the decision reads only the minimal decision-relevant
    /// distinction, not the full history.
    pub fn projected_through(&self, p: &[usize]) -> Self {
        assert_eq!(p.len(), self.adm.len(), "the projection must cover every world");
        Self {
            obs: self.obs.clone(),
            adm: p.iter().map(|&i| self.adm[i].clone()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type S = Structure<usize, char>;

    fn s(obs: Vec<usize>, adm: Vec<Vec<char>>) -> S {
        S::new(obs, adm.into_iter().map(|v| v.into_iter().collect()).collect())
    }

    #[test]
    fn the_invariant_holds_on_every_structure_shape() {
        // overlapping classes
        let ok = s(vec![0, 0, 1, 1], vec![vec!['a', 'b'], vec!['a', 'c'], vec!['x'], vec!['x', 'y']]);
        ok.assert_ct000();
        // a blurred class (disjoint adm under one observation)
        let blur = s(vec![0, 0, 1], vec![vec!['a'], vec!['b'], vec!['x']]);
        blur.assert_ct000();
        // singletons
        let one = s(vec![7], vec![vec!['z']]);
        one.assert_ct000();
    }

    #[test]
    fn a_guarantee_exists_exactly_when_every_class_overlaps() {
        let ok = s(vec![0, 0, 1, 1], vec![vec!['a', 'b'], vec!['a', 'c'], vec!['x'], vec!['x', 'y']]);
        assert!(ok.no_safe_action_witness().is_none());
        let p = ok.guaranteeing_policy().unwrap();
        // deterministic: the smallest common action per class
        assert_eq!(p, vec![(0, 'a'), (1, 'x')]);

        let blur = s(vec![0, 0, 1], vec![vec!['a'], vec!['b'], vec!['x']]);
        assert_eq!(blur.no_safe_action_witness(), Some((0, 1)));
        assert_eq!(blur.guaranteeing_policy(), None);
    }

    #[test]
    fn the_class_and_the_intersection_are_read_correctly() {
        let st = s(vec![0, 0, 0, 1], vec![vec!['a', 'b'], vec!['b', 'c'], vec!['a', 'b'], vec!['q']]);
        assert_eq!(st.class_of(1), vec![0, 1, 2]);
        assert_eq!(st.common_admissible(1), vec!['b']);
        assert_eq!(st.common_admissible(3), vec!['q']);
    }

    #[test]
    fn the_prefix_projection_keeps_the_guarantee_when_admissibility_factors() {
        // CT-015: admissibility read through the prefix representative.
        // worlds 0..3 share two attested prefixes (needs); p maps each
        // world to the first world of its prefix
        let st = s(
            vec![0, 0, 1, 1],
            vec![vec!['a', 'b'], vec!['a', 'b'], vec!['x', 'y'], vec!['x', 'y']],
        );
        let p = vec![0, 0, 2, 2];
        // the premise: admissibility actually factors through the prefix
        for i in 0..4 {
            assert_eq!(st.adm[i], st.adm[p[i]]);
        }
        let proj = st.projected_through(&p);
        assert_eq!(
            st.guaranteeing_policy().is_some(),
            proj.guaranteeing_policy().is_some()
        );
        // and the projection that IS the factorization reproduces the same
        // per-class choices
        assert_eq!(st.guaranteeing_policy(), proj.guaranteeing_policy());

        // the documented flip: a projection that REWRITES the admissible
        // sets is not a premise — it can manufacture an overlap that does
        // not exist. The guarantee is read through the factorization, not
        // through a mercy.
        let blur = s(vec![0, 0], vec![vec!['a'], vec!['b']]);
        assert_eq!(blur.guaranteeing_policy(), None);
        let merciful = blur.projected_through(&[0, 0]);
        assert_eq!(merciful.adm[1], merciful.adm[0]);
        assert_eq!(merciful.guaranteeing_policy(), Some(vec![(0, 'a')]));
    }
}
