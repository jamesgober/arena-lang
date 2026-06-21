//! Property tests for the arena invariants.
//!
//! The arena is checked against a deliberately naive `Vec`-backed reference: a
//! model that allocates by pushing and resolves by indexing. The real arena is
//! correct only if a handle resolves to the same value the model stored, for every
//! sequence of allocations.

#![allow(clippy::unwrap_used)]

use arena_lang::{Arena, Id};
use proptest::prelude::*;

/// The reference model: allocation is a push, resolution is an index. Obviously
/// correct, and the oracle the real arena must match value-for-value.
#[derive(Default)]
struct RefArena<T> {
    values: Vec<T>,
}

impl<T> RefArena<T> {
    fn alloc(&mut self, value: T) -> usize {
        let at = self.values.len();
        self.values.push(value);
        at
    }

    fn get(&self, at: usize) -> Option<&T> {
        self.values.get(at)
    }
}

proptest! {
    /// Every handle resolves to exactly the value the model stored at the same
    /// step — round-trip fidelity across an arbitrary allocation sequence.
    #[test]
    fn handles_match_the_reference_model(values in prop::collection::vec(any::<i64>(), 0..200)) {
        let mut arena = Arena::new();
        let mut model = RefArena::default();

        // Allocate the same sequence into both.
        let handles: Vec<(Id<i64>, usize)> = values
            .iter()
            .map(|&v| (arena.alloc(v), model.alloc(v)))
            .collect();

        // Every handle still resolves to the model's value for that step.
        for (&v, (id, at)) in values.iter().zip(&handles) {
            prop_assert_eq!(arena.get(*id), model.get(*at));
            prop_assert_eq!(arena.get(*id), Some(&v));
        }

        prop_assert_eq!(arena.len(), values.len());
    }

    /// Distinct allocations yield distinct handles; no two live values share one.
    #[test]
    fn distinct_allocations_have_distinct_handles(n in 0usize..300) {
        let mut arena = Arena::new();
        let ids: Vec<Id<usize>> = (0..n).map(|i| arena.alloc(i)).collect();

        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        prop_assert_eq!(sorted.len(), ids.len()); // no duplicates
    }

    /// A handle issued early keeps resolving to its original value through every
    /// later allocation — growth never invalidates or moves an existing handle.
    #[test]
    fn early_handles_survive_later_growth(
        head in any::<u32>(),
        rest in prop::collection::vec(any::<u32>(), 0..200),
    ) {
        let mut arena = Arena::new();
        let first = arena.alloc(head);
        for &v in &rest {
            let _ = arena.alloc(v);
        }
        prop_assert_eq!(arena.get(first), Some(&head));
        prop_assert!(arena.contains(first));
    }

    /// `iter` visits every allocated value exactly once.
    #[test]
    fn iter_is_a_permutation_of_all_values(values in prop::collection::vec(any::<i32>(), 0..200)) {
        let mut arena = Arena::new();
        for &v in &values {
            let _ = arena.alloc(v);
        }

        let mut seen: Vec<i32> = arena.iter().map(|(_, v)| *v).collect();
        let mut expected = values.clone();
        seen.sort_unstable();
        expected.sort_unstable();
        prop_assert_eq!(seen, expected);
    }
}
