//! multi-client — the Central Theorem at multi-agent scale, the E2E gap.
//!
//! Every pre-existing test in the workspace is single-zone, mostly
//! single-actor. These pin what the theorem says at the scale where it
//! lives: several clients under one observation map — indistinguishable
//! clients share their admissible set, a blurred observation admits no
//! safe action, and the guarantee reads through the attested prefix
//! (minimal history), not the full trajectory.
//!
//! Structured level by design: the wire and the tick loop are already
//! pinned by the mesh-node tests; here the epistemic structure is the
//! system under test.

use admissibility::keeper_instance::admissible_actions;
use admissibility::Structure;
use std::collections::HashSet;

type Needs = (f64, f64, f64);

/// one client's theorem structure: the observation is the attested needs
/// (what the agent can show the keeper), the admissible set is what the
/// attested world entitles
fn client_structure(obs: &[Needs]) -> Structure<String, String> {
    let obs_v = obs
        .iter()
        .map(|(h, r, s)| format!("needs({h}, {r}, {s})"))
        .collect();
    let adm_v = obs
        .iter()
        .map(|(h, r, s)| {
            admissible_actions(*h, *r, *s)
                .into_iter()
                .map(str::to_string)
                .collect::<HashSet<_>>()
        })
        .collect();
    Structure::new(obs_v, adm_v)
}

/// CT-000 premise exercised on real keeper data: two clients whose
/// attested needs are byte-identical are epistemically indistinguishable,
/// and they share exactly the same admissible-action set.
#[test]
fn indistinguishable_clients_share_their_admissible_set() {
    let a: Needs = (0.2, 0.9, 0.9); // hungry, rested, settled
    let b: Needs = a;

    let st = client_structure(&[a, b]);
    assert_eq!(st.class_of(0), vec![0, 1]);
    assert_eq!(st.common_admissible(0), st.common_admissible(1));

    // the shared set is the keeper's own verdict set for that world —
    // forage entitled (h ≤ 0.35), sleep and flee not (r, s above theirs)
    let shared: Vec<&str> = admissible_actions(0.2, 0.9, 0.9);
    assert_eq!(shared, vec!["forage the field"]);
    assert_eq!(
        st.common_admissible(0),
        vec!["forage the field".to_string()]
    );
    st.assert_ct000();
    assert_eq!(
        st.guaranteeing_policy(),
        Some(vec![(
            "needs(0.2, 0.9, 0.9)".to_string(),
            "forage the field".to_string()
        )])
    );

    // drift one client — the observation classes split, and each keeps
    // its own set
    let c: Needs = (0.9, 0.9, 0.9);
    let st2 = client_structure(&[a, c]);
    assert_eq!(st2.class_of(0), vec![0]);
    assert_eq!(st2.class_of(1), vec![1]);
    assert_eq!(st2.common_admissible(0), vec!["forage the field".to_string()]);
    assert!(st2.common_admissible(1).is_empty());
}

/// CT-000⁺ — the no-safe-action corollary: two worlds the observation
/// cannot tell apart, whose admissible sets are disjoint, admit no
/// policy that is admissible in both.
#[test]
fn a_blurred_observation_admits_no_safe_action() {
    // hand-built: the observation blurs a distinction that
    // admissibility needs — world 0 entitles only 'a', world 1 only 'b'
    let st = Structure::new(
        vec!["blur".to_string(), "blur".to_string(), "clear".to_string()],
        vec![
            HashSet::from(["a".to_string()]),
            HashSet::from(["b".to_string()]),
            HashSet::from(["x".to_string()]),
        ],
    );
    assert_eq!(st.no_safe_action_witness(), Some((0, 1)));
    assert_eq!(st.guaranteeing_policy(), None, "no safe action can be guaranteed");
    st.assert_ct000();
}

/// CT-000 (⇒) — when every class overlaps, the guaranteeing policy
/// exists, picks the smallest common action per class, and that action is
/// a member of every world's admissible set in the class.
#[test]
fn the_policy_guarantees_when_every_class_overlaps() {
    let st = Structure::new(
        vec![
            "one".to_string(),
            "one".to_string(),
            "one".to_string(),
            "two".to_string(),
            "two".to_string(),
        ],
        vec![
            HashSet::from(["b".to_string(), "a".to_string()]),
            HashSet::from(["a".to_string(), "c".to_string()]),
            HashSet::from(["a".to_string(), "b".to_string()]),
            HashSet::from(["y".to_string()]),
            HashSet::from(["z".to_string(), "y".to_string()]),
        ],
    );
    assert_eq!(st.no_safe_action_witness(), None);
    let p = st.guaranteeing_policy().unwrap();
    assert_eq!(
        p,
        vec![("one".to_string(), "a".to_string()), ("two".to_string(), "y".to_string())]
    );
    // the chosen action is admissible in every world of its class
    for (o, a) in &p {
        for i in 0..st.worlds() {
            if st.obs[i] == *o {
                assert!(st.adm[i].contains(a));
            }
        }
    }
    st.assert_ct000();
}

/// CT-015 at client scale — the minimal-history premise: when admissibility
/// reads only from the attested prefix (the needs), the guarantee computed
/// on the full client cast equals the guarantee computed through the
/// prefix projection.
#[test]
fn the_guarantee_reads_through_the_attested_prefix() {
    // four clients under two attested prefixes; the observation also
    // carries the tick (full trajectory view), but the admissible set is
    // a function of the needs alone
    let cast: Vec<Needs> = vec![
        (0.2, 0.9, 0.9), // prefix P0 — forage entitled
        (0.9, 0.1, 0.9), // prefix P1 — sleep entitled
        (0.2, 0.9, 0.9),
        (0.9, 0.1, 0.9),
    ];
    let obs: Vec<String> = cast
        .iter()
        .enumerate()
        .map(|(i, (h, r, s))| format!("tick-{i}: needs({h}, {r}, {s})"))
        .collect();
    let adm: Vec<HashSet<String>> = cast
        .iter()
        .map(|(h, r, s)| {
            admissible_actions(*h, *r, *s)
                .into_iter()
                .map(str::to_string)
                .collect()
        })
        .collect();
    let st = Structure::new(obs, adm);

    // the prefix map: each client points at the first client of its
    // attested prefix
    let p = vec![0, 1, 0, 1];
    for i in 0..4 {
        assert_eq!(st.adm[i], st.adm[p[i]], "the premise: admissibility factors through the prefix");
    }

    let through_prefix = st.projected_through(&p);
    assert_eq!(
        st.guaranteeing_policy().is_some(),
        through_prefix.guaranteeing_policy().is_some()
    );
    st.assert_ct000();
    through_prefix.assert_ct000();
}
