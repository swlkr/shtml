# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build           # Build all crates
cargo test            # Run all tests
cargo test <name>     # Run a single test by name
cargo test --features chaos  # Run tests including chaos feature tests
```

CI runs `cargo build --verbose && cargo test --verbose` on push/PR to main.

## Architecture

**shtml** is a Rust library for server-side HTML rendering using a JSX-like macro syntax. It's a `no_std` crate (uses `alloc`).

### Two-crate workspace:

- **`shtml`** (`src/lib.rs`) — Public API: `Component` struct (wraps rendered HTML string), `Elements` type alias (children), `Render` trait (implemented for primitives, strings, `Vec<T>`), `escape()` function, and re-exports the `html!` macro.
- **`shtml_macros`** (`shtml_macros/src/lib.rs`) — Proc macro crate: `html!` macro parses JSX-like syntax via `rstml`, recursively renders nodes into an `Output` struct that combines static string segments with dynamic token streams. `chaos.rs` implements the `#[component]` attribute macro (behind `chaos` feature flag) which transforms functions into a struct plus a builder, enabling flexible attribute ordering and skippable optional props. In chaos mode `html!` emits a builder chain (`Fn::builder().attr(val)....build()`) rather than a positional call.

### Key patterns:

- **Components** are PascalCase functions returning `Component`. They receive typed attributes as parameters and optionally an `elements: Elements` parameter for children.
- **Render trait** is the core abstraction — anything rendered inside `html!` must implement it. String content is automatically HTML-escaped; `Component` content is not (already rendered).
- **Spread attributes**: `{..expr}` where expr evaluates to `Vec<(T, T)>` of key-value pairs.
- **Fragments**: `<>...</>` for grouping without a wrapper element.
- **Optional props** (chaos only): a `#[component]` parameter typed `Option<T>` can be omitted at the call site (defaults to `None`) or passed as `Some(value)`. Non-`Option` params stay required and panic at `build()` if omitted. `Option<&str>` is unsupported (needs a named lifetime); use `Option<String>`.
