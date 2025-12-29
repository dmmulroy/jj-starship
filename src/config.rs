//! Configuration for jj-starship

use std::borrow::Cow;
use std::env;

/// Default symbol for JJ repos
pub const DEFAULT_JJ_SYMBOL: &str = "󱗆 ";
/// Default symbol for Git repos
pub const DEFAULT_GIT_SYMBOL: &str = " ";

/// Display options for a repo type
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct DisplayConfig {
    pub show_prefix: bool,
    pub show_name: bool,
    pub show_id: bool,
    pub show_status: bool,
    pub show_color: bool,
    /// Show unique prefix coloring for `change_id` (JJ only)
    pub show_prefix_color: bool,
}

impl DisplayConfig {
    pub const fn all_visible() -> Self {
        Self {
            show_prefix: true,
            show_name: true,
            show_id: true,
            show_status: true,
            show_color: true,
            show_prefix_color: true,
        }
    }
}

/// Configuration options
#[derive(Debug, Clone)]
pub struct Config {
    /// Max length for branch/bookmark name (0 = unlimited)
    pub truncate_name: usize,
    /// Length of `change_id/commit` hash to display
    pub id_length: usize,
    /// Max depth to search for ancestor bookmarks (0 = disabled, default: 10)
    pub ancestor_bookmark_depth: usize,
    /// Max bookmarks to display (0 = unlimited)
    pub bookmarks_display_limit: usize,
    /// Symbol prefix for JJ repos
    pub jj_symbol: Cow<'static, str>,
    /// Symbol prefix for Git repos
    #[cfg_attr(not(feature = "git"), allow(dead_code))]
    pub git_symbol: Cow<'static, str>,
    /// JJ display options
    pub jj_display: DisplayConfig,
    /// Git display options
    #[cfg_attr(not(feature = "git"), allow(dead_code))]
    pub git_display: DisplayConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            truncate_name: 0, // unlimited
            id_length: 8,
            ancestor_bookmark_depth: 10,
            bookmarks_display_limit: 3,
            jj_symbol: Cow::Borrowed(DEFAULT_JJ_SYMBOL),
            git_symbol: Cow::Borrowed(DEFAULT_GIT_SYMBOL),
            jj_display: DisplayConfig::all_visible(),
            git_display: DisplayConfig::all_visible(),
        }
    }
}

/// CLI display flags for a repo type
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct DisplayFlags {
    pub no_prefix: bool,
    pub no_name: bool,
    pub no_id: bool,
    pub no_status: bool,
    pub no_color: bool,
    pub no_prefix_color: bool,
}

impl DisplayFlags {
    fn into_config(self, env_prefix: &str) -> DisplayConfig {
        DisplayConfig {
            show_prefix: !self.no_prefix && env::var(format!("{env_prefix}_PREFIX")).is_err(),
            show_name: !self.no_name && env::var(format!("{env_prefix}_NAME")).is_err(),
            show_id: !self.no_id && env::var(format!("{env_prefix}_ID")).is_err(),
            show_status: !self.no_status && env::var(format!("{env_prefix}_STATUS")).is_err(),
            show_color: !self.no_color && env::var(format!("{env_prefix}_COLOR")).is_err(),
            show_prefix_color: !self.no_prefix_color
                && env::var("JJ_STARSHIP_NO_PREFIX_COLOR").is_err(),
        }
    }
}

impl Config {
    /// Create config from CLI args and environment variables
    /// CLI args take precedence over env vars
    #[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
    pub fn new(
        truncate_name: Option<usize>,
        id_length: Option<usize>,
        ancestor_bookmark_depth: Option<usize>,
        bookmarks_display_limit: Option<usize>,
        jj_symbol: Option<String>,
        git_symbol: Option<String>,
        no_symbol: bool,
        jj_flags: DisplayFlags,
        git_flags: DisplayFlags,
    ) -> Self {
        let truncate_name = truncate_name
            .or_else(|| env::var("JJ_STARSHIP_TRUNCATE_NAME").ok()?.parse().ok())
            .unwrap_or(0);

        let id_length = id_length
            .or_else(|| env::var("JJ_STARSHIP_ID_LENGTH").ok()?.parse().ok())
            .unwrap_or(8);

        let ancestor_bookmark_depth = ancestor_bookmark_depth
            .or_else(|| {
                env::var("JJ_STARSHIP_ANCESTOR_BOOKMARK_DEPTH")
                    .ok()?
                    .parse()
                    .ok()
            })
            .unwrap_or(10);

        let bookmarks_display_limit = bookmarks_display_limit
            .or_else(|| {
                env::var("JJ_STARSHIP_BOOKMARKS_DISPLAY_LIMIT")
                    .ok()?
                    .parse()
                    .ok()
            })
            .unwrap_or(3);

        let (jj_symbol, git_symbol) = if no_symbol {
            (Cow::Borrowed(""), Cow::Borrowed(""))
        } else {
            let jj = jj_symbol
                .or_else(|| env::var("JJ_STARSHIP_JJ_SYMBOL").ok())
                .map_or(Cow::Borrowed(DEFAULT_JJ_SYMBOL), Cow::Owned);
            let git = git_symbol
                .or_else(|| env::var("JJ_STARSHIP_GIT_SYMBOL").ok())
                .map_or(Cow::Borrowed(DEFAULT_GIT_SYMBOL), Cow::Owned);
            (jj, git)
        };

        Self {
            truncate_name,
            id_length,
            ancestor_bookmark_depth,
            bookmarks_display_limit,
            jj_symbol,
            git_symbol,
            jj_display: jj_flags.into_config("JJ_STARSHIP_NO_JJ"),
            git_display: git_flags.into_config("JJ_STARSHIP_NO_GIT"),
        }
    }

    /// Truncate a string to max length, adding ellipsis if needed
    pub fn truncate<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if self.truncate_name == 0 || s.chars().count() <= self.truncate_name {
            Cow::Borrowed(s)
        } else if self.truncate_name <= 1 {
            Cow::Borrowed("…")
        } else {
            let truncated: String = s.chars().take(self.truncate_name - 1).collect();
            Cow::Owned(truncated + "…")
        }
    }
}
