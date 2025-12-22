//! JJ repository info collection

use crate::error::{Error, Result};
use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::ref_name::RefName;
use jj_lib::repo::{Repo, StoreFactories};
use jj_lib::settings::UserSettings;
use jj_lib::str_util::{StringMatcher, StringPattern};
use jj_lib::workspace::{Workspace, default_working_copy_factories};
use std::path::Path;
use std::sync::Arc;

/// JJ repository status info
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct JjInfo {
    /// Short change ID (8 chars)
    pub change_id: String,
    /// Bookmarks with distances: vec of (name, distance). Empty if none found.
    /// Distance 0 = directly on WC, 1+ = ancestor distance
    pub bookmarks: Vec<(String, usize)>,
    /// Description is empty (needs commit message)
    pub empty_desc: bool,
    /// Has conflicts in tree
    pub conflict: bool,
    /// Multiple commits for same `change_id`
    pub divergent: bool,
    /// Whether any bookmark has a remote
    pub has_remote: bool,
    /// Whether any bookmark is synced with remote
    pub is_synced: bool,
}

/// Create minimal `UserSettings` for read-only operations
fn create_user_settings() -> Result<UserSettings> {
    let mut config = StackedConfig::with_defaults();

    // Minimal config required by UserSettings
    let mut user_layer = ConfigLayer::empty(ConfigSource::User);
    user_layer
        .set_value("user.name", "jj-starship")
        .map_err(|e| Error::Jj(format!("set user.name: {e}")))?;
    user_layer
        .set_value("user.email", "jj-starship@localhost")
        .map_err(|e| Error::Jj(format!("set user.email: {e}")))?;
    config.add_layer(user_layer);

    UserSettings::from_config(config).map_err(|e| Error::Jj(format!("settings: {e}")))
}

/// Find immutable head commits (trunk + tags + untracked remote bookmarks)
/// Mirrors jj's `builtin_immutable_heads()` without revset evaluation
fn find_immutable_heads(view: &jj_lib::view::View) -> std::collections::HashSet<jj_lib::backend::CommitId> {
    use std::collections::HashSet;

    let mut immutable = HashSet::new();

    // Single pass over all remote bookmarks
    for (symbol, remote_ref) in view.remote_bookmarks_matching(&StringMatcher::All, &StringMatcher::All) {
        let name = symbol.name.as_str();
        let remote = symbol.remote.as_str();

        if remote == "git" {
            continue;
        }

        // trunk: main/master/trunk on origin/upstream
        let is_trunk =
            matches!(remote, "origin" | "upstream") && matches!(name, "main" | "master" | "trunk");

        // untracked: no local counterpart
        let is_untracked = view.get_local_bookmark(symbol.name).is_absent();

        if is_trunk || is_untracked {
            if let Some(id) = remote_ref.target.as_normal() {
                immutable.insert(id.clone());
            }
        }
    }

    // Tags (usually few)
    for (_, target) in view.tags() {
        if let Some(id) = target.local_target.as_normal() {
            immutable.insert(id.clone());
        }
    }

    immutable
}

/// Search for all bookmarks on ancestor commits using BFS
/// Returns bookmarks sorted by distance (closest first)
fn find_ancestor_bookmarks(
    repo: &Arc<jj_lib::repo::ReadonlyRepo>,
    view: &jj_lib::view::View,
    wc_id: &jj_lib::backend::CommitId,
    max_depth: usize,
) -> Result<Vec<(String, usize)>> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let mut queue: VecDeque<(jj_lib::backend::CommitId, usize)> = VecDeque::new();
    let mut visited = HashSet::new();
    let mut bookmarks_with_distances: HashMap<String, usize> = HashMap::new();

    // Pre-compute immutable heads to stop traversal at trunk/tags/untracked remotes
    let immutable_heads = find_immutable_heads(view);

    // Start BFS from WC commit parents
    let wc_commit = repo
        .store()
        .get_commit(wc_id)
        .map_err(|e| Error::Jj(format!("get commit: {e}")))?;

    for parent_id in wc_commit.parent_ids() {
        queue.push_back((parent_id.clone(), 1));
    }

    while let Some((commit_id, depth)) = queue.pop_front() {
        // Stop if we exceed max depth
        if depth > max_depth {
            continue;
        }

        // Skip if already visited
        if !visited.insert(commit_id.clone()) {
            continue;
        }

        // Collect all bookmarks at this commit
        for (bookmark_name, _) in view.local_bookmarks_for_commit(&commit_id) {
            let name = bookmark_name.as_str().to_string();
            // Only record first (shortest) distance for each bookmark
            bookmarks_with_distances.entry(name).or_insert(depth);
        }

        // Stop at immutable heads - don't traverse past trunk/tags/untracked remotes
        if immutable_heads.contains(&commit_id) {
            continue;
        }

        // Add parents to queue for next level
        if depth < max_depth {
            let commit = repo
                .store()
                .get_commit(&commit_id)
                .map_err(|e| Error::Jj(format!("get commit: {e}")))?;

            for parent_id in commit.parent_ids() {
                queue.push_back((parent_id.clone(), depth + 1));
            }
        }
    }

    // Convert to vec and sort by distance
    let mut result: Vec<(String, usize)> = bookmarks_with_distances.into_iter().collect();
    result.sort_by_key(|(_, distance)| *distance);
    Ok(result)
}

/// Collect JJ repo info from the given path
pub fn collect(repo_root: &Path, id_length: usize, ancestor_depth: usize) -> Result<JjInfo> {
    let settings = create_user_settings()?;

    let workspace = Workspace::load(
        &settings,
        repo_root,
        &StoreFactories::default(),
        &default_working_copy_factories(),
    )
    .map_err(|e| Error::Jj(format!("load workspace: {e}")))?;

    let repo: Arc<jj_lib::repo::ReadonlyRepo> = workspace
        .repo_loader()
        .load_at_head()
        .map_err(|e| Error::Jj(format!("load repo: {e}")))?;

    let view = repo.view();

    // Get WC commit ID
    let wc_id = view
        .wc_commit_ids()
        .get(workspace.workspace_name())
        .ok_or_else(|| Error::Jj("no working copy".into()))?;

    // Load commit
    let commit = repo
        .store()
        .get_commit(wc_id)
        .map_err(|e| Error::Jj(format!("get commit: {e}")))?;

    // Change ID in JJ's reverse hex format
    let change_id_full = encode_reverse_hex(commit.change_id().as_bytes());
    let change_id = change_id_full[..id_length.min(change_id_full.len())].to_string();

    // Empty description check
    let empty_desc = commit.description().trim().is_empty();

    // Conflict check
    let conflict = commit.has_conflict();

    // Divergent check - multiple commits for same change_id
    let divergent = repo
        .resolve_change_id(commit.change_id())
        .ok()
        .flatten()
        .is_some_and(|commits| commits.len() > 1);

    // Find bookmarks - first check direct bookmarks on WC (distance 0)
    let mut bookmarks: Vec<(String, usize)> = view
        .local_bookmarks_for_commit(wc_id)
        .map(|(name, _)| (name.as_str().to_string(), 0))
        .collect();

    // Always search ancestors if enabled (useful for stacked PR context)
    // Ancestor bookmarks are disjoint from direct bookmarks (different commits)
    if ancestor_depth > 0 {
        let ancestors = find_ancestor_bookmarks(&repo, view, wc_id, ancestor_depth)?;
        bookmarks.extend(ancestors);
    }

    // Check remote sync status for first (closest) bookmark only
    // For stacked PRs, this reflects whether current stack position needs pushing
    let (has_remote, is_synced) = if bookmarks.is_empty() {
        (false, true)
    } else {
        let (bm_name, _) = &bookmarks[0];
        let local_target = view.get_local_bookmark(RefName::new(bm_name));

        let name_matcher = StringPattern::exact(bm_name).to_matcher();
        let mut has_remote = false;
        let mut is_synced = false;

        for (symbol, remote_ref) in view.remote_bookmarks_matching(&name_matcher, &StringMatcher::All)
        {
            if symbol.remote.as_str() == "git" {
                continue;
            }
            has_remote = true;
            if remote_ref.target == *local_target {
                is_synced = true;
                break;
            }
        }

        (has_remote, is_synced || !has_remote)
    };

    Ok(JjInfo {
        change_id,
        bookmarks,
        empty_desc,
        conflict,
        divergent,
        has_remote,
        is_synced,
    })
}
