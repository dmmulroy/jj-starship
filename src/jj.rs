//! JJ repository info collection

use crate::error::{Error, Result};
use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
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
    /// Bookmark name if WC is on one
    pub bookmark: Option<String>,
    /// Description is empty (needs commit message)
    pub empty_desc: bool,
    /// Has conflicts in tree
    pub conflict: bool,
    /// Multiple commits for same `change_id`
    pub divergent: bool,
    /// Bookmark exists on a remote
    pub has_remote: bool,
    /// Local bookmark == remote bookmark
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

/// Collect JJ repo info from the given path
pub fn collect(repo_root: &Path, id_length: usize) -> Result<JjInfo> {
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

    // Find bookmark at WC commit
    let bookmark: Option<String> = view
        .local_bookmarks_for_commit(wc_id)
        .next()
        .map(|(name, _)| name.as_str().to_string());

    // Check remote sync status (only if we have a bookmark)
    let (has_remote, is_synced) = if let Some(ref bm_name) = bookmark {
        let name_matcher = StringPattern::exact(bm_name).to_matcher();
        let remote_matcher = StringMatcher::All;

        // Single pass over remote bookmarks
        view.remote_bookmarks_matching(&name_matcher, &remote_matcher)
            .filter(|(symbol, _)| symbol.remote.as_str() != "git")
            .fold((false, false), |(_, synced), (_, remote_ref)| {
                let this_synced = remote_ref.target.as_normal().is_some_and(|id| id == wc_id);
                (true, synced || this_synced)
            })
    } else {
        (false, true)
    };

    Ok(JjInfo {
        change_id,
        bookmark,
        empty_desc,
        conflict,
        divergent,
        has_remote,
        is_synced,
    })
}
