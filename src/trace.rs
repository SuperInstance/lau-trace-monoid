//! Trace monoid M(Σ, I) — the free monoid Σ* modulo the independence relation.
//!
//! Two words u, v ∈ Σ* are trace-equivalent (~_I) if one can be obtained from the
//! other by repeatedly swapping adjacent independent symbols: ...ab... ↔ ...ba...
//! when (a,b) ∈ I. A trace [u] is the equivalence class.

use crate::independence::Independence;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// A trace: an equivalence class of words over Σ modulo the independence relation I.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace<A: Eq + Hash + Clone + Ord> {
    /// A canonical (lexicographically least) representative.
    canonical: Vec<A>,
}

impl<A: Eq + Hash + Clone + Ord> Trace<A> {
    /// Create a trace from a word, computing the canonical representative
    /// via a bubble-sort style normalization (swap adjacent independent symbols
    /// until stable, choosing lexicographic order).
    pub fn new(word: Vec<A>, ind: &Independence<A>) -> Self {
        let canonical = normalize(word, ind);
        Trace { canonical }
    }

    /// The canonical representative.
    pub fn canonical(&self) -> &[A] {
        &self.canonical
    }

    /// Length of the trace (all representatives have the same length).
    pub fn len(&self) -> usize {
        self.canonical.len()
    }

    pub fn is_empty(&self) -> bool {
        self.canonical.is_empty()
    }

    /// Concatenate two traces: [u] · [v] = [uv].
    pub fn concatenate(&self, other: &Trace<A>, ind: &Independence<A>) -> Trace<A> {
        let mut combined = self.canonical.clone();
        combined.extend(other.canonical.iter().cloned());
        Trace::new(combined, ind)
    }

    /// Check if two words represent the same trace.
    pub fn words_equivalent(u: &[A], v: &[A], ind: &Independence<A>) -> bool {
        if u.len() != v.len() {
            return false;
        }
        normalize(u.to_vec(), ind) == normalize(v.to_vec(), ind)
    }

    /// The Parikh image: count of each symbol in the trace.
    pub fn parikh_image(&self) -> HashMap<A, usize> {
        let mut counts = HashMap::new();
        for a in &self.canonical {
            *counts.entry(a.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Alphabet of this trace (symbols actually used).
    pub fn alphabet(&self) -> BTreeSet<A> {
        self.canonical.iter().cloned().collect()
    }
}

/// Normalize a word to its lexicographically least representative
/// by repeatedly swapping adjacent independent pairs toward sorted order.
fn normalize<A: Eq + Hash + Clone + Ord>(mut word: Vec<A>, ind: &Independence<A>) -> Vec<A> {
    if word.is_empty() {
        return word;
    }
    // Bubble-sort style: swap adjacent (a, b) when (a,b) ∈ I and a > b
    // until fixed point.
    let n = word.len();
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n.saturating_sub(1) {
            if word[i] > word[i + 1] && ind.are_independent(&word[i], &word[i + 1]) {
                word.swap(i, i + 1);
                changed = true;
            }
        }
    }
    word
}

/// Compute all linearizations (total order extensions) of a trace.
/// Brute-force for small traces: enumerate all permutations and keep those
/// equivalent to the trace.
pub fn all_linearizations<A: Eq + Hash + Clone + Ord + std::fmt::Debug>(
    trace: &Trace<A>,
    ind: &Independence<A>,
) -> Vec<Vec<A>> {
    let word = trace.canonical.clone();
    let mut result = Vec::new();
    let mut permutations = Vec::new();
    enumerate_permutations(&word, &mut permutations);
    let mut seen: HashSet<Vec<A>> = HashSet::new();
    for perm in permutations {
        if seen.contains(&perm) {
            continue;
        }
        if Trace::words_equivalent(&word, &perm, ind) {
            seen.insert(perm.clone());
            result.push(perm);
        }
    }
    result
}

fn enumerate_permutations<A: Clone + Ord>(items: &[A], out: &mut Vec<Vec<A>>) {
    if items.is_empty() {
        out.push(vec![]);
        return;
    }
    let mut sorted = items.to_vec();
    sorted.sort();
    let mut used = vec![false; sorted.len()];
    let mut current = Vec::new();
    perm_helper(&sorted, &mut used, &mut current, out);
}

fn perm_helper<A: Clone + Eq>(
    items: &[A],
    used: &mut [bool],
    current: &mut Vec<A>,
    out: &mut Vec<Vec<A>>,
) {
    if current.len() == items.len() {
        out.push(current.clone());
        return;
    }
    for i in 0..items.len() {
        if used[i] {
            continue;
        }
        // Skip duplicates
        if i > 0 && items[i] == items[i - 1] && !used[i - 1] {
            continue;
        }
        used[i] = true;
        current.push(items[i].clone());
        perm_helper(items, used, current, out);
        current.pop();
        used[i] = false;
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
    fn trace_identity() {
        let ind = make_ind();
        let t = Trace::new(vec!['a', 'b'], &ind);
        assert_eq!(t.canonical(), &['a', 'b']);
    }

    #[test]
    fn trace_swap_independent() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['b', 'a'], &ind);
        let t2 = Trace::new(vec!['a', 'b'], &ind);
        assert_eq!(t1.canonical(), t2.canonical());
    }

    #[test]
    fn trace_no_swap_dependent() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['a', 'c'], &ind);
        let t2 = Trace::new(vec!['c', 'a'], &ind);
        assert_ne!(t1.canonical(), t2.canonical());
    }

    #[test]
    fn words_equivalent_basic() {
        let ind = make_ind();
        assert!(Trace::words_equivalent(&['b', 'a'], &['a', 'b'], &ind));
        assert!(!Trace::words_equivalent(&['a', 'c'], &['c', 'a'], &ind));
    }

    #[test]
    fn trace_concatenate() {
        let ind = make_ind();
        let t1 = Trace::new(vec!['a'], &ind);
        let t2 = Trace::new(vec!['b'], &ind);
        let combined = t1.concatenate(&t2, &ind);
        assert_eq!(combined.len(), 2);
    }

    #[test]
    fn parikh_image() {
        let ind = make_ind();
        let t = Trace::new(vec!['a', 'b', 'a'], &ind);
        let parikh = t.parikh_image();
        assert_eq!(parikh[&'a'], 2);
        assert_eq!(parikh[&'b'], 1);
    }

    #[test]
    fn empty_trace() {
        let ind = make_ind();
        let t = Trace::new(vec![], &ind);
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn longer_normalization() {
        // ab independent, so bac → abc (swap b,a since a<b and independent)
        let ind = make_ind();
        let t = Trace::new(vec!['b', 'a', 'c'], &ind);
        assert_eq!(t.canonical(), &['a', 'b', 'c']);
    }
}
