# lau-trace-monoid

**Mazurkiewicz trace monoids, right-angled Artin groups, and CRDT lattice structures for concurrent computation.**

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 2021](https://img.shields.io/badge/edition-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)

---

## What This Does

`lau-trace-monoid` implements the algebra of **concurrency** — the mathematical structures that describe when operations can be reordered, run in parallel, or must be sequenced. The crate provides:

- **Independence relations** — symmetric irreflexive relations $I \subseteq \Sigma \times \Sigma$ declaring which operations commute
- **Trace monoids** $M(\Sigma, I)$ — the free monoid $\Sigma^*$ modulo commutation of independent symbols
- **Prefix order** — the natural partial order on traces (the "happened-before" relation)
- **Foata normal form** — the canonical layer decomposition of a trace into maximal concurrent steps
- **Linearizations** — all total orders (serial executions) consistent with a trace's partial order
- **Right-angled Artin groups (RAAGs)** — the group-theoretic extension of trace monoids, defined by commutation graphs
- **Concurrent composition** — the parallel product of traces using the independence relation
- **CRDT lattice structures** — G-Counter, LWW-Register, and OR-Set as semilattice elements with merge operations
- **CALM theorem** — monotonicity analysis to determine which computations need coordination
- **Kleene fixpoints** — iterative computation of least fixed points on lattice structures

This crate bridges abstract algebra and distributed systems: the same commutation structure that defines a trace monoid also determines which CRDT operations can be merged without coordination.

---

## Key Idea

A **Mazurkiewicz trace** over an alphabet $\Sigma$ with independence relation $I$ is an equivalence class of words where two words are equivalent if one can be obtained from the other by swapping adjacent independent symbols:

$$u \sim_I v \iff u \text{ can be transformed to } v \text{ by swaps } ab \leftrightarrow ba \text{ where } (a,b) \in I$$

The trace monoid $M(\Sigma, I) = \Sigma^* / \sim_I$ is the quotient. Each trace has:

- A **canonical representative** (lexicographically least word in the class)
- A **Parikh image** (multiset of symbols — invariant under commutation)
- A **Foata normal form** (layer decomposition into maximal concurrent steps)
- A set of **linearizations** (all valid total orderings)

When the independence relation comes from a commutation graph $\Gamma$, the trace monoid embeds into the **right-angled Artin group** $A(\Gamma)$ — the group with generators $V(\Gamma)$ and relations $vw = wv$ whenever $(v,w) \in E(\Gamma)$.

For distributed systems, if operations are symbols and their commutativity defines $I$, then CRDT merge is exactly the trace monoid concatenation — independent operations can be applied in any order and still yield the same state.

---

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-trace-monoid = "0.1"
```

Or clone directly:

```bash
git clone https://github.com/SuperInstance/lau-trace-monoid.git
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| [`nalgebra`](https://crates.io/crates/nalgebra) `0.33` | Adjacency and Laplacian matrices for RAAG graphs |
| [`serde`](https://crates.io/crates/serde) `1` | Serialization of traces, RAAG words, CRDT states |
| [`serde_json`](https://crates.io/crates/serde_json) `1` | JSON round-tripping |

---

## Quick Start

```rust
use lau_trace_monoid::{
    Independence, Trace, FoataNormalForm, Linearization,
    CommutationGraph, RaagWord,
    GCounter, LWWRegister, ORSet,
    CalmAnalysis,
};
use std::collections::BTreeSet;

// 1. Define which operations commute
let alphabet: BTreeSet<char> = "abcd".chars().collect();
let ind = Independence::new(alphabet, vec![('a', 'b'), ('c', 'd')]);
// a↔b independent, c↔d independent; all other pairs must be sequenced

// 2. Create traces
let t1 = Trace::new(vec!['b', 'a', 'c'], &ind);
let t2 = Trace::new(vec!['a', 'b', 'c'], &ind);
assert_eq!(t1.canonical(), t2.canonical()); // Same trace! (a,b commute)

// 3. Foata normal form — maximal concurrent steps
let fnf = FoataNormalForm::from_word(vec!['a', 'c', 'b', 'd'], &ind);
// Step 1: {a, b} (concurrent), Step 2: {c, d} (concurrent)
assert_eq!(fnf.depth(), 2);

// 4. Linearizations — all valid serial orderings
let trace = Trace::new(vec!['a', 'b'], &ind);
let lins = linearizations(&trace, &ind);
assert_eq!(lins.len(), 2); // "ab" and "ba" are both valid

// 5. RAAG from the commutation graph
let graph = CommutationGraph::from_independence(&ind);
let w1 = RaagWord::new(vec![('a', 1), ('b', 1)]);
let w2 = RaagWord::new(vec![('b', 1), ('a', 1)]);
let product = w1.multiply(&w2, &graph); // Commutes via graph relation

// 6. CRDTs — built on the same commutation structure
let mut c1 = GCounter::new();
c1.inc("replica-1");
let mut c2 = GCounter::new();
c2.inc("replica-2");
let merged = c1.merge(&c2); // Monotone merge — no coordination needed

// 7. CALM analysis
let analysis = CalmAnalysis::g_counter_sum();
assert!(analysis.coordination_free); // Monotone → coordination-free
```

---

## API Reference

### `independence` — `Independence<A>`

A symmetric, irreflexive relation $I \subseteq \Sigma \times \Sigma$.

| Method | Description |
|--------|-------------|
| `new(alphabet, pairs)` | Construct from alphabet $\Sigma$ and independent pairs |
| `are_independent(a, b)` | Check if $(a, b) \in I$ |
| `are_dependent(a, b)` | Check if $(a, b) \notin I$ |
| `dependency_pairs()` | The complement relation $D = (\Sigma \times \Sigma) \setminus I$ |
| `commutation_graph_adjacency()` | Adjacency matrix of the commutation graph |

### `trace` — `Trace<A>`

An equivalence class $[w] \in M(\Sigma, I)$.

| Method | Description |
|--------|-------------|
| `new(word, ind)` | Create trace, computing the canonical representative |
| `canonical()` | Lexicographically least representative |
| `concatenate(other, ind)` | Trace multiplication $[u] \cdot [v] = [uv]$ |
| `parikh_image()` | Multiset of symbols (commutation-invariant) |
| `words_equivalent(u, v, ind)` | Check if two words represent the same trace |

### `foata` — `FoataNormalForm<A>`

The layer decomposition $F = (S_1)(S_2)\cdots(S_k)$ where each $S_i$ is a maximal set of mutually independent symbols.

| Method | Description |
|--------|-------------|
| `from_trace(trace, ind)` | Compute Foata normal form |
| `from_word(word, ind)` | Directly from a word |
| `steps()` | The layers $(S_1, \ldots, S_k)$ |
| `depth()` | Parallel depth = number of layers |
| `hasse_diagram(ind)` | The dependency DAG edges |

### `prefix` — `PrefixOrder<A>`

The partial order on traces: $[u] \leq [v]$ iff $v = [u] \cdot [w]$ for some $w$.

| Method | Description |
|--------|-------------|
| `leq(t1, t2, ind)` | Check if $t_1 \leq t_2$ |
| `prefixes(trace, ind)` | All prefixes of a trace |
| `greatest_lower_bound(t1, t2)` | Meet in the prefix order |

### `linearize` — `Linearization<A>`

| Method | Description |
|--------|-------------|
| `linearizations(trace, ind)` | All total orders consistent with the trace |
| `linearization_count(trace, ind)` | Number of linearizations |
| `transposition_distance(l1, l2)` | Adjacent swaps to transform $l_1$ into $l_2$ |
| `canonical_linearization(trace)` | The lex-least linearization |

### `raag` — `CommutationGraph<A>`, `RaagWord<A>`

Right-angled Artin groups from commutation graphs.

| Method | Description |
|--------|-------------|
| `CommutationGraph::from_independence(ind)` | Build graph from independence relation |
| `adjacency_matrix()` | Graph adjacency as `nalgebra::DMatrix` |
| `clique_number()` | $\omega(\Gamma)$ — largest complete subgraph |
| `is_free()` | No edges → free group $F_n$ |
| `is_complete()` | All edges → free abelian group $\mathbb{Z}^n$ |
| `RaagWord::reduce(graph)` | Normalize using commutation relations |
| `RaagWord::multiply(other, graph)` | Product in the RAAG |

### `concurrent` — Concurrent Composition

Parallel product of traces: $[u] \parallel [v] = [u] \cdot [v]$ after interleaving independent parts.

### `crdt_lattice` — `GCounter`, `LWWRegister<T>`, `ORSet<A>`

State-based CRDTs as semilattice elements.

| Type | Merge | Order |
|------|-------|-------|
| `GCounter` | Componentwise `max` | $\leq$ by all components |
| `LWWRegister<T>` | Last-writer-wins by timestamp | Last value wins |
| `ORSet<A>` | Observed-remove: union minus tombstones | Subset order |

All implement `Clone`, `PartialEq`, `Serialize`, `Deserialize`.

### `calm` — `CalmAnalysis`, `MonotoneFunction`

The **CALM theorem** (Consistency As Logical Monotonicity): a computation is coordination-free if and only if it is monotone.

| Method | Description |
|--------|-------------|
| `CalmAnalysis::g_counter_sum()` | Monotone → coordination-free ✓ |
| `CalmAnalysis::threshold_query()` | Non-monotone → requires coordination ✗ |
| `CalmAnalysis::set_union()` | Monotone → coordination-free ✓ |
| `MonotoneFunction::check_monotonicity` | Verify $a \leq b \implies f(a) \leq f(b)$ |
| `check_convergence(states, merge)` | Verify all replicas converge to the same state |

### `fixpoint` — Kleene Fixpoints

Iterative least-fixed-point computation on lattices. Given a monotone function $f : L \to L$ on a complete lattice, compute $\mathrm{lfp}(f) = \bigsqcup_{n} f^n(\bot)$.

---

## How It Works

The crate implements the algebra of concurrency in three layers:

```
Layer 1: Combinatorial
    Independence → Trace → Foata → Linearizations → Prefix order
                                    │
Layer 2: Algebraic               ▼
    CommutationGraph → RAAG → RaagWord → Group operations
                                    │
Layer 3: Distributed              ▼
    CRDTs (GCounter, LWW, ORSet) → CALM → Fixpoints
```

1. **Normalization**: A word is reduced to its canonical representative by bubble-sorting adjacent independent pairs: if $a > b$ and $(a,b) \in I$, swap them. This terminates because each swap reduces the inversion count in the total order.

2. **Foata decomposition**: Greedily collect maximal independent prefixes. At each step, all symbols that have no unresolved dependency with anything already consumed form the next concurrent layer.

3. **RAAG connection**: The commutation graph $\Gamma$ of the independence relation $I$ defines a right-angled Artin group $A(\Gamma)$. The trace monoid $M(\Sigma, I)$ is the positive monoid inside $A(\Gamma)$ (words with all exponents $+1$).

4. **CRDT merge as trace concatenation**: When CRDT operations are commutative (independent in $I$), merging replicas corresponds to trace multiplication — the result is independent of the merge order.

5. **CALM monotonicity**: The `MonotoneFunction` trait checks whether $f : L \to L$ is order-preserving. By the CALM theorem, monotone functions on CRDT lattices are coordination-free.

---

## The Math

### Trace Monoid

Given $(\Sigma, I)$ with $I$ symmetric and irreflexive, the **Mazurkiewicz trace monoid** is:

$$M(\Sigma, I) = \Sigma^* / \sim_I$$

where $\sim_I$ is the smallest congruence containing $ab \sim ba$ for all $(a,b) \in I$.

**Key property**: The Parikh image $\pi : M(\Sigma, I) \to \mathbb{N}^\Sigma$ is well-defined (commutation preserves symbol counts).

### Foata Normal Form

Every trace $[w]$ has a unique decomposition into **maximal concurrent steps**:

$$[w] = (S_1)(S_2)\cdots(S_k)$$

where each $S_i$ is a maximal antichain in the dependency order. The number of steps $k$ is the **parallel depth** (minimum parallel time to execute the trace).

### Right-Angled Artin Groups

For a graph $\Gamma = (V, E)$, the RAAG $A(\Gamma)$ has:

- **Generators**: $V$
- **Relations**: $vw = wv$ whenever $(v,w) \in E$

Special cases:
- $\Gamma = \emptyset$ (no edges) → free group $F_n$
- $\Gamma = K_n$ (complete graph) → free abelian group $\mathbb{Z}^n$
- General $\Gamma$ → interpolates between free and abelian

### CRDT Semilattices

A state-based CRDT is a join-semilattice $(S, \sqcup)$. Merge is the join operation $\sqcup$, which is:

- **Commutative**: $s_1 \sqcup s_2 = s_2 \sqcup s_1$
- **Associative**: $(s_1 \sqcup s_2) \sqcup s_3 = s_1 \sqcup (s_2 \sqcup s_3)$
- **Idempotent**: $s \sqcup s = s$

These are exactly the algebraic properties guaranteed by the trace monoid structure when operations are independent.

### CALM Theorem

A distributed computation $Q$ is **coordination-free** (can be computed without synchronous barriers) if and only if $Q$ is **monotone**:

$$S \subseteq T \implies Q(S) \subseteq Q(T)$$

This connects order theory to distributed systems: monotone queries on CRDT lattices never need coordination.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
