# jj-starship

**Commit:** 69cca91 | **Branch:** main

## Overview

Rust CLI for Starship shell prompts - displays unified VCS info for JJ and Git repos. Latency-optimized.

## Structure

```
src/
  main.rs      # CLI (clap) + subcommands: prompt, detect, version
  jj.rs        # JJ repo info via jj-lib (274 lines)
  git.rs       # Git repo info via git2 (feature-gated)
  output.rs    # Format/render prompt strings (814 lines - largest)
  config.rs    # Config layering: CLI > env vars > defaults
  detect.rs    # Repo type detection (JJ/Git/colocated)
  color.rs     # ANSI styling
  error.rs     # thiserror types
build.rs       # Embeds JJ_CHANGE_ID, GIT_COMMIT, BUILD_DATE at compile time
```

## Where to Look

| Task | Location |
|------|----------|
| Add CLI flag | `main.rs` (Cli/GitArgs structs) |
| New status symbol | `output.rs` |
| JJ data collection | `jj.rs` (uses jj-lib directly) |
| Git data collection | `git.rs` (uses git2) |
| Config/defaults | `config.rs` (DisplayConfig, DEFAULTS) |
| Repo detection | `detect.rs` |

## Conventions

- **Edition 2024, MSRV 1.85** - bleeding edge; requires nightly or latest stable
- **Feature gates:** `#[cfg(feature = "git")]` throughout - git support optional
- **No lib.rs** - binary-only crate, tests inline
- **Cow<'static, str>** for zero-alloc defaults in config.rs
- **DisplayFlags (negative) -> DisplayConfig (positive)** - CLI uses `--no-*` flags, internal code uses positive bools

## Anti-Patterns

- `unsafe` - denied via lint
- Generic/non-pedantic clippy - `clippy::pedantic` enforced

## Build

build.rs requires `jj` CLI at build time (runs `jj log` commands). Will fail in pure git checkout without JJ initialized.

Rebuild trigger: `.jj/repo/op_heads` changes.

## Commands

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build --release

# JJ-only build (no git2)
cargo build --no-default-features
```

## CI

- `test.yml` - test + lint + build (ubuntu + macos)
- `release.yml` - manual dispatch, cross-compile 8 targets, crates.io + homebrew
- Clippy warnings = errors (`-D warnings`)

## Notes

- output.rs (814 lines) is complexity hotspot - all formatting logic
- Colocated repos (JJ+Git): detected but treated as JJ
- Status symbols derive from closest/first bookmark only
