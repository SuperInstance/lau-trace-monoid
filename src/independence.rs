//! Independence relation I ⊆ Σ×Σ (symmetric, irreflexive).
//!
//! An independence relation over an alphabet Σ is a symmetric, irreflexive
//! binary relation. If (a,b) ∈ I then a and b may occur concurrently.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::hash::Hash;

/// Independence relation over an alphabet Σ.
///
/// Stored as a set of unordered pairs {a, b} with a ≠ b (irreflexive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Independence<A: Eq + Hash + Clone + Ord> {
    /// The alphabet Σ.
    alphabet: BTreeSet<A>,
    /// Independent pairs, stored as ordered pairs (a, b) with a < b.
    pairs: BTreeSet<(A, A)>,
}

impl<A: Eq + Hash + Clone + Ord> Independence<A> {
    /// Create a new independence relation with the given alphabet and pairs.
    ///
    /// # Errors
    /// Returns `None` if any pair violates symmetry/irreflexivity
    /// (this is automatically enforced by construction).
    pub fn new(alphabet: BTreeSet<A>, pairs: Vec<(A, A)>) -> Self {
        let mut normalized = BTreeSet::new();
        for (a, b) in pairs {
            if a != b && alphabet.contains(&a) && alphabet.contains(&b) {
                normalized.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
        Independence {
            alphabet,
            pairs: normalized,
        }
    }

    /// The underlying alphabet Σ.
    pub fn alphabet(&self) -> &BTreeSet<A> {
        &self.alphabet
    }

    /// Check if (a, b) ∈ I (i.e., a and b are independent).
    pub fn are_independent(&self, a: &A, b: &A) -> bool {
        if a == b {
            return false; // irreflexive
        }
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        self.pairs.contains(&key)
    }

    /// Check if (a, b) ∉ I (i.e., a and b are dependent / must be ordered).
    pub fn are_dependent(&self, a: &A, b: &A) -> bool {
        !self.are_independent(a, b)
    }

    /// The independence pairs.
    pub fn pairs(&self) -> &BTreeSet<(A, A)> {
        &self.pairs
    }

    /// Number of independent pairs.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// The dependency relation D = (Σ×Σ) \ I (restricted to Σ).
    pub fn dependency_pairs(&self) -> Vec<(A, A)> {
        let mut deps = Vec::new();
        let symbs: Vec<A> = self.alphabet.iter().cloned().collect();
        for i in 0..symbs.len() {
            for j in 0..symbs.len() {
                if i != j && self.are_dependent(&symbs[i], &symbs[j]) {
                    deps.push((symbs[i].clone(), symbs[j].clone()));
                }
            }
        }
        deps
    }

    /// Commutation graph: the undirected graph whose edges are the independent pairs.
    /// This is the complement of the dependency / commutation graph used for RAAGs.
    pub fn commutation_graph_adjacency(&self) -> Vec<Vec<bool>> {
        let n = self.alphabet.len();
        let symbs: Vec<&A> = self.alphabet.iter().collect();
        let mut adj = vec![vec![false; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i != j && self.are_independent(symbs[i], symbs[j]) {
                    adj[i][j] = true;
                }
            }
        }
        adj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_independence() {
        let alpha: BTreeSet<char> = "ab".chars().collect();
        let ind = Independence::new(alpha, vec![]);
        assert!(!ind.are_independent(&'a', &'b'));
        assert!(!ind.are_independent(&'a', &'a')); // irreflexive
        assert_eq!(ind.len(), 0);
    }

    #[test]
    fn symmetric_pair() {
        let alpha: BTreeSet<char> = "ab".chars().collect();
        let ind = Independence::new(alpha, vec![('a', 'b')]);
        assert!(ind.are_independent(&'a', &'b'));
        assert!(ind.are_independent(&'b', &'a')); // symmetric
        assert!(ind.are_dependent(&'a', &'a'));
    }

    #[test]
    fn full_independence() {
        let alpha: BTreeSet<char> = "abc".chars().collect();
        let pairs = vec![('a', 'b'), ('a', 'c'), ('b', 'c')];
        let ind = Independence::new(alpha, pairs);
        assert_eq!(ind.len(), 3);
        // All distinct pairs are independent
        for a in "abc".chars() {
            for b in "abc".chars() {
                if a != b {
                    assert!(ind.are_independent(&a, &b));
                }
            }
        }
    }

    #[test]
    fn ignores_out_of_alphabet() {
        let alpha: BTreeSet<char> = "ab".chars().collect();
        let ind = Independence::new(alpha, vec![('a', 'z')]);
        assert_eq!(ind.len(), 0);
    }

    #[test]
    fn dependency_pairs_correct() {
        let alpha: BTreeSet<char> = "abc".chars().collect();
        let ind = Independence::new(alpha, vec![('a', 'b')]);
        let deps = ind.dependency_pairs();
        // a-b independent, so deps: a-c, c-a, b-c, c-b
        assert!(deps.contains(&('a', 'c')));
        assert!(deps.contains(&('c', 'a')));
        assert!(deps.contains(&('b', 'c')));
        assert_eq!(deps.len(), 4);
    }
}
