//! Seed 001 (`8b-is/8b-is-engine#1` · `p===peter+1`) — Computation Power,
//! Held + Maintained: Is the Maintenance the Wealth?
//!
//! Encodes the Seed 001 stock-vs-flow compute ledger as a concrete Central
//! Theorem (`CT-000` / `CT-000⁺`) instance:
//!
//! - **The Common Ledger (`FlowOnlyObs`)**: books compute power solely by
//!   instantaneous work output (`ActiveFlops` vs `ZeroFlops`). Under this
//!   observation map, a **Held & Maintained Sanctuary Reserve** (e.g., an M1
//!   Max or T4 held warm at 0 current FLOPs) is epistemically indistinguishable
//!   from **Unmaintained Hardware Decay** (0 FLOPs, no maintenance stream).
//!   Because their admissible continuations are disjoint, `guaranteeing_policy()`
//!   is `None` (`CT-000⁺` witness `(1, 3)`).
//! - **The Seed 001 / Kuznets Diff Ledger (`StockAndMaintenanceObs`)**: books
//!   both the held stock and the maintenance input flow (`power + cooling +
//!   attention`, plus the counterfactual foreclosed alternative). With the
//!   maintenance flow visible, every equivalence class has a non-empty common
//!   admissible continuation (`guaranteeing_policy().is_some()`).

use crate::Structure;
use std::collections::HashSet;

/// The four worlds of compute capacity across stock, maintenance flow, and work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputeWorld {
    /// World 0: Stock held, maintenance flow paid, actively executing work.
    HeldActiveSovereign = 0,
    /// World 1: Stock held, maintenance flow paid, currently idle (sovereign reserve / option value).
    HeldIdleReserve = 1,
    /// World 2: No stock held, metered cloud flow rented per token (ephemeral claim).
    RentedCloudEphemeral = 2,
    /// World 3: Stock physically present, but maintenance flow halted (bit-rot / uncooled decay).
    UnmaintainedDecay = 3,
}

/// Coarse observation that only measures instantaneous work output (FLOPs),
/// ignoring whether a stock is being maintained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlowOnlyObs {
    ActiveFlops,
    ZeroFlops,
}

/// Refined Seed 001 observation that records both held stock and the
/// maintenance flow (`power + cooling + attention`) that sustains it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StockAndMaintenanceObs {
    HeldMaintainedActive,
    HeldMaintainedReserve,
    RentedMeteredFlow,
    UnmaintainedStock,
}

/// Admissible capital/compute allocation continuations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AllocationAction {
    /// Sustain the maintenance stream (power, cooling, stewardship) and preserve sovereign readiness.
    HoldAndMaintain = 0,
    /// Dispatch a high-trust sovereign workload onto the held stock.
    DispatchSovereignWork = 1,
    /// Step down to low-draw standby while preserving state.
    StandbyLowDraw = 2,
    /// Write down / relinquish the unmaintained or ephemeral claim.
    RelinquishClaim = 3,
}

fn admissible_sets() -> Vec<HashSet<AllocationAction>> {
    use AllocationAction::*;
    vec![
        // World 0: HeldActiveSovereign
        [HoldAndMaintain, DispatchSovereignWork].into_iter().collect(),
        // World 1: HeldIdleReserve (maintenance IS the wealth / readiness)
        [HoldAndMaintain, StandbyLowDraw].into_iter().collect(),
        // World 2: RentedCloudEphemeral
        [StandbyLowDraw, RelinquishClaim].into_iter().collect(),
        // World 3: UnmaintainedDecay (cannot safely hold or dispatch without maintenance)
        [RelinquishClaim].into_iter().collect(),
    ]
}

/// The naive "work-only" ledger: blurs `HeldIdleReserve` (World 1) with
/// `UnmaintainedDecay` (World 3) because both emit `ZeroFlops` at `t`.
pub fn flow_only_ledger() -> Structure<FlowOnlyObs, AllocationAction> {
    use FlowOnlyObs::*;
    let obs = vec![
        ActiveFlops, // World 0: HeldActiveSovereign
        ZeroFlops,   // World 1: HeldIdleReserve
        ActiveFlops, // World 2: RentedCloudEphemeral
        ZeroFlops,   // World 3: UnmaintainedDecay
    ];
    Structure::new(obs, admissible_sets())
}

/// The Seed 001 ledger (`p===peter+1`): distinguishes held + maintained stock
/// from unmaintained stock and rented ephemeral flow.
pub fn seed001_maintenance_ledger() -> Structure<StockAndMaintenanceObs, AllocationAction> {
    use StockAndMaintenanceObs::*;
    let obs = vec![
        HeldMaintainedActive,  // World 0
        HeldMaintainedReserve, // World 1
        RentedMeteredFlow,     // World 2
        UnmaintainedStock,     // World 3
    ];
    Structure::new(obs, admissible_sets())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_only_ledger_blurs_idle_reserve_with_decay_and_has_no_safe_action() {
        let s = flow_only_ledger();
        assert!(
            s.guaranteeing_policy().is_none(),
            "FlowOnlyObs blurs HeldIdleReserve and UnmaintainedDecay (CT-000+)"
        );
        // World 0 (HeldActiveSovereign) and World 2 (RentedCloudEphemeral) both show ActiveFlops
        // and are disjoint; World 1 (HeldIdleReserve) and World 3 (UnmaintainedDecay) both show
        // ZeroFlops and are disjoint.
        let witness = s.no_safe_action_witness();
        assert!(witness.is_some());
    }

    #[test]
    fn seed001_maintenance_ledger_guarantees_admissible_policy() {
        let s = seed001_maintenance_ledger();
        let policy = s
            .guaranteeing_policy()
            .expect("Seed 001 Stock+Maintenance observation separates all 4 worlds");
        assert_eq!(policy.len(), 4);
        assert!(s.no_safe_action_witness().is_none());
        assert_eq!(
            policy[1],
            (
                StockAndMaintenanceObs::HeldMaintainedReserve,
                AllocationAction::HoldAndMaintain
            )
        );
    }
}
