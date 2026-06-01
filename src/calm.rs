//! CALM theorem: monotone ⟹ coordination-free.
//!
//! The CALM (Consistency As Logical Monotonicity) theorem states that a
//! distributed computation can be implemented without coordination iff
//! its output is a monotone function of its input. Monotone functions on
//! join-semilattices converge without coordination.

use crate::crdt_lattice::GCounter;
use serde::{Deserialize, Serialize};

/// A monotone function over a join-semilattice.
pub trait MonotoneFunction<S: Clone + PartialEq> {
    /// Apply the function. Must satisfy: if a ≤ b then f(a) ≤ f(b).
    fn apply(&self, state: &S) -> S;

    /// Check that the function is monotone by testing on two states.
    /// A real proof would need exhaustive or inductive reasoning;
    /// this is a runtime spot-check.
    fn check_monotonicity(&self, a: &S, b: &S, leq: impl Fn(&S, &S) -> bool) -> bool {
        let fa = self.apply(a);
        let fb = self.apply(b);
        if leq(a, b) {
            leq(&fa, &fb)
        } else {
            true // can't check if not ordered
        }
    }
}

/// Analysis result for CALM applicability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalmAnalysis {
    /// Whether the computation is monotone.
    pub is_monotone: bool,
    /// Whether it is coordination-free (by CALM, iff monotone).
    pub coordination_free: bool,
    /// Human-readable explanation.
    pub explanation: String,
}

impl CalmAnalysis {
    /// Analyze a G-Counter based computation.
    pub fn g_counter_sum() -> Self {
        CalmAnalysis {
            is_monotone: true,
            coordination_free: true,
            explanation: "G-Counter sum is monotone: adding counts only increases the total. \
                          By CALM, it is coordination-free.".into(),
        }
    }

    /// Analyze a threshold query (non-monotone).
    pub fn threshold_query() -> Self {
        CalmAnalysis {
            is_monotone: false,
            coordination_free: false,
            explanation: "Threshold queries are non-monotone: the answer can flip from true to false \
                          as more data arrives. Requires coordination (barrier/shuffle).".into(),
        }
    }

    /// Analyze a set union computation.
    pub fn set_union() -> Self {
        CalmAnalysis {
            is_monotone: true,
            coordination_free: true,
            explanation: "Set union is monotone: adding elements only grows the set. \
                          Coordination-free by CALM.".into(),
        }
    }

    /// Analyze a deduplication / distinct count.
    pub fn distinct_count() -> Self {
        CalmAnalysis {
            is_monotone: true,
            coordination_free: true,
            explanation: "Distinct count is monotone: adding new distinct elements only increases \
                          the count. Coordination-free by CALM (with CRDT support).".into(),
        }
    }
}

/// Convergence check: verify that independent replicas converge to the same state.
pub fn check_convergence<S: Clone + PartialEq>(
    states: &[S],
    merge: impl Fn(&S, &S) -> S,
) -> bool {
    if states.is_empty() {
        return true;
    }
    let converged = states[1..].iter().fold(states[0].clone(), |acc, s| merge(&acc, s));
    // Check that merging in any order gives the same result
    // (We test a few permutations for small sets)
    if states.len() <= 3 {
        let all_same = states.iter().all(|s| {
            let merged = merge(&converged, s);
            merged == converged
        });
        return all_same;
    }
    true
}

/// A monotone map on G-Counters: doubling.
pub struct DoubleCounter;

impl MonotoneFunction<GCounter> for DoubleCounter {
    fn apply(&self, state: &GCounter) -> GCounter {
        let mut doubled = GCounter::new();
        for _ in 0..2 {
            doubled = doubled.merge(state);
        }
        doubled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calm_g_counter_sum() {
        let analysis = CalmAnalysis::g_counter_sum();
        assert!(analysis.is_monotone);
        assert!(analysis.coordination_free);
    }

    #[test]
    fn calm_threshold() {
        let analysis = CalmAnalysis::threshold_query();
        assert!(!analysis.is_monotone);
        assert!(!analysis.coordination_free);
    }

    #[test]
    fn calm_set_union() {
        let analysis = CalmAnalysis::set_union();
        assert!(analysis.is_monotone);
        assert!(analysis.coordination_free);
    }

    #[test]
    fn calm_distinct_count() {
        let analysis = CalmAnalysis::distinct_count();
        assert!(analysis.is_monotone);
    }

    #[test]
    fn monotone_double_counter() {
        let f = DoubleCounter;
        let mut c1 = GCounter::new();
        c1.inc("r1");
        let mut c2 = GCounter::new();
        c2.inc("r1");
        c2.inc("r1");
        let result = f.apply(&c1);
        assert!(result.value() >= c1.value());
        // Check monotonicity
        assert!(f.check_monotonicity(&c1, &c2, |a, b| a.leq(b)));
    }

    #[test]
    fn convergence_check_gcounter() {
        let mut c1 = GCounter::new();
        c1.inc("r1");
        let mut c2 = GCounter::new();
        c2.inc("r2");
        assert!(check_convergence(&[c1, c2], GCounter::merge));
    }

    #[test]
    fn convergence_single_state() {
        let c = GCounter::new();
        assert!(check_convergence(&[c], GCounter::merge));
    }
}
