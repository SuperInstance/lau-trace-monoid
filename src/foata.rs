//! Foata normal form / Hasse diagram for traces.
//!
//! The Foata normal form decomposes a trace [w] into layers of concurrent steps.
//! Each layer is a set of pairwise-independent symbols that can be executed
//! simultaneously. Layers are separated by synchronization barriers.

use crate::independence::Independence;
use crate::trace::Trace;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::hash::Hash;

/// Foata normal form: a sequence of steps (layers), each step being a set of
/// pairwise-independent symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoataNormalForm<A: Eq + Hash + Clone + Ord> {
    steps: Vec<BTreeSet<A>>,
}

impl<A: Eq + Hash + Clone + Ord> FoataNormalForm<A> {
    /// Compute the Foata normal form of a trace.
    ///
    /// Algorithm: greedily assign each symbol (in order of the canonical word)
    /// to the earliest possible step where it is independent of all symbols
    /// already in that step.
    pub fn from_trace(trace: &Trace<A>, ind: &Independence<A>) -> Self {
        let word = trace.canonical();
        let mut steps: Vec<BTreeSet<A>> = Vec::new();

        for symbol in word {
            let mut placed = false;
            for step in &mut steps {
                // Check if symbol is independent of all symbols in this step
                let compatible = step.iter().all(|s| ind.are_independent(&symbol, s));
                if compatible {
                    step.insert(symbol.clone());
                    placed = true;
                    break;
                }
            }
            if !placed {
                steps.push({
                    let mut s = BTreeSet::new();
                    s.insert(symbol.clone());
                    s
                });
            }
        }

        FoataNormalForm { steps }
    }

    /// Compute from a raw word directly.
    pub fn from_word(word: Vec<A>, ind: &Independence<A>) -> Self {
        let trace = Trace::new(word, ind);
        Self::from_trace(&trace, ind)
    }

    /// The steps (layers).
    pub fn steps(&self) -> &[BTreeSet<A>] {
        &self.steps
    }

    /// Number of steps (parallel depth).
    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    /// Reconstruct a linearization by concatenating steps in order.
    pub fn to_word(&self) -> Vec<A> {
        self.steps.iter().flat_map(|s| s.iter().cloned()).collect()
    }

    /// The Hasse diagram as a DAG: for each symbol, find which earlier symbols
    /// it depends on (i.e., is not independent with). Returns edges as (from, to).
    pub fn hasse_diagram(&self, ind: &Independence<A>) -> Vec<(A, A)> {
        let mut edges = Vec::new();
        // Collect all symbols in order
        let all_syms: Vec<A> = self.to_word();
        for i in 1..all_syms.len() {
            for j in 0..i {
                if ind.are_dependent(&all_syms[i], &all_syms[j]) {
                    // Check if there's a k with j < k < i that also depends on both
                    let has_intermediate = (j + 1..i).any(|k| {
                        ind.are_dependent(&all_syms[i], &all_syms[k])
                            && ind.are_dependent(&all_syms[k], &all_syms[j])
                    });
                    if !has_intermediate {
                        edges.push((all_syms[j].clone(), all_syms[i].clone()));
                    }
                }
            }
        }
        edges
    }
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
    fn foata_all_independent() {
        let ind = Independence::new(
            "ab".chars().collect(),
            vec![('a', 'b')],
        );
        let trace = Trace::new(vec!['a', 'b'], &ind);
        let fnf = FoataNormalForm::from_trace(&trace, &ind);
        assert_eq!(fnf.depth(), 1); // Both in one step
        assert_eq!(fnf.steps()[0].len(), 2);
    }

    #[test]
    fn foata_all_dependent() {
        let ind = Independence::new(
            "ab".chars().collect(),
            vec![],
        );
        let trace = Trace::new(vec!['a', 'b'], &ind);
        let fnf = FoataNormalForm::from_trace(&trace, &ind);
        assert_eq!(fnf.depth(), 2); // Two sequential steps
    }

    #[test]
    fn foata_mixed() {
        let ind = make_ind();
        // a, c dependent; a, b independent; c, d independent
        let trace = Trace::new(vec!['a', 'c'], &ind);
        let fnf = FoataNormalForm::from_trace(&trace, &ind);
        assert_eq!(fnf.depth(), 2);
    }

    #[test]
    fn foata_from_word() {
        let ind = make_ind();
        let fnf = FoataNormalForm::from_word(vec!['b', 'a', 'c', 'd'], &ind);
        // b,a independent → step 1; c,d independent → step 2
        assert_eq!(fnf.depth(), 2);
    }

    #[test]
    fn hasse_edges() {
        let ind = make_ind();
        let fnf = FoataNormalForm::from_word(vec!['a', 'c'], &ind);
        let edges = fnf.hasse_diagram(&ind);
        assert!(edges.contains(&('a', 'c')));
    }

    #[test]
    fn foata_empty() {
        let ind = make_ind();
        let trace = Trace::new(vec![], &ind);
        let fnf = FoataNormalForm::from_trace(&trace, &ind);
        assert_eq!(fnf.depth(), 0);
        assert!(fnf.to_word().is_empty());
    }

    #[test]
    fn foata_serialization() {
        let ind = make_ind();
        let fnf = FoataNormalForm::from_word(vec!['a', 'b', 'c'], &ind);
        let json = serde_json::to_string(&fnf).unwrap();
        let fnf2: FoataNormalForm<char> = serde_json::from_str(&json).unwrap();
        assert_eq!(fnf.steps(), fnf2.steps());
    }
}
