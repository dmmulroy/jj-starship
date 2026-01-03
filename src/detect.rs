//! Repo type detection - walks up from cwd to find .jj or .git

use std::path::{Path, PathBuf};

/// Type of repository detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RepoType {
    /// Pure JJ repo (.jj/ only)
    Jj,
    /// Colocated JJ+Git repo (.jj/ and .git/)
    JjColocated,
    /// Pure Git repo (.git/ only)
    Git,
    /// Not in any repo
    None,
}

/// Result of repo detection
#[derive(Debug)]
pub struct DetectResult {
    pub repo_type: RepoType,
    pub repo_root: Option<PathBuf>,
}

/// Detect repo type by walking up from the given path
#[must_use = "returns detection result, does not modify state"]
pub fn detect(start: &Path) -> DetectResult {
    let mut current = start.to_path_buf();

    loop {
        let has_jj = current.join(".jj").is_dir();
        let has_git = current.join(".git").exists(); // can be file (worktree) or dir

        let repo_type = match (has_jj, has_git) {
            (true, true) => RepoType::JjColocated,
            (true, false) => RepoType::Jj,
            (false, true) => RepoType::Git,
            (false, false) => RepoType::None,
        };

        if repo_type != RepoType::None {
            return DetectResult {
                repo_type,
                repo_root: Some(current),
            };
        }

        // Walk up
        if !current.pop() {
            break;
        }
    }

    DetectResult {
        repo_type: RepoType::None,
        repo_root: None,
    }
}

/// Returns true if in any repo (for `jj-starship detect` command)
#[must_use = "returns detection result, does not modify state"]
pub fn in_repo(start: &Path) -> bool {
    detect(start).repo_type != RepoType::None
}
