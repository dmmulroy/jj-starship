//! jj-starship - Unified Git/JJ Starship prompt module

mod color;
mod config;
mod detect;
mod error;
#[cfg(feature = "git")]
mod git;
mod jj;
mod output;

#[cfg(feature = "git")]
use clap::Args;
use clap::{Parser, Subcommand};
use config::{Config, DisplayFlags};
use detect::RepoType;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// CLI args - bool fields are inherent to clap's flag-based interface
#[derive(Parser)]
#[command(name = "jj-starship")]
#[command(version)]
#[command(about = "Unified Git/JJ Starship prompt module")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Override working directory
    #[arg(long, global = true)]
    cwd: Option<PathBuf>,

    /// Max length for branch/bookmark name (0 = unlimited)
    #[arg(long, global = true)]
    truncate_name: Option<usize>,

    /// Length of `change_id/commit` hash to display (default: 8)
    #[arg(long, global = true)]
    id_length: Option<usize>,

    /// Max depth to search for ancestor bookmarks (0 = disabled, default: 10)
    #[arg(long, global = true)]
    ancestor_bookmark_depth: Option<usize>,

    /// Max bookmarks to display (0 = unlimited, default: 3)
    #[arg(long, global = true)]
    bookmarks_display_limit: Option<usize>,

    /// Prefixes to strip from bookmark names (comma-separated)
    #[arg(long, global = true)]
    strip_bookmark_prefix: Option<String>,

    /// Symbol prefix for JJ repos (default: "󱗆")
    #[arg(long, global = true)]
    jj_symbol: Option<String>,

    /// Disable symbol prefix entirely
    #[arg(long, global = true)]
    no_symbol: bool,

    /// Disable output styling
    #[arg(long, global = true)]
    no_color: bool,

    // JJ display flags
    /// Hide "on {symbol}" prefix for JJ repos
    #[arg(long, global = true)]
    no_jj_prefix: bool,
    /// Hide bookmark name for JJ repos
    #[arg(long, global = true)]
    no_jj_name: bool,
    /// Hide `change_id` for JJ repos
    #[arg(long, global = true)]
    no_jj_id: bool,
    /// Hide [status] for JJ repos
    #[arg(long, global = true)]
    no_jj_status: bool,
    /// Disable unique prefix coloring for `change_id`
    #[arg(long, global = true)]
    no_prefix_color: bool,

    #[cfg(feature = "git")]
    #[command(flatten)]
    git: GitArgs,
}

/// Git-specific CLI flags - bools map directly to clap's --no-* pattern
#[cfg(feature = "git")]
#[derive(Args)]
#[allow(clippy::struct_excessive_bools)]
struct GitArgs {
    /// Symbol prefix for Git repos (default: "")
    #[arg(long, global = true)]
    git_symbol: Option<String>,
    /// Hide "on {symbol}" prefix for Git repos
    #[arg(long, global = true)]
    no_git_prefix: bool,
    /// Hide branch name for Git repos
    #[arg(long, global = true)]
    no_git_name: bool,
    /// Hide (commit) for Git repos
    #[arg(long, global = true)]
    no_git_id: bool,
    /// Hide [status] for Git repos
    #[arg(long, global = true)]
    no_git_status: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Output prompt string (default)
    Prompt,
    /// Exit 0 if in repo, 1 otherwise (for starship "when" condition)
    Detect,
    /// Print version and build info
    Version,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(cwd) = cli.cwd.or_else(|| env::current_dir().ok()) else {
        return ExitCode::FAILURE;
    };
    let jj_symbol = cli.jj_symbol;
    let jj_flags = DisplayFlags {
        no_prefix: cli.no_jj_prefix,
        no_name: cli.no_jj_name,
        no_id: cli.no_jj_id,
        no_status: cli.no_jj_status,
        no_color: cli.no_color,
        no_prefix_color: cli.no_prefix_color,
    };

    #[cfg(feature = "git")]
    let (git_symbol, git_flags) = (
        cli.git.git_symbol,
        DisplayFlags {
            no_prefix: cli.git.no_git_prefix,
            no_name: cli.git.no_git_name,
            no_id: cli.git.no_git_id,
            no_status: cli.git.no_git_status,
            no_color: cli.no_color,
            no_prefix_color: false, // N/A for git
        },
    );
    #[cfg(not(feature = "git"))]
    let (git_symbol, git_flags): (Option<String>, DisplayFlags) = (None, DisplayFlags::default());

    let config = Config::new(
        cli.truncate_name,
        cli.id_length,
        cli.ancestor_bookmark_depth,
        cli.bookmarks_display_limit,
        cli.strip_bookmark_prefix,
        jj_symbol,
        git_symbol,
        cli.no_symbol,
        jj_flags,
        git_flags,
    );

    match cli.command.unwrap_or(Command::Prompt) {
        Command::Prompt => {
            if let Some(output) = run_prompt(&cwd, &config) {
                print!("{output}");
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Command::Detect => {
            if detect::in_repo(&cwd) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Command::Version => {
            print_version();
            ExitCode::SUCCESS
        }
    }
}

/// Run prompt generation, returning None on error (silent fail for prompts)
#[allow(unreachable_patterns)]
fn run_prompt(cwd: &Path, config: &Config) -> Option<String> {
    let result = detect::detect(cwd);

    match result.repo_type {
        RepoType::Jj | RepoType::JjColocated => {
            let repo_root = result.repo_root?;
            let info =
                jj::collect(&repo_root, config.id_length, config.ancestor_bookmark_depth).ok()?;
            Some(output::format_jj(&info, config))
        }
        #[cfg(feature = "git")]
        RepoType::Git => {
            let repo_root = result.repo_root?;
            let info = git::collect(&repo_root, config.id_length).ok()?;
            Some(output::format_git(&info, config))
        }
        RepoType::None => None,
        // Catch disabled variants
        _ => None,
    }
}

fn print_version() {
    let version = env!("CARGO_PKG_VERSION");
    let change_id = env!("JJ_CHANGE_ID");
    let commit = env!("GIT_COMMIT");
    let date = env!("BUILD_DATE");

    println!("jj-starship {version}");
    println!("change: {change_id}");
    println!("commit: {commit}");
    println!("built:  {date}");

    #[allow(unused_mut)] // mut needed when features are enabled
    let mut features: Vec<&str> = Vec::new();
    #[cfg(feature = "git")]
    features.push("git");

    if features.is_empty() {
        println!("features: none");
    } else {
        println!("features: {}", features.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn default_command_is_prompt() {
        let cli = Cli::try_parse_from(["jj-starship"]).unwrap();
        assert!(cli.command.is_none()); // None defaults to Prompt in main()
    }

    #[test]
    fn explicit_prompt_subcommand() {
        let cli = Cli::try_parse_from(["jj-starship", "prompt"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Prompt)));
    }

    #[test]
    fn detect_subcommand() {
        let cli = Cli::try_parse_from(["jj-starship", "detect"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Detect)));
    }

    #[test]
    fn version_subcommand() {
        let cli = Cli::try_parse_from(["jj-starship", "version"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Version)));
    }

    #[test]
    fn cwd_arg() {
        let cli = Cli::try_parse_from(["jj-starship", "--cwd", "/some/path"]).unwrap();
        assert_eq!(cli.cwd, Some(PathBuf::from("/some/path")));
    }

    #[test]
    fn truncate_name_arg() {
        let cli = Cli::try_parse_from(["jj-starship", "--truncate-name", "20"]).unwrap();
        assert_eq!(cli.truncate_name, Some(20));
    }

    #[test]
    fn id_length_arg() {
        let cli = Cli::try_parse_from(["jj-starship", "--id-length", "12"]).unwrap();
        assert_eq!(cli.id_length, Some(12));
    }

    #[test]
    fn ancestor_bookmark_depth_arg() {
        let cli = Cli::try_parse_from(["jj-starship", "--ancestor-bookmark-depth", "5"]).unwrap();
        assert_eq!(cli.ancestor_bookmark_depth, Some(5));
    }

    #[test]
    fn bookmarks_display_limit_arg() {
        let cli = Cli::try_parse_from(["jj-starship", "--bookmarks-display-limit", "2"]).unwrap();
        assert_eq!(cli.bookmarks_display_limit, Some(2));
    }

    #[test]
    fn strip_bookmark_prefix_arg() {
        let cli = Cli::try_parse_from(["jj-starship", "--strip-bookmark-prefix", "feature/,fix/"])
            .unwrap();
        assert_eq!(cli.strip_bookmark_prefix, Some("feature/,fix/".to_string()));
    }

    #[test]
    fn jj_symbol_arg() {
        let cli = Cli::try_parse_from(["jj-starship", "--jj-symbol", "JJ:"]).unwrap();
        assert_eq!(cli.jj_symbol, Some("JJ:".to_string()));
    }

    #[test]
    fn no_symbol_takes_precedence_over_jj_symbol() {
        let cli =
            Cli::try_parse_from(["jj-starship", "--jj-symbol", "custom", "--no-symbol"]).unwrap();
        // no_symbol should be true, and when Config is built, symbols become empty
        assert!(cli.no_symbol);
        assert_eq!(cli.jj_symbol, Some("custom".to_string()));

        // Verify Config respects no_symbol precedence
        let config = Config::new(
            None,
            None,
            None,
            None,
            None,
            cli.jj_symbol,
            None,
            cli.no_symbol,
            DisplayFlags::default(),
            DisplayFlags::default(),
        );
        assert_eq!(config.jj_symbol.as_ref(), "");
        assert_eq!(config.git_symbol.as_ref(), "");
    }

    #[test]
    fn no_jj_prefix_flag() {
        let cli = Cli::try_parse_from(["jj-starship", "--no-jj-prefix"]).unwrap();
        assert!(cli.no_jj_prefix);
    }

    #[test]
    fn no_jj_name_flag() {
        let cli = Cli::try_parse_from(["jj-starship", "--no-jj-name"]).unwrap();
        assert!(cli.no_jj_name);
    }

    #[test]
    fn no_jj_id_flag() {
        let cli = Cli::try_parse_from(["jj-starship", "--no-jj-id"]).unwrap();
        assert!(cli.no_jj_id);
    }

    #[test]
    fn no_jj_status_flag() {
        let cli = Cli::try_parse_from(["jj-starship", "--no-jj-status"]).unwrap();
        assert!(cli.no_jj_status);
    }

    #[test]
    fn no_color_flag() {
        let cli = Cli::try_parse_from(["jj-starship", "--no-color"]).unwrap();
        assert!(cli.no_color);
    }

    #[test]
    fn no_prefix_color_flag() {
        let cli = Cli::try_parse_from(["jj-starship", "--no-prefix-color"]).unwrap();
        assert!(cli.no_prefix_color);
    }

    #[test]
    fn multiple_global_args() {
        let cli = Cli::try_parse_from([
            "jj-starship",
            "--cwd",
            "/test",
            "--truncate-name",
            "15",
            "--id-length",
            "6",
            "--no-jj-prefix",
            "--no-jj-status",
        ])
        .unwrap();
        assert_eq!(cli.cwd, Some(PathBuf::from("/test")));
        assert_eq!(cli.truncate_name, Some(15));
        assert_eq!(cli.id_length, Some(6));
        assert!(cli.no_jj_prefix);
        assert!(cli.no_jj_status);
        assert!(!cli.no_jj_name);
        assert!(!cli.no_jj_id);
    }

    #[test]
    fn global_args_work_with_subcommand() {
        let cli = Cli::try_parse_from(["jj-starship", "--id-length", "4", "detect"]).unwrap();
        assert_eq!(cli.id_length, Some(4));
        assert!(matches!(cli.command, Some(Command::Detect)));
    }

    #[cfg(feature = "git")]
    mod git_args {
        use super::*;

        #[test]
        fn git_symbol_arg() {
            let cli = Cli::try_parse_from(["jj-starship", "--git-symbol", "GIT:"]).unwrap();
            assert_eq!(cli.git.git_symbol, Some("GIT:".to_string()));
        }

        #[test]
        fn no_git_prefix_flag() {
            let cli = Cli::try_parse_from(["jj-starship", "--no-git-prefix"]).unwrap();
            assert!(cli.git.no_git_prefix);
        }

        #[test]
        fn no_git_name_flag() {
            let cli = Cli::try_parse_from(["jj-starship", "--no-git-name"]).unwrap();
            assert!(cli.git.no_git_name);
        }

        #[test]
        fn no_git_id_flag() {
            let cli = Cli::try_parse_from(["jj-starship", "--no-git-id"]).unwrap();
            assert!(cli.git.no_git_id);
        }

        #[test]
        fn no_git_status_flag() {
            let cli = Cli::try_parse_from(["jj-starship", "--no-git-status"]).unwrap();
            assert!(cli.git.no_git_status);
        }
    }
}
