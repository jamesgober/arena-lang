# arena-lang - Roadmap

> Path from scaffold to a stable 1.0. Hard parts are front-loaded; each phase has hard exit criteria.
> Master plan: ../../_strategy/LANG_COLLECTION.md
>
> **Anti-deferral rule:** no listed hard task moves to a later phase unless this file records the move and the reason.

## v0.1.0 - Scaffold (DONE)
Compiles, CI green, structure correct, no domain logic.
- [x] Manifest, README, CHANGELOG, REPS, dual license, CI, deny, clippy, rustfmt.

## v0.2.0 - Core (THE HARD PART, NOT DEFERRED) (DONE)
A typed, append-only arena giving ast-lang stable, non-moving node addresses: a
contiguous store handing back a four-byte `Id<T>` that survives growth, with a
bounded capacity and a defined exhaustion error. Built with zero first-party
dependencies on safe Rust (`#![forbid(unsafe_code)]`).
Exit criteria:
- [x] Every public item has rustdoc + a runnable example.
- [x] Core invariants property-tested against a `Vec`-backed reference arena (full DIRECTIVES + API authored at this stage).

## v1.0.0 - API freeze (DONE)
Public surface stable and frozen until 2.0. No functional change from v0.2.0; the
reserved no-op `serde` feature was removed rather than frozen into the contract.
- [x] docs/API.md marked stable; SemVer promise recorded.
- [x] Full test + benchmark suite green on all three platforms.
