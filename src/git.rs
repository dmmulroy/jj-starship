//! Git repository info collection using git2

use crate::error::{Error, Result};
use git2::{Repository, Status, StatusOptions};
use std::path::Path;

/// Git repository status info
#[derive(Debug)]
pub struct GitInfo {
    /// Branch name (None if detached)
    pub branch: Option<String>,
    /// Short commit hash (7 chars)
    pub head_short: String,
    /// Count of staged files
    pub staged: usize,
    /// Count of modified (unstaged) files
    pub modified: usize,
    /// Count of untracked files
    pub untracked: usize,
    /// Count of deleted files
    pub deleted: usize,
    /// Count of conflicted files
    pub conflicted: usize,
    /// Commits ahead of upstream
    pub ahead: usize,
    /// Commits behind upstream
    pub behind: usize,
}

/// Collect Git repo info from the given path
#[must_use = "returns collected repo info, does not modify state"]
pub fn collect(repo_root: &Path, id_length: usize) -> Result<GitInfo> {
    let repo = Repository::open(repo_root).map_err(|e| Error::Git(format!("open: {e}")))?;

    // Status counts - compute once for both empty and normal repos
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false)
        .include_ignored(false)
        .exclude_submodules(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|e| Error::Git(format!("statuses: {e}")))?;

    let mut staged = 0usize;
    let mut modified = 0usize;
    let mut untracked = 0usize;
    let mut deleted = 0usize;
    let mut conflicted = 0usize;

    for entry in statuses.iter() {
        let status = entry.status();

        // Conflicted
        if status.contains(Status::CONFLICTED) {
            conflicted += 1;
            continue;
        }

        // Staged (index changes)
        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }

        // Working tree changes
        if status.intersects(Status::WT_MODIFIED | Status::WT_TYPECHANGE) {
            modified += 1;
        }
        if status.contains(Status::WT_DELETED) {
            deleted += 1;
        }
        if status.contains(Status::WT_NEW) {
            untracked += 1;
        }
    }

    // Get HEAD - may fail if no commits yet
    let Ok(head) = repo.head() else {
        // No commits yet - try to get branch from HEAD reference
        let branch = repo
            .find_reference("HEAD")
            .ok()
            .and_then(|r| r.symbolic_target().map(std::string::ToString::to_string))
            .and_then(|s| s.strip_prefix("refs/heads/").map(String::from));

        return Ok(GitInfo {
            branch,
            head_short: "empty".into(),
            staged,
            modified,
            untracked,
            deleted,
            conflicted,
            ahead: 0,
            behind: 0,
        });
    };

    let detached = repo
        .head_detached()
        .map_err(|e| Error::Git(format!("head_detached: {e}")))?;

    // Branch name
    let branch = if detached {
        None
    } else {
        head.shorthand().map(String::from)
    };

    // Short commit hash
    let head_commit = head
        .peel_to_commit()
        .map_err(|e| Error::Git(format!("peel_to_commit: {e}")))?;
    let full_hash = head_commit.id().to_string();
    let head_short = full_hash[..id_length.min(full_hash.len())].to_string();

    // Ahead/behind upstream
    let (ahead, behind) = get_ahead_behind(&repo, &head).unwrap_or((0, 0));

    Ok(GitInfo {
        branch,
        head_short,
        staged,
        modified,
        untracked,
        deleted,
        conflicted,
        ahead,
        behind,
    })
}

/// Get ahead/behind counts relative to upstream
fn get_ahead_behind(
    repo: &Repository,
    head: &git2::Reference<'_>,
) -> std::result::Result<(usize, usize), git2::Error> {
    // Need a branch, not detached HEAD
    if repo.head_detached()? {
        return Ok((0, 0));
    }

    // Get the branch
    let branch = repo.find_branch(
        head.shorthand()
            .ok_or_else(|| git2::Error::from_str("no branch name"))?,
        git2::BranchType::Local,
    )?;

    // Get upstream
    let upstream = branch.upstream()?;

    let local_oid = head.peel_to_commit()?.id();
    let upstream_oid = upstream.get().peel_to_commit()?.id();

    repo.graph_ahead_behind(local_oid, upstream_oid)
}
