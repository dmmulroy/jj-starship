//! jj-starship - Unified Git/JJ Starship prompt module

mod color;
mod config;
mod detect;
mod error;
mod git;
mod jj;
mod output;

use clap::{Parser, Subcommand};
use config::{Config, DisplayFlags};
use detect::RepoType;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "jj-starship")]
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

    /// Symbol prefix for JJ repos (default: "󰶛 ")
    #[arg(long, global = true)]
    jj_symbol: Option<String>,

    /// Symbol prefix for Git repos (default: " ")
    #[arg(long, global = true)]
    git_symbol: Option<String>,

    /// Disable symbol prefix entirely
    #[arg(long, global = true)]
    no_symbol: bool,

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

    // Git display flags
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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let cwd = cli
        .cwd
        .unwrap_or_else(|| env::current_dir().unwrap_or_default());
    let config = Config::new(
        cli.truncate_name,
        cli.id_length,
        cli.jj_symbol,
        cli.git_symbol,
        cli.no_symbol,
        DisplayFlags {
            no_prefix: cli.no_jj_prefix,
            no_name: cli.no_jj_name,
            no_id: cli.no_jj_id,
            no_status: cli.no_jj_status,
        },
        DisplayFlags {
            no_prefix: cli.no_git_prefix,
            no_name: cli.no_git_name,
            no_id: cli.no_git_id,
            no_status: cli.no_git_status,
        },
    );

    match cli.command.unwrap_or(Command::Prompt) {
        Command::Prompt => {
            if let Some(output) = run_prompt(&cwd, &config) {
                print!("{output}");
            }
            ExitCode::SUCCESS
        }
        Command::Detect => {
            if detect::in_repo(&cwd) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

/// Run prompt generation, returning None on error (silent fail for prompts)
fn run_prompt(cwd: &Path, config: &Config) -> Option<String> {
    let result = detect::detect(cwd);

    match result.repo_type {
        RepoType::JjColocated | RepoType::Jj => {
            let repo_root = result.repo_root?;
            let info = jj::collect(&repo_root, config.id_length).ok()?;
            Some(output::format_jj(&info, config))
        }
        RepoType::Git => {
            let repo_root = result.repo_root?;
            let info = git::collect(&repo_root, config.id_length).ok()?;
            Some(output::format_git(&info, config))
        }
        RepoType::None => None,
    }
}
