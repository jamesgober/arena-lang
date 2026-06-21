//! The typed handle into an arena.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

/// A small, copyable, type-tagged handle to one value in an [`Arena`](crate::Arena).
///
/// An `Id<T>` is the stable way to refer to an allocated value: it stays valid for
/// the life of the arena that issued it, and it is a single `u32` — four bytes,
/// the same as a bare index — so passing one is no more expensive than passing an
/// integer. Unlike a `&T`, it carries no borrow, so a tree of nodes can store ids
/// pointing at one another without tangling the borrow checker — which is exactly
/// why an AST or IR holds child handles rather than child references.
///
/// The `T` tag is compile-time only (it occupies no space): it stops an
/// `Id<Expr>` from being passed where an `Id<Stmt>` is expected, so a handle can
/// only ever index the arena that produced it. `Id<T>` is `Copy`, `Eq`, `Ord`, and
/// `Hash` for **every** `T` — the tag never adds a bound — so it works as a map key
/// regardless of what it points at.
///
/// Resolve a handle with [`Arena::get`](crate::Arena::get) /
/// [`get_mut`](crate::Arena::get_mut); there is no public constructor, so an `Id`
/// can only come from an [`Arena::alloc`](crate::Arena::alloc).
///
/// # Examples
///
/// ```
/// use arena_lang::Arena;
///
/// let mut arena = Arena::new();
/// let id = arena.alloc("node");
///
/// // The handle round-trips back to the value it named.
/// assert_eq!(arena.get(id), Some(&"node"));
///
/// // It is Copy and four bytes wide, whatever it points at.
/// let also = id;
/// assert_eq!(id, also);
/// assert_eq!(core::mem::size_of_val(&id), 4);
/// ```
pub struct Id<T> {
    raw: u32,
    /// Compile-time type tag. `fn() -> T` keeps `Id<T>` `Copy`, `Send`, and `Sync`
    /// for any `T`, and covariant in `T`, without ever borrowing or owning a `T`.
    marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    /// Wraps a raw arena index. Internal: only an arena mints handles, so that the
    /// type tag always matches the arena the handle indexes.
    #[inline]
    pub(crate) const fn new(raw: u32) -> Self {
        Self {
            raw,
            marker: PhantomData,
        }
    }

    /// Returns the raw arena index this handle wraps.
    #[inline]
    pub(crate) const fn raw(self) -> u32 {
        self.raw
    }
}

// The trait impls are written by hand rather than derived: a derive would bound
// each impl on `T` (e.g. `T: Clone`), but a handle must be `Copy`, comparable, and
// hashable for every `T`, since `T` is only a compile-time tag.

impl<T> Clone for Id<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> Hash for Id<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.raw)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::collections::BTreeSet;
    use alloc::format;

    use crate::Arena;

    #[test]
    fn test_id_traits_do_not_depend_on_the_tag() {
        // A tag that is neither `Clone` nor `Eq` must not stop `Id` from being
        // `Copy`, `Eq`, and `Debug` — the tag is compile-time only.
        struct NotClone;
        let mut arena = Arena::<NotClone>::new();
        let id = arena.alloc(NotClone);
        let copy = id; // Copy
        assert_eq!(id, copy); // Eq
        assert!(format!("{id:?}").starts_with("Id")); // Debug
    }

    #[test]
    fn test_distinct_allocations_have_distinct_ids() {
        let mut arena = Arena::new();
        let ids: BTreeSet<_> = (0..16).map(|i| arena.alloc(i)).collect();
        assert_eq!(ids.len(), 16); // all distinct, usable as ordered-set keys
    }

    #[test]
    fn test_id_is_four_bytes_for_any_element() {
        assert_eq!(core::mem::size_of::<crate::Id<u8>>(), 4);
        assert_eq!(core::mem::size_of::<crate::Id<[u128; 4]>>(), 4);
    }
}
