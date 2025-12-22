//! Output formatting for prompt strings

use std::borrow::Cow;
#[cfg(feature = "git")]
use std::fmt::Write;

use crate::color::{BLUE, GREEN, PURPLE, RED, RESET};
use crate::config::Config;
#[cfg(feature = "git")]
use crate::git::GitInfo;
use crate::jj::JjInfo;

fn format_segment(text: &str, color: &str, show_color: bool) -> String {
    if show_color {
        format!("{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Format JJ info as prompt string
/// Pattern: `on {symbol}{name} ({id}) [{status}]`
pub fn format_jj(info: &JjInfo, config: &Config) -> String {
    let mut out = String::with_capacity(128);
    let display = &config.jj_display;

    // "on {symbol}" prefix
    if display.show_prefix {
        out.push_str("on ");
        out.push_str(&format_segment(&config.jj_symbol, BLUE, display.show_color));
    }

    // Name in purple (bookmark or change_id prefix)
    let name: Cow<str> = info
        .bookmark
        .as_ref()
        .map_or(Cow::Borrowed(&info.change_id), |bm| config.truncate(bm));

    if display.show_name {
        out.push_str(&format_segment(&name, PURPLE, display.show_color));
    }

    // ID in green - skip if same as name (deduplicate)
    if display.show_id && *name != info.change_id {
        if !out.is_empty() {
            out.push(' ');
        }
        let id_text = format!("({})", &info.change_id);
        out.push_str(&format_segment(&id_text, GREEN, display.show_color));
    }

    // Status indicators in red (priority: ! > ⇔ > ? > ⇡)
    if display.show_status {
        let mut status = String::new();
        if info.conflict {
            status.push('!');
        }
        if info.divergent {
            status.push('⇔');
        }
        if info.empty_desc {
            status.push('?');
        }
        if info.has_remote && !info.is_synced {
            status.push('⇡');
        }

        if !status.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            let status_text = format!("[{}]", &status);
            out.push_str(&format_segment(&status_text, RED, display.show_color));
        }
    }

    out
}

/// Format Git info as prompt string
/// Pattern: `on {symbol}{name} ({id}) [{status}]`
#[cfg(feature = "git")]
pub fn format_git(info: &GitInfo, config: &Config) -> String {
    let mut out = String::with_capacity(128);
    let display = &config.git_display;

    // "on {symbol}" prefix
    if display.show_prefix {
        out.push_str("on ");
        out.push_str(&format_segment(
            &config.git_symbol,
            BLUE,
            display.show_color,
        ));
    }

    // Name in purple (branch or HEAD)
    if display.show_name {
        let name: Cow<str> = info
            .branch
            .as_ref()
            .map_or(Cow::Borrowed("HEAD"), |b| config.truncate(b));
        out.push_str(&format_segment(&name, PURPLE, display.show_color));
    }

    // ID in green
    if display.show_id {
        if !out.is_empty() {
            out.push(' ');
        }
        let id_text = format!("({})", &info.head_short);
        out.push_str(&format_segment(&id_text, GREEN, display.show_color));
    }

    // Status indicators in red
    if display.show_status {
        let mut status = String::new();

        // File status (order: = > + > ! > ? > ✘)
        if info.conflicted > 0 {
            status.push('=');
        }
        if info.staged > 0 {
            status.push('+');
        }
        if info.modified > 0 {
            status.push('!');
        }
        if info.untracked > 0 {
            status.push('?');
        }
        if info.deleted > 0 {
            status.push('✘');
        }

        // Ahead/behind
        if info.ahead > 0 {
            let _ = write!(status, "⇡{}", info.ahead);
        }
        if info.behind > 0 {
            let _ = write!(status, "⇣{}", info.behind);
        }

        if !status.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            let status_text = format!("[{}]", &status);
            out.push_str(&format_segment(&status_text, RED, display.show_color));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[cfg(feature = "git")]
    use crate::config::DEFAULT_GIT_SYMBOL;
    use crate::config::DEFAULT_JJ_SYMBOL;
    use crate::config::DisplayConfig;

    #[allow(dead_code)]
    fn default_config() -> Config {
        Config::default()
    }

    #[allow(dead_code)]
    fn no_symbol_config() -> Config {
        Config {
            truncate_name: 0,
            id_length: 8,
            jj_symbol: Cow::Borrowed(""),
            git_symbol: Cow::Borrowed(""),
            jj_display: DisplayConfig::all_visible(),
            git_display: DisplayConfig::all_visible(),
        }
    }

    #[test]
    fn test_jj_format_clean() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            bookmark: Some("main".into()),
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: true,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!("on {BLUE}{RESET}{PURPLE}main{RESET} {GREEN}(yzxv1234){RESET}")
        );
    }

    #[test]
    fn test_jj_format_dirty() {
        // When bookmark is None, name = change_id, so (change_id) is skipped (dedupe)
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            bookmark: None,
            empty_desc: true,
            conflict: true,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!("on {BLUE}{RESET}{PURPLE}yzxv1234{RESET} {RED}[!?]{RESET}")
        );
    }

    #[test]
    fn test_jj_format_with_symbol() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            bookmark: Some("main".into()),
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: true,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &default_config()),
            format!(
                "on {BLUE}{DEFAULT_JJ_SYMBOL}{RESET}{PURPLE}main{RESET} {GREEN}(yzxv1234){RESET}"
            )
        );
    }

    #[test]
    fn test_jj_format_truncated() {
        let config = Config {
            truncate_name: 5,
            id_length: 8,
            jj_symbol: Cow::Borrowed(""),
            git_symbol: Cow::Borrowed(""),
            jj_display: DisplayConfig::all_visible(),
            git_display: DisplayConfig::all_visible(),
        };
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            bookmark: Some("very-long-bookmark-name".into()),
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &config),
            format!("on {BLUE}{RESET}{PURPLE}very…{RESET} {GREEN}(yzxv1234){RESET}")
        );
    }

    #[test]
    fn test_jj_format_no_color() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            bookmark: Some("main".into()),
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: true,
            is_synced: true,
        };
        let config = Config {
            truncate_name: 0,
            id_length: 8,
            jj_symbol: Cow::Borrowed("󱗆 "),
            git_symbol: Cow::Borrowed(" "),
            jj_display: DisplayConfig {
                show_prefix: true,
                show_name: true,
                show_id: true,
                show_status: true,
                show_color: false,
            },
            git_display: DisplayConfig::all_visible(),
        };
        assert_eq!(format_jj(&info, &config), "on 󱗆 main (yzxv1234)");
    }

    #[cfg(feature = "git")]
    #[test]
    fn test_git_format_clean() {
        let info = GitInfo {
            branch: Some("main".into()),
            head_short: "a3b4c5d".into(),
            staged: 0,
            modified: 0,
            untracked: 0,
            deleted: 0,
            conflicted: 0,
            ahead: 0,
            behind: 0,
        };
        assert_eq!(
            format_git(&info, &no_symbol_config()),
            format!("on {BLUE}{RESET}{PURPLE}main{RESET} {GREEN}(a3b4c5d){RESET}")
        );
    }

    #[cfg(feature = "git")]
    #[test]
    fn test_git_format_dirty() {
        let info = GitInfo {
            branch: Some("feature".into()),
            head_short: "1234567".into(),
            staged: 2,
            modified: 3,
            untracked: 1,
            deleted: 0,
            conflicted: 0,
            ahead: 2,
            behind: 1,
        };
        assert_eq!(
            format_git(&info, &no_symbol_config()),
            format!(
                "on {BLUE}{RESET}{PURPLE}feature{RESET} {GREEN}(1234567){RESET} {RED}[+!?⇡2⇣1]{RESET}"
            )
        );
    }

    #[cfg(feature = "git")]
    #[test]
    fn test_git_format_with_symbol() {
        let info = GitInfo {
            branch: Some("main".into()),
            head_short: "a3b4c5d".into(),
            staged: 0,
            modified: 0,
            untracked: 0,
            deleted: 0,
            conflicted: 0,
            ahead: 0,
            behind: 0,
        };
        assert_eq!(
            format_git(&info, &default_config()),
            format!(
                "on {BLUE}{DEFAULT_GIT_SYMBOL}{RESET}{PURPLE}main{RESET} {GREEN}(a3b4c5d){RESET}"
            )
        );
    }
}
