# arena-lang &mdash; API Reference

> Complete reference for every public item in `arena-lang`, with examples.
> **Status: stable (1.0).** The surface below is frozen under
> [Semantic Versioning](https://semver.org): within the `1.x` series no public item
> is removed or changed incompatibly. New items may still arrive as minor releases;
> any breaking change waits for `2.0`. See [`dev/ROADMAP.md`](../dev/ROADMAP.md).

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick start](#quick-start)
- [The model](#the-model)
- [`Arena`](#arena)
  - [`Arena::new`](#arenanew)
  - [`Arena::with_capacity`](#arenawith_capacity)
  - [`Arena::reserve`](#arenareserve)
  - [`Arena::alloc`](#arenaalloc)
  - [`Arena::try_alloc`](#arenatry_alloc)
  - [`Arena::get`](#arenaget)
  - [`Arena::get_mut`](#arenaget_mut)
  - [`Arena::contains`](#arenacontains)
  - [`Arena::len` / `is_empty` / `capacity`](#arenalen--is_empty--capacity)
  - [`Arena::iter`](#arenaiter)
- [`Id`](#id)
- [`ArenaError`](#arenaerror)
- [Feature flags](#feature-flags)

---

## Overview

arena-lang is the allocation floor a tree of compiler nodes is built on. It offers
one focused, append-only surface: allocate a value into an [`Arena`](#arena) and get
back an [`Id`](#id) — a four-byte, `Copy`, type-tagged handle that stays valid for the
life of the arena.

The handle, not a raw pointer, is the stable address: a node stores `Id`s pointing
at its children, so a tree is wired by handle and never tangles the borrow checker.
Values are never freed individually; the whole arena is released at once when it is
dropped. It owns typed allocation and stable addressing only — no tree shape, no
traversal, no parsing.

---

## Installation

```toml
[dependencies]
arena-lang = "1"
```

Or from the terminal:

```bash
cargo add arena-lang
```

The crate is `no_std`-friendly: it needs `alloc` but not the full standard library.
Disable the default `std` feature for a `no_std` build.

---

## Quick start

```rust
use arena_lang::{Arena, Id};

enum Expr {
    Int(i64),
    Add(Id<Expr>, Id<Expr>),
}

let mut arena = Arena::new();
let one = arena.alloc(Expr::Int(1));
let two = arena.alloc(Expr::Int(2));
let sum = arena.alloc(Expr::Add(one, two)); // stores handles to its children

// Child handles still resolve after the parent was allocated.
if let Some(Expr::Add(l, r)) = arena.get(sum) {
    assert!(matches!(arena.get(*l), Some(Expr::Int(1))));
    assert!(matches!(arena.get(*r), Some(Expr::Int(2))));
}
```

---

## The model

An `Arena<T>` holds many values of one type. [`alloc`](#arenaalloc) appends a value
and returns an [`Id<T>`](#id); [`get`](#arenaget) resolves a handle back to the
value in a single slot lookup — constant time, no search. A handle stays valid for
the life of the arena and keeps resolving to the same value through every later
allocation, so a node can hold handles to other nodes and the graph never moves.

The arena is append-only: there is no per-value free, which is the allocation
pattern an AST or IR wants — nodes are created forward during a pass and released
together when the arena drops. Handles are addressed by a 32-bit slot counter, so an
arena holds up to `u32::MAX` values over its lifetime; reaching that ceiling is the
[`ArenaError`](#arenaerror) reported by [`try_alloc`](#arenatry_alloc).

---

## `Arena`

`Arena<T>` is the type you construct, allocate into, and query. It implements
`Default` (equivalent to [`new`](#arenanew)) and `Debug` (which prints the arena's
length and capacity, not its contents, so it carries no `T: Debug` bound).

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.alloc("root");
assert_eq!(arena.get(id), Some(&"root"));
assert_eq!(arena.len(), 1);
```

### `Arena::new`

```rust
pub const fn new() -> Arena<T>
```

Creates an empty arena. `const`, so it can initialise a `static` or `const`.

```rust
use arena_lang::Arena;

let arena: Arena<u32> = Arena::new();
assert!(arena.is_empty());
```

### `Arena::with_capacity`

```rust
pub fn with_capacity(capacity: usize) -> Arena<T>
```

Creates an empty arena with room for `capacity` values preallocated.

**Parameters**

- `capacity` — the number of values to reserve backing storage for. A hint only:
  the first `capacity` allocations will not reallocate. The arena still starts
  empty.

```rust
use arena_lang::Arena;

let mut arena = Arena::with_capacity(3);
let ids = [arena.alloc('a'), arena.alloc('b'), arena.alloc('c')];
assert_eq!(ids.map(|id| *arena.get(id).unwrap()), ['a', 'b', 'c']);
```

### `Arena::reserve`

```rust
pub fn reserve(&mut self, additional: usize)
```

Reserves capacity for at least `additional` more values, folding several
incremental growths into one before a burst of allocations.

**Parameters**

- `additional` — the number of further values to make room for.

```rust
use arena_lang::Arena;

let mut arena: Arena<u8> = Arena::new();
arena.reserve(128);
assert!(arena.capacity() >= 128);
```

### `Arena::alloc`

```rust
pub fn alloc(&mut self, value: T) -> Id<T>
```

Allocates `value` and returns a stable [`Id`](#id) handle. This is the hot path; the
handle is valid for the life of the arena.

**Parameters**

- `value` — the value to store. Ownership moves into the arena.

**Panics**

Panics only if the arena has already allocated `u32::MAX` values — a ceiling of more
than four billion live nodes, unreachable for any real tree. Use
[`try_alloc`](#arenatry_alloc) for an explicit non-panicking path.

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.alloc(42);
assert_eq!(arena.get(id), Some(&42));
```

Wiring a node to its children by handle:

```rust
use arena_lang::{Arena, Id};

struct Node { value: i32, next: Option<Id<Node>> }

let mut arena = Arena::new();
let tail = arena.alloc(Node { value: 2, next: None });
let head = arena.alloc(Node { value: 1, next: Some(tail) });

let first = arena.get(head).unwrap();
assert_eq!(first.value, 1);
assert_eq!(arena.get(first.next.unwrap()).unwrap().value, 2);
```

### `Arena::try_alloc`

```rust
pub fn try_alloc(&mut self, value: T) -> Result<Id<T>, ArenaError>
```

The non-panicking counterpart to [`alloc`](#arenaalloc): identical on success, but
returns [`ArenaError::CapacityExhausted`](#arenaerror) instead of panicking at the
`u32::MAX`-value ceiling. Prefer it when building a tree from input whose size you do
not control.

**Parameters**

- `value` — the value to store.

**Errors**

Returns [`ArenaError::CapacityExhausted`](#arenaerror) when the arena's slot space is
full; the arena is left unchanged.

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.try_alloc("ok")?;
assert_eq!(arena.get(id), Some(&"ok"));
# Ok::<(), arena_lang::ArenaError>(())
```

### `Arena::get`

```rust
pub fn get(&self, id: Id<T>) -> Option<&T>
```

Borrows the value behind `id`, or `None` if the handle does not name a live value in
this arena. A direct slot lookup, not a search; the `None` case guards an
out-of-range handle so resolution never reads outside the arena's storage.

**Parameters**

- `id` — a handle from [`alloc`](#arenaalloc) / [`try_alloc`](#arenatry_alloc).

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.alloc(vec![1, 2, 3]);
assert_eq!(arena.get(id).map(Vec::len), Some(3));
```

### `Arena::get_mut`

```rust
pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T>
```

Mutably borrows the value behind `id`, for back-patching a node after it is
allocated — resolving a forward reference, or filling in a parent link.

**Parameters**

- `id` — a handle into this arena.

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.alloc(0_u32);
if let Some(slot) = arena.get_mut(id) {
    *slot = 99;
}
assert_eq!(arena.get(id), Some(&99));
```

### `Arena::contains`

```rust
pub fn contains(&self, id: Id<T>) -> bool
```

Returns `true` if `id` names a live value in this arena.

**Parameters**

- `id` — a handle to test.

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.alloc("x");
assert!(arena.contains(id));
```

### `Arena::len` / `is_empty` / `capacity`

```rust
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn capacity(&self) -> usize
```

The number of values in the arena, whether it holds none, and how many it can hold
before it must grow. Because values are never removed, `len` only grows and equals
the number of handles the arena has issued.

```rust
use arena_lang::Arena;

let mut arena = Arena::with_capacity(4);
assert!(arena.is_empty());
arena.alloc(());
assert_eq!(arena.len(), 1);
assert!(arena.capacity() >= 4);
```

### `Arena::iter`

```rust
pub fn iter(&self) -> impl Iterator<Item = (Id<T>, &T)>
```

Iterates over every value in the arena, paired with its handle. Values are visited in
allocation order — the order their ids were minted — so the first pair is
`(Id 0, first value)`. Useful for a pass that walks all nodes without following the
tree's edges.

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let a = arena.alloc(10);
let b = arena.alloc(20);

// Allocation order, with the matching handles.
let pairs: Vec<_> = arena.iter().collect();
assert_eq!(pairs, vec![(a, &10), (b, &20)]);

let total: i32 = arena.iter().map(|(_, v)| *v).sum();
assert_eq!(total, 30);
```

---

## `Id`

A small, copyable, type-tagged handle to one value in an [`Arena`](#arena). It is a
single `u32` — four bytes, the same as a bare index, for **every** element type — so
passing one is no more expensive than passing an integer. It stays valid for the life
of the arena that issued it.

The `T` tag is compile-time only and occupies no space: it stops an `Id<Expr>` from
being passed where an `Id<Stmt>` is expected. `Id<T>` is `Copy`, `Eq`, `Ord`, and
`Hash` for **every** `T` — the tag never adds a trait bound — so it works as a
`HashMap` / `BTreeMap` key regardless of what it points at. There is no public
constructor: an `Id` can only come from an [`Arena::alloc`](#arenaalloc).

```rust
use arena_lang::{Arena, Id};
use std::collections::HashMap;

let mut arena = Arena::new();
let a = arena.alloc("alpha");
let b = arena.alloc("beta");

// Copy, four bytes, and usable as a map key.
let mut labels: HashMap<Id<&str>, u32> = HashMap::new();
labels.insert(a, 1);
labels.insert(b, 2);
assert_eq!(labels[&a], 1);
assert_ne!(a, b);
assert_eq!(core::mem::size_of_val(&a), 4);
```

---

## `ArenaError`

```rust
#[non_exhaustive]
pub enum ArenaError {
    CapacityExhausted,
}
```

The reason a value could not be allocated, returned by
[`try_alloc`](#arenatry_alloc). The enum is `#[non_exhaustive]`. It implements
`core::error::Error` and `Display`.

**`CapacityExhausted`** — the arena's slot space is full: it already holds `u32::MAX`
values and cannot represent another handle. Unreachable for any realistic tree (more
than four billion live nodes), but reported rather than ignored so the limit is a
defined boundary, never a silent wrap.

```rust
use arena_lang::ArenaError;

assert_eq!(
    ArenaError::CapacityExhausted.to_string(),
    "arena is full: cannot allocate beyond u32::MAX values",
);
```

---

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | yes     | Links the standard library. The crate needs only `alloc`, so this is opt-out: disabling it compiles `arena-lang` under `#![no_std]` with no loss of function. |

Disabling `std` keeps the crate `no_std`:

```toml
[dependencies]
arena-lang = { version = "1", default-features = false }
```

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>
