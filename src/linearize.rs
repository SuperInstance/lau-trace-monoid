//! Total order extraction (S_n quotient from B_n).
//!
//! A trace [w] may have multiple linearizations (total orders of its symbols).
//! These form a coset of S_n modulo the independence swaps, analogous to
//! taking a quotient of the symmetric group S_n by the braid group B_n relations.

use crate::independence::Independence;
use crate::trace::{all_linearizations, Trace};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// A linearization: a total order consistent with the trace's partial order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Linearization<A: Eq + Hash + Clone + Ord> {
    /// The total order as a sequence.
    order: Vec<A>,
}

impl<A: Eq + Hash + Clone + Ord> Linearization<A> {
    pub fn new(order: Vec<A>) -> Self {
        Linearization { order }
    }

    pub fn order(&self) -> &[A] {
        &self.order
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

/// Compute all linearizations of a trace.
pub fn linearizations<A: Eq + Hash + Clone + Ord + std::fmt::Debug>(
    trace: &Trace<A>,
    ind: &Independence<A>,
) -> Vec<Linearization<A>> {
    all_linearizations(trace, ind)
        .into_iter()
        .map(Linearization::new)
        .collect()
}

/// The number of linearizations (|S_n / ~_I| for the trace).
pub fn linearization_count<A: Eq + Hash + Clone + Ord + std::fmt::Debug>(
    trace: &Trace<A>,
    ind: &Independence<A>,
) -> usize {
    linearizations(trace, ind).len()
}

/// Given two linearizations, find the sequence of adjacent transpositions
/// (braid-like) needed to transform one into the other.
/// Returns positions where adjacent swaps should occur.
pub fn transposition_distance<A: Eq + Hash + Clone + Ord>(
    l1: &Linearization<A>,
    l2: &Linearization<A>,
) -> Vec<usize> {
    let mut current = l1.order.clone();
    let target = &l2.order;
    let mut swaps = Vec::new();

    for target_pos in 0..target.len() {
        // Find where target[target_pos] is in current
        let curr_pos = current.iter().position(|x| *x == target[target_pos]).unwrap();
        // Bubble it forward to target_pos
        for i in (target_pos..curr_pos).rev() {
            // No, we need to swap from curr_pos down to target_pos
        }
        // Actually: bubble from curr_pos to target_pos by adjacent swaps
        for i in (target_pos..curr_pos).rev() {
            // swap i+1 to i, i.e., bubble down
        }
        // Let's redo: bubble current[curr_pos] to position target_pos
        for i in (target_pos..curr_pos).rev() {
            // doesn't make sense, let me think again
        }
        // Swap current[i] forward: from curr_pos down to target_pos+1
        // Each step swaps positions i and i-1
        for i in (target_pos + 1..=curr_pos).rev() {
            current.swap(i, i - 1);
            swaps.push(i - 1);
        }
    }
    swaps
}

/// Extract a canonical (lexicographically first) linearization.
pub fn canonical_linearization<A: Eq + Hash + Clone + Ord>(
    trace: &Trace<A>,
) -> Linearization<A> {
    Linearization::new(trace.canonical().to_vec())
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
    fn linearizations_count() {
        let ind = make_ind();
        let trace = Trace::new(vec!['a', 'b'], &ind);
        // a and b independent: 2! = 2 linearizations
        let lins = linearizations(&trace, &ind);
        assert_eq!(lins.len(), 2);
    }

    #[test]
    fn linearizations_all_dependent() {
        let ind = make_ind();
        let trace = Trace::new(vec!['a', 'c'], &ind);
        // a and c dependent: exactly 1 linearization
        let lins = linearizations(&trace, &ind);
        assert_eq!(lins.len(), 1);
    }

    #[test]
    fn linearization_count_fn() {
        let ind = make_ind();
        let trace = Trace::new(vec!['a', 'b'], &ind);
        assert_eq!(linearization_count(&trace, &ind), 2);
    }

    #[test]
    fn transposition_distance_basic() {
        let l1 = Linearization::new(vec!['b', 'a']);
        let l2 = Linearization::new(vec!['a', 'b']);
        let swaps = transposition_distance(&l1, &l2);
        assert!(!swaps.is_empty());
    }

    #[test]
    fn canonical_linearization_works() {
        let ind = make_ind();
        let trace = Trace::new(vec!['b', 'a'], &ind);
        let canon = canonical_linearization(&trace);
        assert_eq!(canon.order(), &['a', 'b']);
    }

    #[test]
    fn empty_linearization() {
        let ind = make_ind();
        let trace = Trace::new(vec![], &ind);
        let lins = linearizations(&trace, &ind);
        assert_eq!(lins.len(), 1);
        assert!(lins[0].is_empty());
    }
}
