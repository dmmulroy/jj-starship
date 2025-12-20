//! Configuration for jj-starship

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
}

impl DisplayConfig {
    pub const fn all_visible() -> Self {
        Self {
            show_prefix: true,
            show_name: true,
            show_id: true,
            show_status: true,
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
    /// Symbol prefix for JJ repos
    pub jj_symbol: String,
    /// Symbol prefix for Git repos
    pub git_symbol: String,
    /// JJ display options
    pub jj_display: DisplayConfig,
    /// Git display options
    pub git_display: DisplayConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            truncate_name: 0, // unlimited
            id_length: 8,
            jj_symbol: DEFAULT_JJ_SYMBOL.into(),
            git_symbol: DEFAULT_GIT_SYMBOL.into(),
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
}

impl DisplayFlags {
    fn into_config(self, env_prefix: &str) -> DisplayConfig {
        DisplayConfig {
            show_prefix: !self.no_prefix && env::var(format!("{env_prefix}_PREFIX")).is_err(),
            show_name: !self.no_name && env::var(format!("{env_prefix}_NAME")).is_err(),
            show_id: !self.no_id && env::var(format!("{env_prefix}_ID")).is_err(),
            show_status: !self.no_status && env::var(format!("{env_prefix}_STATUS")).is_err(),
        }
    }
}

impl Config {
    /// Create config from CLI args and environment variables
    /// CLI args take precedence over env vars
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        truncate_name: Option<usize>,
        id_length: Option<usize>,
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

        let (jj_symbol, git_symbol) = if no_symbol {
            (String::new(), String::new())
        } else {
            let jj = jj_symbol
                .or_else(|| env::var("JJ_STARSHIP_JJ_SYMBOL").ok())
                .unwrap_or_else(|| DEFAULT_JJ_SYMBOL.into());
            let git = git_symbol
                .or_else(|| env::var("JJ_STARSHIP_GIT_SYMBOL").ok())
                .unwrap_or_else(|| DEFAULT_GIT_SYMBOL.into());
            (jj, git)
        };

        Self {
            truncate_name,
            id_length,
            jj_symbol,
            git_symbol,
            jj_display: jj_flags.into_config("JJ_STARSHIP_NO_JJ"),
            git_display: git_flags.into_config("JJ_STARSHIP_NO_GIT"),
        }
    }

    /// Truncate a string to max length, adding ellipsis if needed
    pub fn truncate(&self, s: &str) -> String {
        if self.truncate_name == 0 || s.len() <= self.truncate_name {
            s.to_string()
        } else if self.truncate_name <= 1 {
            "…".to_string()
        } else {
            format!("{}…", &s[..self.truncate_name - 1])
        }
    }
}
