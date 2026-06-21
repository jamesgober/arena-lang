<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>CHANGELOG</b>
</h1>
<p>
  All notable changes to <code>arena-lang</code> will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

---

## [Unreleased]

### Added

### Changed

### Fixed

### Security

---

## [0.2.0] - 2026-06-20

The core: a typed, append-only arena that hands out stable `Copy` handles for AST and
IR nodes, built on safe Rust with no first-party dependencies.

### Added

- `Arena<T>` — a typed arena with `new`, `with_capacity`, `reserve`, `alloc`,
  `try_alloc`, `get`, `get_mut`, `contains`, `len`, `is_empty`, `capacity`, and
  `iter`.
- `Id<T>` — a small, `Copy`, type-tagged handle that stays valid for the life of the
  arena; `Eq`, `Ord`, and `Hash` for every `T`, so it works as a map key.
- `ArenaError` — `#[non_exhaustive]` error type; the `CapacityExhausted` variant is
  returned by `try_alloc` at the `u32::MAX`-value ceiling.
- `Id<T>` is a single `u32` — four bytes for any element type — backed by a
  contiguous `Vec<T>`; `#![forbid(unsafe_code)]` and zero first-party dependencies.
- Property tests (`tests/properties.rs`) checking handle round-trip, distinctness,
  survival across growth, and `iter` completeness against a `Vec`-backed reference
  arena.
- `criterion` benchmarks for the `alloc` and `get` hot paths.

### Changed

- `clippy.toml` MSRV aligned to `1.85` to match `Cargo.toml`.

### Fixed

- `Cargo.toml` `keywords` and `categories` were unquoted barewords, a TOML parse
  error that broke every cargo command.
- `deny.toml` header named the wrong crate.

---

## [0.1.0] - 2026-06-18

Initial scaffold and repository bootstrap. No domain logic yet &mdash; this release establishes the structure, tooling, and quality gates the implementation will be built on.

### Added

- `Cargo.toml` with crate metadata, Rust 2024 edition, MSRV 1.85.
- Dual `Apache-2.0 OR MIT` license files.
- `README.md`, `CHANGELOG.md`, and a documentation skeleton.
- `REPS.md` compliance baseline.
- `.github/workflows/ci.yml` CI matrix; `deny.toml`, `clippy.toml`, `rustfmt.toml`.
- `dev/DIRECTIVES.md` and `dev/ROADMAP.md` (committed engineering standards + plan).

[Unreleased]: https://github.com/jamesgober/arena-lang/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jamesgober/arena-lang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/arena-lang/releases/tag/v0.1.0
