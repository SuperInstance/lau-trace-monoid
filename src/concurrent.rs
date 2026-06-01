//! Parallel composition of independent traces.
//!
//! When two traces [u] and [v] use disjoint subsets of the alphabet (or
//! their symbols are pairwise independent), they can be composed in parallel
//! with all interleavings being trace-equivalent.

use crate::independence::Independence;
use crate::trace::Trace;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::hash::Hash;

/// Check if two traces can be composed in parallel: every symbol of t1
/// must be independent of every symbol of t2.
pub fn are_parallel_composable<A: Eq + Hash + Clone + Ord>(
    t1: &Trace<A>,
    t2: &Trace<A>,
    ind: &Independence<A>,
) -> bool {
    for a in t1.canonical() {
        for b in t2.canonical() {
            if !ind.are_independent(a, b) {
                return false;
            }
        }
    }
    true
}

/// Parallel composition of two traces. The result is the trace whose
/// canonical form interleaves the symbols in sorted order (since they're
/// all pairwise independent, any interleaving is equivalent).
pub fn parallel_compose<A: Eq + Hash + Clone + Ord>(
    t1: &Trace<A>,
    t2: &Trace<A>,
    ind: &Independence<A>,
) -> Result<Trace<A>, String> {
    if !are_parallel_composable(t1, t2, ind) {
        return Err("Traces are not parallel-composable: dependent symbols found".into());
    }
    let mut combined: Vec<A> = Vec::new();
    combined.extend(t1.canonical().iter().cloned());
    combined.extend(t2.canonical().iter().cloned());
    Ok(Trace::new(combined, ind))
}

/// A parallel composition node in a trace term.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParallelTerm<A: Eq + Hash + Clone + Ord> {
    /// A single symbol.
    Symbol(A),
    /// Sequential composition (concatenation).
    Seq(Vec<ParallelTerm<A>>),
    /// Parallel composition.
    Par(Vec<ParallelTerm<A>>),
}

impl<A: Eq + Hash + Clone + Ord> ParallelTerm<A> {
    /// Flatten to a trace by evaluating the term.
    pub fn to_trace(&self, ind: &Independence<A>) -> Trace<A> {
        match self {
            ParallelTerm::Symbol(a) => Trace::new(vec![a.clone()], ind),
            ParallelTerm::Seq(terms) => {
                let mut result = Trace::new(vec![], ind);
                for t in terms {
                    result = result.concatenate(&t.to_trace(ind), ind);
                }
                result
            }
            ParallelTerm::Par(terms) => {
                // Concatenate all symbols (they must be pairwise independent)
                let mut all_syms = Vec::new();
                for t in terms {
                    all_syms.extend(t.to_trace(ind).canonical().iter().cloned());
                }
                Trace::new(all_syms, ind)
            }
        }
    }

    /// Collect all symbols used in this term.
    pub fn symbols(&self) -> BTreeSet<A> {
        match self {
            ParallelTerm::Symbol(a) => {
                let mut s = BTreeSet::new();
                s.insert(a.clone());
                s
            }
            ParallelTerm::Seq(terms) | ParallelTerm::Par(terms) => {
                terms.iter().flat_map(|t| t.symbols()).collect()
            }
        }
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
    fn parallel_composable() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['a'], &ind);
        let t2 = Trace::new(vec!['b'], &ind);
        assert!(are_parallel_composable(&t1, &t2, &ind));
    }

    #[test]
    fn not_parallel_composable() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['a'], &ind);
        let t2 = Trace::new(vec!['c'], &ind);
        assert!(!are_parallel_composable(&t1, &t2, &ind));
    }

    #[test]
    fn parallel_compose_independent() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['a'], &ind);
        let t2 = Trace::new(vec!['b'], &ind);
        let composed = parallel_compose(&t1, &t2, &ind).unwrap();
        assert_eq!(composed.len(), 2);
    }

    #[test]
    fn parallel_compose_fails_dependent() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['a'], &ind);
        let t2 = Trace::new(vec!['c'], &ind);
        assert!(parallel_compose(&t1, &t2, &ind).is_err());
    }

    #[test]
    fn parallel_term_to_trace() {
        let ind = make_ind();
        let term = ParallelTerm::Par(vec![
            ParallelTerm::Symbol('a'),
            ParallelTerm::Symbol('b'),
        ]);
        let trace = term.to_trace(&ind);
        assert_eq!(trace.len(), 2);
    }

    #[test]
    fn parallel_term_symbols() {
        let term = ParallelTerm::Seq(vec![
            ParallelTerm::Symbol('a'),
            ParallelTerm::Symbol('b'),
        ]);
        let syms = term.symbols();
        assert!(syms.contains(&'a'));
        assert!(syms.contains(&'b'));
    }
}
