//! # lau-trace-monoid
//!
//! Mazurkiewicz trace monoids, right-angled Artin groups, and CRDT lattice
//! structures for concurrent computation.
//!
//! This crate implements the mathematical structures underlying lock-free
//! command dispatch: independence relations, trace equivalence, Foata normal
//! forms, prefix orders, parallel composition, right-angled Artin groups,
//! linearization, CRDT join-semilattices, the CALM theorem, and Kleene
//! fixpoints for convergent CRDT states.

pub mod independence;
pub mod trace;
pub mod foata;
pub mod prefix;
pub mod concurrent;
pub mod raag;
pub mod linearize;
pub mod crdt_lattice;
pub mod calm;
pub mod fixpoint;

pub use independence::Independence;
pub use trace::Trace;
pub use foata::FoataNormalForm;
pub use raag::{CommutationGraph, RaagWord};
pub use crdt_lattice::{GCounter, LWWRegister, ORSet, LatticeValue};
pub use calm::CalmAnalysis;
pub use fixpoint::FixpointResult;
