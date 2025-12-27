//! Output formatting for prompt strings

#[cfg(feature = "git")]
use std::borrow::Cow;
#[cfg(feature = "git")]
use std::fmt::Write;

use crate::color::{BLUE, BRIGHT_BLACK, BRIGHT_MAGENTA, GREEN, PURPLE, RED, RESET};
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

/// Format `change_id` with unique prefix highlighting (matching jj log style)
/// Prefix is bright magenta, rest is gray
fn format_change_id(change_id: &str, prefix_len: usize, show_prefix_color: bool) -> String {
    if !show_prefix_color {
        return change_id.to_string();
    }
    let prefix_len = prefix_len.min(change_id.len());
    let prefix = &change_id[..prefix_len];
    let rest = &change_id[prefix_len..];
    if rest.is_empty() {
        format!("{BRIGHT_MAGENTA}{prefix}{RESET}")
    } else {
        format!("{BRIGHT_MAGENTA}{prefix}{RESET}{BRIGHT_BLACK}{rest}{RESET}")
    }
}

/// Format JJ info as prompt string
/// Pattern: `on {symbol}{change_id} ({bookmarks}) [{status}]`
pub fn format_jj(info: &JjInfo, config: &Config) -> String {
    let mut out = String::with_capacity(128);
    let display = &config.jj_display;

    // "on {symbol}" prefix
    if display.show_prefix {
        out.push_str("on ");
        out.push_str(&format_segment(&config.jj_symbol, BLUE, display.show_color));
    }

    // change_id with prefix coloring (controlled by show_id)
    if display.show_id {
        let use_prefix_color = display.show_color && display.show_prefix_color;
        if use_prefix_color {
            out.push_str(&format_change_id(
                &info.change_id,
                info.change_id_prefix_len,
                true,
            ));
        } else {
            out.push_str(&format_segment(&info.change_id, PURPLE, display.show_color));
        }
    }

    // Bookmarks in parentheses (controlled by show_name - they're names/labels)
    if display.show_name && !info.bookmarks.is_empty() {
        if !out.is_empty() {
            out.push(' ');
        }

        let bookmark_strs: Vec<String> = info
            .bookmarks
            .iter()
            .map(|(name, dist)| {
                let truncated = config.truncate(name);
                if *dist > 0 {
                    format!("{truncated}~{dist}")
                } else {
                    truncated.into_owned()
                }
            })
            .collect();
        let bookmarks_text = format!("({})", bookmark_strs.join(", "));
        out.push_str(&format_segment(&bookmarks_text, GREEN, display.show_color));
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
            ancestor_bookmark_depth: 10,
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
            change_id_prefix_len: 4,
            bookmarks: vec![("main".into(), 0)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: true,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!(
                "on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET} {GREEN}(main){RESET}"
            )
        );
    }

    #[test]
    fn test_jj_format_dirty() {
        // When no bookmarks, only change_id is shown
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![],
            empty_desc: true,
            conflict: true,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!(
                "on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET} {RED}[!?]{RESET}"
            )
        );
    }

    #[test]
    fn test_jj_format_with_symbol() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("main".into(), 0)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: true,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &default_config()),
            format!(
                "on {BLUE}{DEFAULT_JJ_SYMBOL}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET} {GREEN}(main){RESET}"
            )
        );
    }

    #[test]
    fn test_jj_format_truncated() {
        let config = Config {
            truncate_name: 5,
            id_length: 8,
            ancestor_bookmark_depth: 10,
            jj_symbol: Cow::Borrowed(""),
            git_symbol: Cow::Borrowed(""),
            jj_display: DisplayConfig::all_visible(),
            git_display: DisplayConfig::all_visible(),
        };
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("very-long-bookmark-name".into(), 0)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &config),
            format!(
                "on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET} {GREEN}(very…){RESET}"
            )
        );
    }

    #[test]
    fn test_jj_format_ancestor_bookmark() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("main".into(), 3)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: true,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!(
                "on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET} {GREEN}(main~3){RESET}"
            )
        );
    }

    #[test]
    fn test_jj_format_no_bookmarks() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!("on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET}")
        );
    }

    #[test]
    fn test_jj_format_multiple_bookmarks() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("feature".into(), 1), ("main".into(), 2)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!(
                "on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET} {GREEN}(feature~1, main~2){RESET}"
            )
        );
    }

    #[test]
    fn test_jj_format_no_color() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("main".into(), 0)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: true,
            is_synced: true,
        };
        let config = Config {
            truncate_name: 0,
            id_length: 8,
            ancestor_bookmark_depth: 10,
            jj_symbol: Cow::Borrowed("󱗆 "),
            git_symbol: Cow::Borrowed(" "),
            jj_display: DisplayConfig {
                show_prefix: true,
                show_name: true,
                show_id: true,
                show_status: true,
                show_color: false,
                show_prefix_color: true,
            },
            git_display: DisplayConfig::all_visible(),
        };
        assert_eq!(format_jj(&info, &config), "on 󱗆 yzxv1234 (main)");
    }

    #[test]
    fn test_jj_format_no_id_hides_change_id() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("main".into(), 0)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        let config = Config {
            truncate_name: 0,
            id_length: 8,
            ancestor_bookmark_depth: 10,
            jj_symbol: Cow::Borrowed(""),
            git_symbol: Cow::Borrowed(""),
            jj_display: DisplayConfig {
                show_prefix: true,
                show_name: true,
                show_id: false, // --no-jj-id
                show_status: false,
                show_color: true,
                show_prefix_color: true,
            },
            git_display: DisplayConfig::all_visible(),
        };
        // --no-jj-id hides change_id, shows only bookmarks
        assert_eq!(
            format_jj(&info, &config),
            format!("on {BLUE}{RESET} {GREEN}(main){RESET}")
        );
    }

    #[test]
    fn test_jj_format_no_name_hides_bookmarks() {
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("main".into(), 0)],
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        let config = Config {
            truncate_name: 0,
            id_length: 8,
            ancestor_bookmark_depth: 10,
            jj_symbol: Cow::Borrowed(""),
            git_symbol: Cow::Borrowed(""),
            jj_display: DisplayConfig {
                show_prefix: true,
                show_name: false, // --no-jj-name
                show_id: true,
                show_status: false,
                show_color: true,
                show_prefix_color: true,
            },
            git_display: DisplayConfig::all_visible(),
        };
        // --no-jj-name hides bookmarks, shows only change_id with prefix coloring
        assert_eq!(
            format_jj(&info, &config),
            format!("on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET}")
        );
    }

    #[test]
    fn test_jj_format_direct_bookmark_distance_zero() {
        // Verifies that when WC has a direct bookmark (distance 0),
        // it shows without ~N suffix even with ancestor search enabled
        let info = JjInfo {
            change_id: "yzxv1234".into(),
            change_id_prefix_len: 4,
            bookmarks: vec![("main".into(), 0)], // distance 0 = directly on WC
            empty_desc: false,
            conflict: false,
            divergent: false,
            has_remote: false,
            is_synced: true,
        };
        assert_eq!(
            format_jj(&info, &no_symbol_config()),
            format!(
                "on {BLUE}{RESET}{BRIGHT_MAGENTA}yzxv{RESET}{BRIGHT_BLACK}1234{RESET} {GREEN}(main){RESET}"
            )
        );
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
