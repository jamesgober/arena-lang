//! The typed, append-only arena.

use alloc::vec::Vec;
use core::fmt;

use crate::{ArenaError, Id};

/// A typed arena that allocates values and hands back stable [`Id`] handles.
///
/// `Arena<T>` is the allocation floor a tree of nodes is built on. Allocate a
/// value with [`alloc`](Arena::alloc) and you get an [`Id<T>`](Id) — a small,
/// `Copy`, type-tagged handle that stays valid for the life of the arena. Resolve
/// it back to the value with [`get`](Arena::get). Values are never freed
/// individually; the whole arena is released at once when it is dropped, which is
/// the allocation pattern an AST or IR wants — many nodes allocated forward during
/// a pass, none removed, all gone together at the end.
///
/// The handle, not a raw pointer, is the stable address: storing an `Id<T>` in one
/// node to point at another is how a tree is wired, and the handle keeps resolving
/// to the same value no matter how many later allocations grow the arena.
///
/// # Examples
///
/// ```
/// use arena_lang::Arena;
///
/// // A tiny expression tree, wired by handle.
/// enum Expr {
///     Int(i64),
///     Add(arena_lang::Id<Expr>, arena_lang::Id<Expr>),
/// }
///
/// let mut arena = Arena::new();
/// let one = arena.alloc(Expr::Int(1));
/// let two = arena.alloc(Expr::Int(2));
/// let sum = arena.alloc(Expr::Add(one, two));
///
/// // The parent's handles still resolve after it was itself allocated.
/// match arena.get(sum) {
///     Some(Expr::Add(l, r)) => {
///         assert!(matches!(arena.get(*l), Some(Expr::Int(1))));
///         assert!(matches!(arena.get(*r), Some(Expr::Int(2))));
///     }
///     _ => unreachable!(),
/// }
/// ```
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    /// Creates an empty arena. `const`, so it can initialise a `static`.
    ///
    /// No allocation happens until the first value is added.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let arena: Arena<u32> = Arena::new();
    /// assert!(arena.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Creates an empty arena with room for `capacity` values preallocated.
    ///
    /// A hint only: it reserves backing storage so the first `capacity`
    /// allocations do not reallocate. Sizing it to the expected node count — for
    /// instance, a multiple of the token count of the source — keeps allocation on
    /// the flat part of the cost curve.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::with_capacity(3);
    /// let ids = [arena.alloc('a'), arena.alloc('b'), arena.alloc('c')];
    /// assert_eq!(arena.len(), 3);
    /// assert_eq!(ids.map(|id| *arena.get(id).unwrap()), ['a', 'b', 'c']);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Reserves capacity for at least `additional` more values.
    ///
    /// Use it before a burst of allocations whose count is known, to fold what
    /// would be several incremental growths into one.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena: Arena<u8> = Arena::new();
    /// arena.reserve(128);
    /// assert!(arena.capacity() >= 128);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
    }

    /// Allocates `value` and returns a stable [`Id`] handle to it.
    ///
    /// This is the hot path. The handle is valid for the life of the arena and
    /// keeps resolving to this value through every later allocation.
    ///
    /// # Panics
    ///
    /// Panics only if the arena has already allocated `u32::MAX` values and cannot
    /// represent another handle — a ceiling of more than four billion live nodes,
    /// unreachable for any real tree. Use [`try_alloc`](Arena::try_alloc) for an
    /// explicit non-panicking path.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let id = arena.alloc(42);
    /// assert_eq!(arena.get(id), Some(&42));
    /// ```
    #[inline]
    pub fn alloc(&mut self, value: T) -> Id<T> {
        match self.try_alloc(value) {
            Ok(id) => id,
            Err(_) => panic!("arena is full: cannot allocate beyond u32::MAX values"),
        }
    }

    /// Allocates `value`, returning its [`Id`] or an error if the arena is full.
    ///
    /// The non-panicking counterpart to [`alloc`](Arena::alloc): identical on
    /// success, but it returns [`ArenaError::CapacityExhausted`] instead of
    /// panicking at the `u32::MAX`-value ceiling. Prefer it when building a tree
    /// from untrusted input whose size you do not control.
    ///
    /// # Errors
    ///
    /// Returns [`ArenaError::CapacityExhausted`] when the arena's slot space is
    /// full. The arena is left unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let id = arena.try_alloc("ok")?;
    /// assert_eq!(arena.get(id), Some(&"ok"));
    /// # Ok::<(), arena_lang::ArenaError>(())
    /// ```
    #[inline]
    pub fn try_alloc(&mut self, value: T) -> Result<Id<T>, ArenaError> {
        // The next handle is the current length; if that no longer fits in a
        // `u32`, the arena is full. Checked before the push, so a rejected
        // allocation leaves the arena untouched.
        let raw = u32::try_from(self.items.len()).map_err(|_| ArenaError::CapacityExhausted)?;
        self.items.push(value);
        Ok(Id::new(raw))
    }

    /// Borrows the value behind `id`, or `None` if the handle does not name a live
    /// value in this arena.
    ///
    /// Resolution is a direct slot lookup, not a search. A handle from this arena
    /// always resolves; the `None` case guards against an out-of-range handle, so
    /// resolving one never reads outside the arena's storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let id = arena.alloc(vec![1, 2, 3]);
    /// assert_eq!(arena.get(id).map(Vec::len), Some(3));
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.items.get(id.raw() as usize)
    }

    /// Mutably borrows the value behind `id`, or `None` if the handle does not name
    /// a live value in this arena.
    ///
    /// Useful for back-patching a node after it is allocated — resolving a forward
    /// reference, or filling in a parent link once the parent exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let id = arena.alloc(0_u32);
    /// if let Some(slot) = arena.get_mut(id) {
    ///     *slot = 99;
    /// }
    /// assert_eq!(arena.get(id), Some(&99));
    /// ```
    #[inline]
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        self.items.get_mut(id.raw() as usize)
    }

    /// Returns `true` if `id` names a live value in this arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let id = arena.alloc("x");
    /// assert!(arena.contains(id));
    /// ```
    #[inline]
    #[must_use]
    pub fn contains(&self, id: Id<T>) -> bool {
        (id.raw() as usize) < self.items.len()
    }

    /// Returns the number of values in the arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// assert_eq!(arena.len(), 0);
    /// arena.alloc(());
    /// assert_eq!(arena.len(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the arena holds no values.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// assert!(arena.is_empty());
    /// arena.alloc(());
    /// assert!(!arena.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of values the arena can hold before it must grow.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let arena: Arena<u64> = Arena::with_capacity(8);
    /// assert!(arena.capacity() >= 8);
    /// ```
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// Iterates over every value in the arena, paired with its handle.
    ///
    /// Values are visited in allocation order — the order their ids were minted —
    /// so the first pair is `(Id 0, first value)`. Useful for a pass that walks all
    /// nodes without following the tree's edges.
    ///
    /// # Examples
    ///
    /// ```
    /// use arena_lang::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let a = arena.alloc(10);
    /// let b = arena.alloc(20);
    ///
    /// // Allocation order, with the matching handles.
    /// let pairs: Vec<_> = arena.iter().collect();
    /// assert_eq!(pairs, vec![(a, &10), (b, &20)]);
    ///
    /// let total: i32 = arena.iter().map(|(_, v)| *v).sum();
    /// assert_eq!(total, 30);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (Id<T>, &T)> + '_ {
        self.items
            .iter()
            .enumerate()
            .map(|(i, value)| (Id::new(i as u32), value))
    }
}

impl<T> Default for Arena<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for Arena<T> {
    /// Shows the arena's size, not its contents — an arena can hold millions of
    /// nodes, and dumping them all is rarely what a debug print wants. This also
    /// keeps `Arena<T>: Debug` free of any `T: Debug` bound.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arena")
            .field("len", &self.items.len())
            .field("capacity", &self.items.capacity())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn test_alloc_then_get_round_trips() {
        let mut arena = Arena::new();
        let id = arena.alloc("payload");
        assert_eq!(arena.get(id), Some(&"payload"));
        assert!(arena.contains(id));
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn test_handles_stay_valid_as_the_arena_grows() {
        let mut arena = Arena::new();
        let first = arena.alloc(1);
        for n in 2..=1000 {
            let _ = arena.alloc(n);
        }
        // The very first handle still resolves to its original value.
        assert_eq!(arena.get(first), Some(&1));
        assert_eq!(arena.len(), 1000);
    }

    #[test]
    fn test_get_mut_back_patches_in_place() {
        let mut arena = Arena::new();
        let id = arena.alloc(0_u32);
        *arena.get_mut(id).expect("live handle") = 7;
        assert_eq!(arena.get(id), Some(&7));
    }

    #[test]
    fn test_out_of_range_handle_resolves_to_none() {
        // A handle minted by a larger arena names a slot the small one lacks.
        let mut big = Arena::new();
        let mut last = big.alloc(0);
        for n in 1..50 {
            last = big.alloc(n);
        }
        let small: Arena<i32> = Arena::new();
        assert_eq!(small.get(last), None);
        assert!(!small.contains(last));
    }

    #[test]
    fn test_iter_visits_every_value_once() {
        let mut arena = Arena::new();
        let mut expected = Vec::new();
        for n in 0..32 {
            let _ = arena.alloc(n);
            expected.push(n);
        }
        let mut seen: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
        seen.sort_unstable();
        assert_eq!(seen, expected);
    }

    #[test]
    fn test_try_alloc_succeeds_on_a_fresh_arena() {
        let mut arena = Arena::new();
        let id = arena.try_alloc(123).expect("first slot is free");
        assert_eq!(arena.get(id), Some(&123));
    }

    #[test]
    fn test_default_is_empty() {
        let arena: Arena<u8> = Arena::default();
        assert!(arena.is_empty());
        assert!(arena.capacity() >= arena.len());
    }
}
