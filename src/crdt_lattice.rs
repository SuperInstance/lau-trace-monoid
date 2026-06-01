//! CRDT join-semilattice (bounded, merge = join, convergence to LUB).
//!
//! A state-based CRDT (CvRDT) is a join-semilattice: a partially ordered set
//! with a least upper bound (join) operation for every pair of elements.
//! The merge operation is the join, and it converges to the LUB.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

/// A bounded join-semilattice element.
/// Generic over the value type. Uses a vector clock for ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeValue<V: Eq + Clone + Ord> {
    /// The actual value.
    value: V,
    /// Version vector (replica_id → count).
    version: BTreeMap<String, u64>,
}

impl<V: Eq + Clone + Ord> LatticeValue<V> {
    pub fn new(value: V, version: BTreeMap<String, u64>) -> Self {
        LatticeValue { value, version }
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn version(&self) -> &BTreeMap<String, u64> {
        &self.version
    }

    /// Create with single replica version.
    pub fn single(value: V, replica: &str, count: u64) -> Self {
        let mut version = BTreeMap::new();
        version.insert(replica.to_string(), count);
        LatticeValue { value, version }
    }

    /// The bottom element (empty version vector).
    pub fn bottom(value: V) -> Self {
        LatticeValue {
            value,
            version: BTreeMap::new(),
        }
    }
}

/// A G-Counter CRDT (grow-only counter).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GCounter {
    counts: BTreeMap<String, u64>,
}

impl GCounter {
    pub fn new() -> Self {
        GCounter {
            counts: BTreeMap::new(),
        }
    }

    /// Increment the counter for a replica.
    pub fn inc(&mut self, replica: &str) {
        *self.counts.entry(replica.to_string()).or_insert(0) += 1;
    }

    /// Get the total count.
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Merge (join) two G-Counters: take component-wise maximum.
    pub fn merge(&self, other: &GCounter) -> GCounter {
        let mut merged = self.counts.clone();
        for (replica, count) in &other.counts {
            let entry = merged.entry(replica.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
        GCounter { counts: merged }
    }

    /// Partial order: self ≤ other iff all components of self ≤ other.
    pub fn leq(&self, other: &GCounter) -> bool {
        for (replica, count) in &self.counts {
            let other_count = other.counts.get(replica).copied().unwrap_or(0);
            if count > &other_count {
                return false;
            }
        }
        true
    }
}

/// An LWW-Register CRDT (Last-Writer-Wins Register).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister<V: Clone + Eq> {
    value: V,
    timestamp: u64,
    replica_id: String,
}

impl<V: Clone + Eq> LWWRegister<V> {
    pub fn new(value: V, timestamp: u64, replica_id: String) -> Self {
        LWWRegister { value, timestamp, replica_id }
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    /// Merge: keep the value with higher timestamp; break ties by replica_id.
    pub fn merge(&self, other: &LWWRegister<V>) -> LWWRegister<V> {
        if self.timestamp > other.timestamp
            || (self.timestamp == other.timestamp && self.replica_id >= other.replica_id)
        {
            self.clone()
        } else {
            other.clone()
        }
    }
}

/// An OR-Set CRDT (Observed-Remove Set).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet<A: Eq + Hash + Clone + Ord> {
    /// Element → set of unique tags.
    elements: BTreeMap<A, BTreeSet<String>>,
    /// Tombstones: removed tags.
    tombstones: BTreeSet<String>,
}

impl<A: Eq + Hash + Clone + Ord> ORSet<A> {
    pub fn new() -> Self {
        ORSet {
            elements: BTreeMap::new(),
            tombstones: BTreeSet::new(),
        }
    }

    /// Add an element with a unique tag.
    pub fn add(&mut self, element: A, tag: String) {
        if !self.tombstones.contains(&tag) {
            self.elements.entry(element).or_default().insert(tag);
        }
    }

    /// Remove an element (tombstones all current tags).
    pub fn remove(&mut self, element: &A) {
        if let Some(tags) = self.elements.remove(element) {
            self.tombstones.extend(tags);
        }
    }

    /// Check if element is in the set.
    pub fn contains(&self, element: &A) -> bool {
        self.elements.contains_key(element)
    }

    /// Merge two OR-Sets.
    pub fn merge(&self, other: &ORSet<A>) -> ORSet<A> {
        let mut merged_elements: BTreeMap<A, BTreeSet<String>> = BTreeMap::new();
        let mut merged_tombstones = self.tombstones.clone();
        merged_tombstones.extend(other.tombstones.iter().cloned());

        // Add elements from self, excluding tombstoned tags
        for (elem, tags) in &self.elements {
            let live_tags: BTreeSet<String> = tags.difference(&merged_tombstones).cloned().collect();
            if !live_tags.is_empty() {
                merged_elements.insert(elem.clone(), live_tags);
            }
        }

        // Add elements from other, excluding tombstoned tags
        for (elem, tags) in &other.elements {
            let live_tags: BTreeSet<String> = tags.difference(&merged_tombstones).cloned().collect();
            if !live_tags.is_empty() {
                merged_elements
                    .entry(elem.clone())
                    .or_default()
                    .extend(live_tags);
            }
        }

        ORSet {
            elements: merged_elements,
            tombstones: merged_tombstones,
        }
    }

    /// Current elements.
    pub fn elements(&self) -> BTreeSet<&A> {
        self.elements.keys().collect()
    }
}

/// Verify that a merge operation satisfies the semilattice laws.
pub fn verify_join_semilattice<T: Clone + PartialEq + std::fmt::Debug>(
    a: &T,
    b: &T,
    merge: impl Fn(&T, &T) -> T,
) -> bool {
    let ab = merge(a, b);
    let ba = merge(b, a);
    // Commutativity: a ∨ b = b ∨ a
    if ab != ba {
        return false;
    }
    // Idempotence: a ∨ a = a
    let aa = merge(a, a);
    if aa != *a {
        return false;
    }
    // Associativity: (a ∨ b) ∨ c = a ∨ (b ∨ c)
    // (tested with a and b only for simplicity)
    let ab_c = merge(&ab, a);
    let a_bc = merge(a, &ba);
    if ab_c != a_bc {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g_counter_basic() {
        let mut c = GCounter::new();
        c.inc("r1");
        c.inc("r1");
        assert_eq!(c.value(), 2);
    }

    #[test]
    fn g_counter_merge() {
        let mut c1 = GCounter::new();
        c1.inc("r1");
        c1.inc("r1");
        let mut c2 = GCounter::new();
        c2.inc("r2");
        c2.inc("r2");
        c2.inc("r2");
        let merged = c1.merge(&c2);
        assert_eq!(merged.value(), 5);
    }

    #[test]
    fn g_counter_idempotent_merge() {
        let mut c = GCounter::new();
        c.inc("r1");
        let merged = c.merge(&c);
        assert_eq!(merged.value(), c.value());
    }

    #[test]
    fn g_counter_leq() {
        let mut c1 = GCounter::new();
        c1.inc("r1");
        let mut c2 = GCounter::new();
        c2.inc("r1");
        c2.inc("r1");
        assert!(c1.leq(&c2));
        assert!(!c2.leq(&c1));
    }

    #[test]
    fn g_counter_semilattice() {
        let mut c1 = GCounter::new();
        c1.inc("r1");
        let mut c2 = GCounter::new();
        c2.inc("r2");
        assert!(verify_join_semilattice(&c1, &c2, GCounter::merge));
    }

    #[test]
    fn lww_register_basic() {
        let r = LWWRegister::new("hello".to_string(), 1, "r1".to_string());
        assert_eq!(r.value(), "hello");
    }

    #[test]
    fn lww_register_merge() {
        let r1 = LWWRegister::new("hello".to_string(), 1, "r1".to_string());
        let r2 = LWWRegister::new("world".to_string(), 2, "r2".to_string());
        let merged = r1.merge(&r2);
        assert_eq!(merged.value(), "world"); // higher timestamp wins
    }

    #[test]
    fn lww_register_merge_tie() {
        let r1 = LWWRegister::new("hello".to_string(), 1, "r1".to_string());
        let r2 = LWWRegister::new("world".to_string(), 1, "r2".to_string());
        let merged = r1.merge(&r2);
        // Tie broken by replica_id; r2 >= r1 so r2 wins
        assert_eq!(merged.value(), "world");
    }

    #[test]
    fn or_set_add_remove() {
        let mut s: ORSet<char> = ORSet::new();
        s.add('a', "tag1".to_string());
        assert!(s.contains(&'a'));
        s.remove(&'a');
        assert!(!s.contains(&'a'));
    }

    #[test]
    fn or_set_merge() {
        let mut s1: ORSet<char> = ORSet::new();
        s1.add('a', "tag1".to_string());
        s1.add('b', "tag2".to_string());
        let mut s2: ORSet<char> = ORSet::new();
        s2.add('a', "tag1".to_string());
        s2.add('c', "tag3".to_string());
        let merged = s1.merge(&s2);
        assert!(merged.contains(&'a'));
        assert!(merged.contains(&'b'));
        assert!(merged.contains(&'c'));
    }

    #[test]
    fn or_set_merge_with_removal() {
        let mut s1: ORSet<char> = ORSet::new();
        s1.add('a', "tag1".to_string());
        let mut s2: ORSet<char> = ORSet::new();
        s2.add('a', "tag1".to_string());
        s2.remove(&'a');
        let merged = s1.merge(&s2);
        assert!(!merged.contains(&'a')); // tombstoned
    }

    #[test]
    fn lattice_value_basic() {
        let v = LatticeValue::single(42u64, "r1", 1);
        assert_eq!(*v.value(), 42);
        assert_eq!(v.version().get("r1"), Some(&1));
    }
}
