# arena-lang &mdash; Engineering Directives

> Engineering standards and the definition of done for this project. Read alongside `REPS.md` (root, authoritative) and `dev/ROADMAP.md` (current phase). If anything here conflicts with `REPS.md`, `REPS.md` wins.

---

## 0. Philosophy

This library is built and maintained to a production standard and treated as a flagship piece of work. Plan the full path, then build one verified step at a time. "Good enough" is treated as a defect. arena-lang sits directly beneath the AST and IR: a single parse allocates one node per token and then never frees them individually, so the cost of one allocation and the stability of the address it returns are multiplied across an entire tree. A wrong address here is a dangling reference in every pass that walks the tree.

---

## 1. What this is

arena-lang is a typed bump allocator for compiler tree nodes. It hands out a node's storage from a contiguous chunk, returns a stable handle that never moves for the life of the arena, and frees everything at once when the arena is dropped. It is the allocation substrate beneath `ast-lang` and the IR: nodes are allocated forward, addresses stay put as the arena grows, and there is no per-node deallocation. It owns arena allocation and stable addressing only — no tree shape, no traversal, no parsing.

---

## 2. Engineering law (non-negotiable)

- **Performance** — peak is the baseline; allocating a node is a pointer bump in the common case, not a general-purpose `malloc`; an already-allocated handle resolves without a bounds-checked walk; growing the arena never copies or moves a previously handed-out node; no "faster" claim without `criterion` numbers.
- **Correctness** — the invariants in section 4 are covered by property tests, cross-checked against a `Vec`-backed reference arena.
- **Security** — capacity is bounded and exhaustion is a defined, non-panicking outcome; resolving a handle never reads out of bounds; index and offset arithmetic uses checked/`usize` math that cannot overflow into another node's storage; hostile input (many nodes, large nodes) is handled without UB.
- **Architecture** — SOLID, KISS, YAGNI; one responsibility; the typed-arena surface sits behind one seam over the underlying chunk store.
- **Cross-platform** — Linux/macOS/Windows first-class, verified by CI; alignment and layout handling is explicit, not platform-assumed.
- **Error handling** — every fallible path (capacity exhaustion, resolving a foreign handle) returns `Result`/`Option` per the documented contract; nothing is silently wrong.
- **Production-ready** — `#![forbid(unsafe_code)]`, or every `unsafe` carries a `// SAFETY:` proof and a `# Safety` doc section if the contiguous store requires raw pointers; `#![deny(missing_docs)]` from the first commit; no stray `println!`/`dbg!`; every public item has rustdoc with a runnable example.

---

## 3. Definition of done

1. Compiles clean on Linux/macOS/Windows, stable and MSRV 1.85.
2. `fmt`, `clippy -D warnings`, `test --all-features`, `cargo doc -D warnings` clean.
3. `cargo audit` + `cargo deny check` pass.
4. No `unwrap`/`expect`/`todo!`/`dbg!` in shipping code; any `unsafe` justified with `// SAFETY:`.
5. A Tier-1 API exists and headlines the docs.
6. Property tests cover every section-4 invariant.
7. Hot-path changes carry benchmarks; no regression over 5%.
8. Docs and `CHANGELOG.md` updated; the matching `docs/release/vX.Y.Z.md` written before the tag.

---

## 4. Project-specific invariants

- A handle returned by the arena resolves to the same node for the entire lifetime of the arena that issued it; allocating more nodes, including growth that adds a new chunk, never moves or invalidates a previously issued handle.
- Two distinct allocations return two distinct, non-aliasing handles; no two live nodes share storage.
- `get(alloc(node)) == &node` for every node the arena accepts — round-trip fidelity, property-tested against a reference arena.
- A handle is small and `Copy`; passing one is never more expensive than passing an integer index.
- Capacity exhaustion is reported as a defined error, never a silent wrap or a panic; resolving a handle never reads outside the arena's storage.
