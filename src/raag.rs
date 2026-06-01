//! Right-angled Artin groups (RAAGs) from commutation graph Γ.
//!
//! A RAAG A(Γ) is defined by an undirected graph Γ = (V, E). The group has
//! generators V and relations v·w = w·v whenever (v,w) ∈ E.
//! When Γ is the commutation graph of an independence relation I,
//! A(Γ) ≅ M(Σ, I) (the trace monoid extended to a group).

use crate::independence::Independence;
use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::hash::Hash;

/// A commutation graph Γ defining a RAAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommutationGraph<A: Eq + Hash + Clone + Ord> {
    /// Vertices (generators).
    vertices: BTreeSet<A>,
    /// Edges (commutation relations): (v, w) with v < w.
    edges: BTreeSet<(A, A)>,
}

impl<A: Eq + Hash + Clone + Ord> CommutationGraph<A> {
    /// Create from explicit vertices and edges.
    pub fn new(vertices: BTreeSet<A>, edges: Vec<(A, A)>) -> Self {
        let mut normalized = BTreeSet::new();
        for (a, b) in edges {
            if a != b && vertices.contains(&a) && vertices.contains(&b) {
                normalized.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
        CommutationGraph {
            vertices,
            edges: normalized,
        }
    }

    /// Create from an independence relation: Γ has edges exactly where (a,b) ∈ I.
    pub fn from_independence(ind: &Independence<A>) -> Self {
        CommutationGraph {
            vertices: ind.alphabet().clone(),
            edges: ind.pairs().clone(),
        }
    }

    pub fn vertices(&self) -> &BTreeSet<A> {
        &self.vertices
    }

    pub fn edges(&self) -> &BTreeSet<(A, A)> {
        &self.edges
    }

    /// Check if two generators commute.
    pub fn commute(&self, a: &A, b: &A) -> bool {
        if a == b {
            return false;
        }
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        self.edges.contains(&key)
    }

    /// Adjacency matrix (using nalgebra).
    pub fn adjacency_matrix(&self) -> DMatrix<f64> {
        let n = self.vertices.len();
        let symbs: Vec<&A> = self.vertices.iter().collect();
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                if i != j && self.commute(symbs[i], symbs[j]) {
                    data[i * n + j] = 1.0;
                }
            }
        }
        DMatrix::from_row_slice(n, n, &data)
    }

    /// The complement graph (dependency graph).
    pub fn complement(&self) -> CommutationGraph<A> {
        let symbs: Vec<&A> = self.vertices.iter().collect();
        let mut comp_edges = Vec::new();
        for i in 0..symbs.len() {
            for j in (i + 1)..symbs.len() {
                if !self.commute(symbs[i], symbs[j]) {
                    comp_edges.push((symbs[i].clone(), symbs[j].clone()));
                }
            }
        }
        CommutationGraph::new(self.vertices.clone(), comp_edges)
    }

    /// The clique number ω(Γ): size of the largest complete subgraph.
    pub fn clique_number(&self) -> usize {
        let symbs: Vec<&A> = self.vertices.iter().collect();
        let n = symbs.len();
        if n == 0 {
            return 0;
        }
        let mut max_clique = 1usize;
        // Try all subsets (feasible for small graphs)
        for mask in 1u32..(1 << n) {
            let indices: Vec<usize> = (0..n).filter(|&i| mask & (1 << i) != 0).collect();
            let is_clique = indices.iter().all(|&i| {
                indices.iter().all(|&j| {
                    i == j || self.commute(symbs[i], symbs[j])
                })
            });
            if is_clique {
                max_clique = max_clique.max(indices.len());
            }
        }
        max_clique
    }

    /// Whether the graph is a complete graph (free abelian group).
    pub fn is_complete(&self) -> bool {
        let n = self.vertices.len();
        let max_edges = n * (n - 1) / 2;
        self.edges.len() == max_edges
    }

    /// Whether the graph has no edges (free group).
    pub fn is_free(&self) -> bool {
        self.edges.is_empty()
    }
}

/// A RAAG word: a sequence of generators (with exponents ±1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaagWord<A: Eq + Hash + Clone + Ord> {
    /// (generator, exponent) pairs. Exponent is +1 or -1.
    letters: Vec<(A, i8)>,
}

impl<A: Eq + Hash + Clone + Ord> RaagWord<A> {
    pub fn new(letters: Vec<(A, i8)>) -> Self {
        RaagWord { letters }
    }

    pub fn letters(&self) -> &[(A, i8)] {
        &self.letters
    }

    /// Reduce the word using commutation relations only (no cancellation of inverses
    /// since RAAGs have no torsion relations beyond commutation).
    pub fn reduce(&self, graph: &CommutationGraph<A>) -> RaagWord<A> {
        let mut word = self.letters.clone();
        // Bubble sort style: swap adjacent commuting generators
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..word.len().saturating_sub(1) {
                // Can swap if the generators commute and it would be more "sorted"
                let (ref a, ref _ea) = word[i];
                let (ref b, ref _eb) = word[i + 1];
                if a != b && graph.commute(a, b) && a > b {
                    word.swap(i, i + 1);
                    changed = true;
                }
            }
        }
        RaagWord { letters: word }
    }

    /// Multiply two RAAG words (concatenation).
    pub fn multiply(&self, other: &RaagWord<A>, graph: &CommutationGraph<A>) -> RaagWord<A> {
        let mut combined = self.letters.clone();
        combined.extend(other.letters.iter().cloned());
        RaagWord { letters: combined }.reduce(graph)
    }

    /// The inverse word.
    pub fn inverse(&self) -> RaagWord<A> {
        let mut inv: Vec<(A, i8)> = self.letters.iter().rev()
            .map(|(a, e)| (a.clone(), -e))
            .collect();
        inv.reverse();
        RaagWord { letters: inv.into_iter().rev().collect() }
    }

    /// Length of the word.
    pub fn len(&self) -> usize {
        self.letters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.letters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> CommutationGraph<char> {
        let verts: BTreeSet<char> = "abc".chars().collect();
        CommutationGraph::new(verts, vec![('a', 'b')])
    }

    #[test]
    fn from_independence() {
        let alpha: BTreeSet<char> = "ab".chars().collect();
        let ind = Independence::new(alpha, vec![('a', 'b')]);
        let g = CommutationGraph::from_independence(&ind);
        assert!(g.commute(&'a', &'b'));
        assert!(!g.commute(&'a', &'a'));
    }

    #[test]
    fn adjacency_matrix() {
        let g = make_graph();
        let m = g.adjacency_matrix();
        assert_eq!(m.nrows(), 3);
        assert_eq!(m.ncols(), 3);
    }

    #[test]
    fn clique_number() {
        let g = make_graph();
        assert_eq!(g.clique_number(), 2); // {a,b} is a clique
    }

    #[test]
    fn complete_graph() {
        let verts: BTreeSet<char> = "ab".chars().collect();
        let g = CommutationGraph::new(verts, vec![('a', 'b')]);
        assert!(g.is_complete());
    }

    #[test]
    fn free_graph() {
        let verts: BTreeSet<char> = "ab".chars().collect();
        let g = CommutationGraph::new(verts, vec![]);
        assert!(g.is_free());
    }

    #[test]
    fn complement_graph() {
        let g = make_graph();
        let comp = g.complement();
        // Original has edge (a,b); complement has edges (a,c) and (b,c)
        assert!(!comp.commute(&'a', &'b'));
        assert!(comp.commute(&'a', &'c'));
        assert!(comp.commute(&'b', &'c'));
    }

    #[test]
    fn raag_word_reduce() {
        let g = make_graph();
        let w = RaagWord::new(vec![('b', 1), ('a', 1)]);
        let reduced = w.reduce(&g);
        // a and b commute, a < b, so should become a, b
        assert_eq!(reduced.letters()[0].0, 'a');
        assert_eq!(reduced.letters()[1].0, 'b');
    }

    #[test]
    fn raag_word_multiply() {
        let g = make_graph();
        let w1 = RaagWord::new(vec![('a', 1)]);
        let w2 = RaagWord::new(vec![('b', 1)]);
        let product = w1.multiply(&w2, &g);
        assert_eq!(product.len(), 2);
    }

    #[test]
    fn raag_inverse() {
        let g = make_graph();
        let w = RaagWord::new(vec![('a', 1), ('b', 1)]);
        let inv = w.inverse();
        assert_eq!(inv.letters()[0], ('b', -1));
        assert_eq!(inv.letters()[1], ('a', -1));
    }
}
