# jj-starship
<img width="350" height="350" alt="image" src="https://github.com/user-attachments/assets/73d68dbf-1ce8-4ed3-87cd-d7446a1ca8b4" />


Unified [Starship](https://starship.rs) prompt module for Git and [Jujutsu](https://github.com/jj-vcs/jj) repositories that is optimized for latency.

## Installation

```sh
cargo install jj-starship
```

Or build from source:

```sh
git clone https://github.com/dmmulroy/jj-starship
cd jj-starship
cargo install --path .
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
when = "jj-starship detect"
```

To hide built-in modules when in a JJ repo:

```toml
[git_branch]
disabled = true

[git_status]
disabled = true
```

### Powerline Prompt

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
when = "jj-starship detect"
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

## License

MIT
