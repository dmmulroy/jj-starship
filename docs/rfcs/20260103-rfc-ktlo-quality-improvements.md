# RFC: KTLO Quality Improvements Batch

**Status:** Implemented  
**Author:** OpenCode  
**Date:** 2026-01-03  
**Scope:** `src/`

---

## Summary

Five small, low-risk changes to improve code quality, API safety, and test coverage without altering functionality.

---

## Motivation

As jj-starship matures, periodic KTLO (Keep The Lights On) work prevents accumulation of small issues. This batch addresses:

1. **Compile-time safety** — Catch ignored return values
2. **API stability** — Allow enum growth without semver breaks
3. **Performance** — Micro-optimizations in latency-critical paths
4. **Test coverage** — Prevent CLI regressions
5. **Code clarity** — Document intentional lint suppressions

### Non-Goals

- New features
- Behavioral changes
- Large refactors

---

## Changes

### 1. `#[must_use]` Attributes

**Problem:** Public functions return values that callers might accidentally ignore, leading to silent bugs.

**Solution:** Add `#[must_use]` with contextual messages.

| File | Function | Message |
|------|----------|---------|
| `jj.rs` | `collect()` | "returns collected repo info, does not modify state" |
| `git.rs` | `collect()` | "returns collected repo info, does not modify state" |
| `detect.rs` | `detect()` | "returns detection result, does not modify state" |
| `detect.rs` | `in_repo()` | "returns detection result, does not modify state" |
| `output.rs` | `format_jj()` | "returns formatted string, does not print" |
| `output.rs` | `format_git()` | "returns formatted string, does not print" |
| `config.rs` | `truncate()` | "returns truncated string, does not modify input" |
| `config.rs` | `strip_prefix()` | "returns stripped string, does not modify input" |

**Risk:** None. Additive change, compiler warns on misuse.

---

### 2. `#[non_exhaustive]` on Public Enums

**Problem:** Adding variants to public enums is a breaking change under semver.

**Solution:** Mark enums as non-exhaustive to allow future growth.

```rust
// detect.rs
#[non_exhaustive]
pub enum RepoType {
    Jj,
    JjColocated,
    Git,
    None,
}

// error.rs
#[non_exhaustive]
pub enum Error {
    Jj(String),
    #[cfg(feature = "git")]
    Git(String),
    Io(#[from] std::io::Error),
}
```

**Rationale:**
- `RepoType` may expand (Pijul, Fossil, Sapling)
- `Error` may gain variants (config errors, timeout errors)

**Risk:** Low. Downstream match arms must include `_ =>` wildcard. Since this is a binary crate (not a library), impact is internal only.

---

### 3. `String::with_capacity` in Hot Paths

**Problem:** Status string building allocates incrementally on every prompt render.

**Solution:** Pre-allocate based on known max sizes.

```rust
// output.rs - format_jj()
let mut status = String::with_capacity(8);  // "!⇔?⇡" max ~6 chars

// output.rs - format_git()  
let mut status = String::with_capacity(16); // "=+!?✘⇡999⇣999" worst case
```

**Rationale:** Prompt rendering is latency-sensitive. Eliminating 1-3 reallocations per render reduces jitter.

**Risk:** None. Capacity is a hint; oversizing wastes trivial memory, undersizing falls back to realloc.

---

### 4. CLI Argument Parsing Tests

**Problem:** 16+ CLI args with complex interactions (e.g., `--no-symbol` vs `--jj-symbol`) have no test coverage.

**Solution:** Add integration tests using `Cli::try_parse_from()`.

**Test coverage:**
- Default subcommand resolution (`None` → `Prompt`)
- Explicit subcommands: `prompt`, `detect`, `version`
- Global args: `--cwd`, `--truncate-name`, `--id-length`, `--ancestor-bookmark-depth`, `--bookmarks-display-limit`, `--strip-bookmark-prefix`
- Symbol precedence: `--no-symbol` overrides `--jj-symbol`
- Display flags: `--no-jj-prefix`, `--no-jj-name`, `--no-jj-id`, `--no-jj-status`
- Color flags: `--no-color`, `--no-prefix-color`
- Feature-gated git args: `--git-symbol`, `--no-git-*`

**Risk:** None. Test-only changes.

---

### 5. Document `struct_excessive_bools` Suppressions

**Problem:** Five structs suppress `clippy::struct_excessive_bools` without explanation.

**Solution:** Add documentation justifying why bools are appropriate.

| Struct | Justification |
|--------|---------------|
| `JjInfo` | Independent, orthogonal status flags; bitflags add complexity without benefit |
| `Cli` | Bools inherent to clap's `--flag` / `--no-flag` pattern |
| `GitArgs` | Same as `Cli` |
| `DisplayConfig` | 6 orthogonal visibility toggles; clearer than bitset |
| `DisplayFlags` | Mirrors `DisplayConfig` with inverted semantics for CLI |

**Risk:** None. Documentation-only.

---

## Changes Summary

| File | Change Type | Lines |
|------|-------------|-------|
| `src/jj.rs` | `#[must_use]`, documentation | ~5 |
| `src/git.rs` | `#[must_use]` | ~2 |
| `src/detect.rs` | `#[must_use]`, `#[non_exhaustive]` | ~4 |
| `src/error.rs` | `#[non_exhaustive]` | ~1 |
| `src/output.rs` | `#[must_use]`, `with_capacity` | ~4 |
| `src/config.rs` | `#[must_use]`, documentation | ~6 |
| `src/main.rs` | CLI tests, documentation | ~120 |

**Total:** ~140 lines changed, 0 behavioral changes.

---

## Trade-offs

### Alternative: Replace bools with `EnumSet<DisplayPart>`

**Rejected.** Analysis showed:
- Bool fields are truly orthogonal (can be independently true/false)
- `EnumSet`/bitflags require additional dependency or boilerplate
- Cognitive overhead exceeds benefit for 4-6 flags
- clap derives work naturally with bools

Documentation explaining the design decision is preferable to unnecessary abstraction.

---

## Security Considerations

None. All changes are internal; no new external inputs or credential handling.

---

## Migration

No migration required. All changes are backward-compatible.

## Testing

- `cargo test` - 46 tests pass (25 new CLI tests)
- `cargo clippy --all-targets -- -D warnings` - clean
- `cargo fmt --check` - clean

---

## Conclusion

Five surgical improvements that:

1. Catch ignored return values at compile time
2. Allow future enum growth without semver breaks
3. Reduce allocation jitter in hot paths
4. Prevent CLI parsing regressions
5. Document intentional design decisions

All changes are low-risk, non-behavioral, and verified passing CI checks.
