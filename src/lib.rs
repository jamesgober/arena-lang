//! # arena_lang
//!
//! Typed bump/arena allocation for AST and IR nodes with stable addresses.
//!
//! `arena-lang` is the allocation floor a tree of compiler nodes is built on. It
//! offers one focused, append-only surface: allocate a value into an [`Arena`] and
//! get back an [`Id`] — a four-byte, `Copy`, type-tagged handle that stays valid
//! for the life of the arena and keeps resolving to the same value no matter how
//! many later allocations grow it.
//!
//! The handle, not a raw pointer, is the stable address. A node stores `Id`s
//! pointing at its children, so a tree is wired by handle and never tangles the
//! borrow checker — the pattern an AST or IR is built on. Values are never freed
//! individually; the whole arena is released at once when it is dropped.
//!
//! It owns typed allocation and stable addressing only — no tree shape, no
//! traversal, no parsing.
//!
//! ## Quickstart
//!
//! ```
//! use arena_lang::{Arena, Id};
//!
//! enum Expr {
//!     Int(i64),
//!     Neg(Id<Expr>),
//! }
//!
//! let mut arena = Arena::new();
//! let five = arena.alloc(Expr::Int(5));
//! let neg = arena.alloc(Expr::Neg(five)); // stores a handle to `five`
//!
//! // The child handle still resolves after the parent was allocated.
//! if let Some(Expr::Neg(inner)) = arena.get(neg) {
//!     assert!(matches!(arena.get(*inner), Some(Expr::Int(5))));
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

extern crate alloc;

mod arena;
mod error;
mod id;

pub use arena::Arena;
pub use error::ArenaError;
pub use id::Id;
