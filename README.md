<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>arena-lang</b>
    <br>
    <sub><sup>ARENA ALLOCATION</sup></sub>
</h1>

<div align="center">
    <a href="https://crates.io/crates/arena-lang"><img alt="Crates.io" src="https://img.shields.io/crates/v/arena-lang"></a>
    <a href="https://crates.io/crates/arena-lang"><img alt="Downloads" src="https://img.shields.io/crates/d/arena-lang?color=%230099ff"></a>
    <a href="https://docs.rs/arena-lang"><img alt="docs.rs" src="https://img.shields.io/docsrs/arena-lang"></a>
    <a href="https://github.com/jamesgober/arena-lang/actions"><img alt="CI" src="https://github.com/jamesgober/arena-lang/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        arena-lang is a typed bump allocator for compiler tree nodes. It hands a node's storage out of a contiguous chunk, returns a stable handle that never moves for the life of the arena, and frees everything at once when the arena is dropped. It is the allocation substrate beneath the AST and IR: nodes are allocated forward, their addresses stay put as the arena grows, and there is no per-node deallocation to track.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.85+</strong> (Rust 2024 edition).
    </p>
    <blockquote>
        <strong>Status: stable.</strong> The public API is frozen at <code>1.0.0</code> under Semantic Versioning &mdash; no breaking change before <code>2.0</code>. See <a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

## Installation

```toml
[dependencies]
arena-lang = "1"
```

Or from the terminal:

```bash
cargo add arena-lang
```

<br>

## Usage

Allocate a value into the arena and get back a stable, `Copy` handle. The handle —
not a raw pointer — is the stable address, so a tree of nodes is wired by handle and
never tangles the borrow checker.

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

Back-patch a node after it exists, and walk every node without following the edges:

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.alloc(0_u32);
if let Some(slot) = arena.get_mut(id) {
    *slot = 99;
}
assert_eq!(arena.get(id), Some(&99));

let total: u32 = arena.iter().map(|(_, v)| *v).sum();
assert_eq!(total, 99);
```

The fallible path returns a defined error instead of panicking at the `u32::MAX`
node ceiling:

```rust
use arena_lang::Arena;

let mut arena = Arena::new();
let id = arena.try_alloc("ok")?;
assert_eq!(arena.get(id), Some(&"ok"));
# Ok::<(), arena_lang::ArenaError>(())
```

See <a href="./docs/API.md"><code>docs/API.md</code></a> for the full reference.

<br>

## How it works

An <code>Arena&lt;T&gt;</code> stores its values end to end in one contiguous buffer.
Allocation appends a value and returns its position as a 32-bit
<code>Id&lt;T&gt;</code>; resolving one is a single indexed lookup, constant time, no
search. Because values are only appended — never moved or individually freed — a
handle stays valid for the life of the arena and keeps resolving to the same value
through every later allocation, so nodes can reference one another by handle and the
tree never moves. The whole arena is released at once when it drops, which is the
allocation pattern an AST or IR wants. The handle is four bytes whatever it points
at, so the space addressed by a single arena is capped at <code>u32::MAX</code>
values; overrunning it is a defined <code>ArenaError</code>, never a silent wrap.

<br>

## Status

<code>v1.0.0</code> &mdash; <strong>stable.</strong> The full surface is in place: the
typed <code>Arena&lt;T&gt;</code>, the four-byte <code>Copy</code>
<code>Id&lt;T&gt;</code> handle, and the fallible <code>try_alloc</code> path, each
invariant property-tested against a <code>Vec</code>-backed reference arena. The
public API is frozen under Semantic Versioning &mdash; no breaking change before
<code>2.0</code>; see the <a href="./dev/ROADMAP.md"><code>ROADMAP</code></a>.

<hr>
<br>

## Contributing

See <a href="./dev/DIRECTIVES.md"><code>dev/DIRECTIVES.md</code></a> for engineering standards and the definition of done. Before a PR: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` must be clean.

<br>

<div id="license">
    <h2>License</h2>
    <p>Licensed under either of</p>
    <ul>
        <li><b>Apache License, Version 2.0</b> &mdash; <a href="./LICENSE-APACHE">LICENSE-APACHE</a></li>
        <li><b>MIT License</b> &mdash; <a href="./LICENSE-MIT">LICENSE-MIT</a></li>
    </ul>
    <p>at your option.</p>
</div>

<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>James Gober <me@jamesgober.com>.</strong></sup>
</div>
