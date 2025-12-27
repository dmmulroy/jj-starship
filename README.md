# jj-starship

<img width="1274" height="316" alt="CleanShot 2025-12-26 at 20 46 16@2x" src="https://github.com/user-attachments/assets/940c74fc-f572-4a26-b1ce-d27b70cc3385" />





Unified [Starship](https://starship.rs) prompt module for Git and [Jujutsu](https://github.com/jj-vcs/jj) repositories that is optimized for latency.

## Installation

### Homebrew (macOS)

```sh
brew install dmmulroy/tap/jj-starship
```

### Cargo

```sh
cargo install jj-starship
```

### Build from source

```sh
git clone https://github.com/dmmulroy/jj-starship
cd jj-starship
cargo install --path .
```

### Nix

```sh
# Try it
nix run github:dmmulroy/jj-starship

# Install to profile
nix profile install github:dmmulroy/jj-starship

# Minimal build (no git support, smaller closure)
nix run github:dmmulroy/jj-starship#jj-starship-no-git
```

Or add to your flake inputs:

```nix
{
  inputs.jj-starship.url = "github:dmmulroy/jj-starship";

  outputs = { self, nixpkgs, jj-starship, ... }: {
    # Use the overlay
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [{
        nixpkgs.overlays = [ jj-starship.overlays.default ];
        environment.systemPackages = [ pkgs.jj-starship ];
      }];
    };
  };
}
```

## Feature Flags

The `git` feature is enabled by default. Disable to compile out the git backend:

```sh
# JJ only (excludes git2 dependency)
cargo install --no-default-features jj-starship
```

## Starship Configuration

Add to `~/.config/starship.toml`:

```toml
[custom.jj]
command = "jj-starship"
detect_folders = [".jj", ".git"]
shell = ["sh"]
format = "$output "
```

**Why these settings:**
- `detect_folders` - Starship checks directory existence natively (no process spawn), only runs command in repos
- `shell = ["sh"]` - Uses minimal POSIX shell, bypasses user's shell rc files (~100ms faster)
- `format = "$output "` - Passes through jj-starship's ANSI colors directly

For JJ repos only (keep native git modules for pure git repos):
```toml
detect_folders = [".jj"]
```

To hide built-in modules when in a JJ repo:

```toml
[git_branch]
disabled = true

[git_status]
disabled = true
```

## Output Format

### JJ Format

```
on {symbol}{change_id} ({bookmarks}) [{status}]
```

- `{change_id}` - Short change ID (hide with `--no-jj-id`)
- `{bookmarks}` - Comma-separated bookmarks with distance, sorted by proximity (hide with `--no-jj-name`)
  - Distance 0 (bookmark on WC): `main`
  - Ancestor bookmark: `main~3` (3 commits behind)
- `{status}` - Sync status based on **first/closest** bookmark only

Examples:
- `on 󱗆 yzxv1234 [?]` - No bookmarks
- `on 󱗆 yzxv1234 (main) [?]` - On bookmark `main`
- `on 󱗆 yzxv1234 (main~3) [?]` - 3 commits ahead of `main`
- `on 󱗆 yzxv1234 (pr-3, pr-2~1, main~5)` - Direct + ancestor bookmarks

### Git Format

```
on {symbol}{branch} ({commit}) [{status}]
```

### JJ Status Symbols

| Symbol | Meaning |
|--------|---------|
| `!` | Conflict |
| `?` | Empty description |
| `⇔` | Divergent |
| `⇡` | Current or closest bookmark unsynced with remote |

### Git Status Symbols

| Symbol | Meaning |
|--------|---------|
| `=` | Conflicted |
| `+` | Staged |
| `!` | Modified |
| `?` | Untracked |
| `✘` | Deleted |
| `⇡n` | Ahead by n |
| `⇣n` | Behind by n |

## CLI Options

| Option | Description |
|--------|-------------|
| `--cwd <PATH>` | Override working directory |
| `--truncate-name <N>` | Max branch/bookmark name length (0 = unlimited) |
| `--id-length <N>` | Hash display length (default: 8) |
| `--ancestor-bookmark-depth <N>` | Max depth to search for ancestor bookmarks (default: 10, 0 = disabled) |
| `--jj-symbol <S>` | JJ repo symbol (default: `󱗆 `) |
| `--git-symbol <S>` | Git repo symbol (default: ` `) |
| `--no-color` | Disable output styling |
| `--no-symbol` | Disable symbol prefix |
| `--no-jj-prefix` | Hide "on {symbol}" for JJ |
| `--no-jj-name` | Hide bookmark name |
| `--no-jj-id` | Hide change ID |
| `--no-jj-status` | Hide JJ status |
| `--no-git-prefix` | Hide "on {symbol}" for Git |
| `--no-git-name` | Hide branch name |
| `--no-git-id` | Hide commit hash |
| `--no-git-status` | Hide Git status |

## Environment Variables

All options can be set via environment variables (CLI args take precedence):

- `JJ_STARSHIP_TRUNCATE_NAME`
- `JJ_STARSHIP_ID_LENGTH`
- `JJ_STARSHIP_ANCESTOR_BOOKMARK_DEPTH`
- `JJ_STARSHIP_JJ_SYMBOL`
- `JJ_STARSHIP_GIT_SYMBOL`
- `JJ_STARSHIP_NO_JJ_PREFIX`
- `JJ_STARSHIP_NO_JJ_COLOR`
- `JJ_STARSHIP_NO_JJ_NAME`
- `JJ_STARSHIP_NO_JJ_ID`
- `JJ_STARSHIP_NO_JJ_STATUS`
- `JJ_STARSHIP_NO_GIT_PREFIX`
- `JJ_STARSHIP_NO_GIT_COLOR`
- `JJ_STARSHIP_NO_GIT_NAME`
- `JJ_STARSHIP_NO_GIT_ID`
- `JJ_STARSHIP_NO_GIT_STATUS`

## Powerline Prompt

Example configuration in a powerline prompt, for instance [Gruvbox Rainbow](https://starship.rs/presets/gruvbox-rainbow):

```toml
format = """
[](color_orange)\
$os\
$username\
[](bg:color_yellow fg:color_orange)\
$directory\
[](fg:color_yellow bg:color_aqua)\
${custom.jj}\ # <- replace $git_branch $git_status here
[](fg:color_aqua bg:color_blue)\

...
"""

[git_branch]
disabled = true

[git_status]
disabled = true

[custom.jj]
symbol = ""
style = "bg:color_aqua"
format = '[[ $symbol $output ](fg:color_fg0 bg:color_aqua)]($style)'
command = "jj-starship --no-color --no-symbol --no-jj-prefix --no-git-prefix"
detect_folders = [".jj", ".git"]
shell = ["sh"]
```

## License

MIT
