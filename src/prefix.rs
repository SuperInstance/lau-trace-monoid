//! Trace prefix order, ideal structure, and extensions.
//!
//! The prefix order on traces: [u] ≤ [v] iff there exists [w] such that [v] = [u][w].
//! An ideal (downward-closed set) in this order represents a consistent concurrent state.

use crate::independence::Independence;
use crate::trace::Trace;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Check if trace t1 is a prefix of trace t2.
/// [u] ≤ [v] iff ∃ w: v ≡ uw (trace equivalence).
pub fn is_prefix<A: Eq + Hash + Clone + Ord + std::fmt::Debug>(
    t1: &Trace<A>,
    t2: &Trace<A>,
    ind: &Independence<A>,
) -> bool {
    let w1 = t1.canonical();
    let w2 = t2.canonical();
    if w1.len() > w2.len() {
        return false;
    }
    // Try all linearizations of t2 and check if any starts with a linearization of t1
    // More efficient: check if there's a prefix of w2's linearizations equivalent to t1
    // For correctness, use the suffix approach:
    if w1.is_empty() {
        return true;
    }
    // Find all linearizations of t1, check if any is a prefix of a linearization of t2
    let all_lin_t2 = crate::trace::all_linearizations(t2, ind);
    for lin in &all_lin_t2 {
        let prefix: Vec<A> = lin.iter().take(w1.len()).cloned().collect();
        if Trace::words_equivalent(&prefix, w1, ind) {
            return true;
        }
    }
    false
}

/// An ideal (downward-closed set) in the trace prefix order.
/// Represented as the set of traces it contains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ideal<A: Eq + Hash + Clone + Ord> {
    /// The maximal elements (generators) of this ideal.
    /// An ideal is determined by its maximal traces.
    generators: Vec<Trace<A>>,
}

impl<A: Eq + Hash + Clone + Ord + std::fmt::Debug> Ideal<A> {
    /// Create an ideal from generator traces.
    pub fn new(generators: Vec<Trace<A>>) -> Self {
        Ideal { generators }
    }

    /// The generators (maximal traces) of this ideal.
    pub fn generators(&self) -> &[Trace<A>] {
        &self.generators
    }

    /// The empty ideal (contains only the empty trace).
    pub fn empty() -> Self {
        Ideal { generators: vec![] }
    }

    /// Check if a trace is contained in this ideal.
    /// A trace t is in the ideal if it is a prefix of some generator.
    pub fn contains(&self, trace: &Trace<A>, ind: &Independence<A>) -> bool {
        if trace.is_empty() {
            return true;
        }
        self.generators.iter().any(|g| is_prefix(trace, g, ind))
    }

    /// Extend the ideal by adding a symbol to all possible positions.
    /// Returns a new ideal representing the state after executing the symbol.
    pub fn extend(&self, symbol: A, ind: &Independence<A>) -> Ideal<A> {
        if self.generators.is_empty() {
            let new_trace = Trace::new(vec![symbol], ind);
            return Ideal { generators: vec![new_trace] };
        }
        let mut new_gens = Vec::new();
        for gen in &self.generators {
            let mut extended = gen.canonical().to_vec();
            extended.push(symbol.clone());
            new_gens.push(Trace::new(extended, ind));
        }
        // Remove dominated generators
        let mut filtered = Vec::new();
        for i in 0..new_gens.len() {
            let dominated = (0..new_gens.len())
                .filter(|&j| j != i)
                .any(|j| is_prefix(&new_gens[i], &new_gens[j], ind));
            if !dominated {
                filtered.push(new_gens[i].clone());
            }
        }
        Ideal {
            generators: if filtered.is_empty() { new_gens } else { filtered },
        }
    }
}

/// Find all extensions of a trace by one symbol that produce distinct traces.
pub fn extensions<A: Eq + Hash + Clone + Ord>(
    trace: &Trace<A>,
    symbol: A,
    ind: &Independence<A>,
) -> Vec<Trace<A>> {
    let mut results = Vec::new();
    let word = trace.canonical().to_vec();
    // Try inserting symbol at every position
    for pos in 0..=word.len() {
        let mut extended = word.clone();
        extended.insert(pos, symbol.clone());
        let new_trace = Trace::new(extended, ind);
        // Only keep if it's genuinely new
        if !results.iter().any(|r: &Trace<A>| r.canonical() == new_trace.canonical()) {
            results.push(new_trace);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn make_ind() -> Independence<char> {
        let alpha: BTreeSet<char> = "abcd".chars().collect();
        Independence::new(alpha, vec![('a', 'b'), ('c', 'd')])
    }

    #[test]
    fn empty_is_prefix() {
        let ind = make_ind();
        let t1 = Trace::new(vec![], &ind);
        let t2 = Trace::new(vec!['a', 'b'], &ind);
        assert!(is_prefix(&t1, &t2, &ind));
    }

    #[test]
    fn self_prefix() {
        let ind = make_ind();
        let t = Trace::new(vec!['a', 'b'], &ind);
        assert!(is_prefix(&t, &t, &ind));
    }

    #[test]
    fn prefix_of_longer() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['a'], &ind);
        let t2 = Trace::new(vec!['a', 'c'], &ind);
        assert!(is_prefix(&t1, &t2, &ind));
    }

    #[test]
    fn non_prefix() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['c'], &ind);
        let t2 = Trace::new(vec!['a', 'b'], &ind);
        assert!(!is_prefix(&t1, &t2, &ind));
    }

    #[test]
    fn ideal_contains_empty() {
        let ind = make_ind();
        let ideal = Ideal::empty();
        let empty = Trace::new(vec![], &ind);
        assert!(ideal.contains(&empty, &ind));
    }

    #[test]
    fn ideal_extend() {
        let ind = make_ind();
        let ideal = Ideal::empty();
        let extended = ideal.extend('a', &ind);
        assert_eq!(extended.generators().len(), 1);
    }

    #[test]
    fn extensions_basic() {
        let ind = make_ind();
        let t = Trace::new(vec!['a'], &ind);
        let exts = extensions(&t, 'b', &ind);
        // a and b are independent, so all insertions give same trace
        assert_eq!(exts.len(), 1);
    }

    #[test]
    fn extensions_dependent() {
        let ind = make_ind();
        let t = Trace::new(vec!['a'], &ind);
        let exts = extensions(&t, 'c', &ind);
        // a and c are dependent, so position matters: [a,c] vs [c,a]
        assert_eq!(exts.len(), 2);
    }
}
