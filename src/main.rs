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

    let mut features = Vec::new();
    #[cfg(feature = "git")]
    features.push("git");

    if features.is_empty() {
        println!("features: none");
    } else {
        println!("features: {}", features.join(", "));
    }
}
