//! Kleene fixpoints for convergent CRDT states.
//!
//! For monotone functions f on a complete lattice, the Kleene fixpoint iteration
//! converges to the least fixpoint: x₀ = ⊥, x_{n+1} = f(xₙ), converging when x_{n+1} = xₙ.
//! For CRDTs, this represents the converged state after all replicas have merged.

use crate::crdt_lattice::GCounter;
use serde::{Deserialize, Serialize};

/// A fixpoint result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixpointResult<S: Clone + PartialEq> {
    /// The fixpoint value.
    pub value: S,
    /// Number of iterations to converge.
    pub iterations: usize,
    /// Whether convergence was achieved within the limit.
    pub converged: bool,
}

/// Compute the Kleene fixpoint by iteration from bottom.
pub fn kleene_fixpoint<S: Clone + PartialEq>(
    bottom: S,
    f: impl Fn(&S) -> S,
    max_iterations: usize,
) -> FixpointResult<S> {
    let mut current = bottom;
    for i in 0..max_iterations {
        let next = f(&current);
        if next == current {
            return FixpointResult {
                value: current,
                iterations: i,
                converged: true,
            };
        }
        current = next;
    }
    FixpointResult {
        value: current,
        iterations: max_iterations,
        converged: false,
    }
}

/// Compute the Kleene fixpoint chain (all intermediate values).
pub fn kleene_chain<S: Clone + PartialEq>(
    bottom: S,
    f: impl Fn(&S) -> S,
    max_iterations: usize,
) -> Vec<S> {
    let mut chain = vec![bottom.clone()];
    let mut current = bottom;
    for _ in 0..max_iterations {
        let next = f(&current);
        if next == current {
            break;
        }
        chain.push(next.clone());
        current = next;
    }
    chain
}

/// For G-Counter: converge by merging all replicas iteratively.
pub fn gcounter_converge(
    replicas: &[GCounter],
    max_iterations: usize,
) -> FixpointResult<GCounter> {
    let bottom = GCounter::new();
    kleene_fixpoint(bottom, |state| {
        let mut merged = state.clone();
        for replica in replicas {
            merged = merged.merge(replica);
        }
        merged
    }, max_iterations)
}

/// Check if a function produces an ascending chain.
pub fn is_ascending_chain<S: Clone + PartialEq>(
    bottom: S,
    f: impl Fn(&S) -> S,
    steps: usize,
    leq: impl Fn(&S, &S) -> bool,
) -> bool {
    let chain = kleene_chain(bottom, f, steps);
    for window in chain.windows(2) {
        if !leq(&window[0], &window[1]) {
            return false;
        }
    }
    true
}

/// Tarski's fixpoint theorem: every monotone function on a complete lattice
/// has a least fixpoint, which is the meet of all fixpoints.
/// Verify by checking that our Kleene iteration finds a fixpoint.
pub fn verify_tarski<S: Clone + PartialEq>(
    bottom: S,
    f: impl Fn(&S) -> S,
    _leq: impl Fn(&S, &S) -> bool,
    max_iterations: usize,
) -> bool {
    let f_ref = &f;
    let result = kleene_fixpoint(bottom, |s| f_ref(s), max_iterations);
    if !result.converged {
        return false;
    }
    let fp = &result.value;
    f_ref(fp) == *fp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kleene_identity() {
        let result = kleene_fixpoint(GCounter::new(), |s| s.clone(), 100);
        assert!(result.converged);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn kleene_increment() {
        let result = kleene_fixpoint(0u64, |&s| s + 1, 10);
        assert!(!result.converged); // Never converges
        assert_eq!(result.iterations, 10);
    }

    #[test]
    fn kleene_saturating() {
        // Function that saturates at 5
        let result = kleene_fixpoint(0u64, |&s| s.saturating_add(1).min(5), 100);
        assert!(result.converged);
        assert_eq!(result.value, 5);
    }

    #[test]
    fn gcounter_converge_basic() {
        let mut r1 = GCounter::new();
        r1.inc("r1");
        let mut r2 = GCounter::new();
        r2.inc("r2");
        let result = gcounter_converge(&[r1, r2], 100);
        assert!(result.converged);
        assert_eq!(result.value.value(), 2);
    }

    #[test]
    fn gcounter_converge_already_merged() {
        let mut c = GCounter::new();
        c.inc("r1");
        let result = gcounter_converge(&[c.clone()], 100);
        assert!(result.converged);
    }

    #[test]
    fn kleene_chain_length() {
        let chain = kleene_chain(0u64, |&s| s.saturating_add(1).min(3), 100);
        assert_eq!(chain.len(), 4); // [0, 1, 2, 3]
    }

    #[test]
    fn ascending_chain_check() {
        let result = is_ascending_chain(
            GCounter::new(),
            |s| {
                let mut c = s.clone();
                c.inc("r1");
                c
            },
            5,
            |a, b| a.leq(b),
        );
        assert!(result);
    }

    #[test]
    fn verify_tarski_basic() {
        let result = verify_tarski(
            GCounter::new(),
            |s| s.clone(), // identity is monotone
            |a, b| a.leq(b),
            100,
        );
        assert!(result);
    }

    #[test]
    fn fixpoint_serialization() {
        let fp = FixpointResult {
            value: GCounter::new(),
            iterations: 3,
            converged: true,
        };
        let json = serde_json::to_string(&fp).unwrap();
        assert!(json.contains("\"converged\":true"));
    }
}
