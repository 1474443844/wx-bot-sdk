use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::auth::accounts::{derive_raw_account_id, normalize_account_id};

use super::state_dir::resolve_state_dir;

fn sync_buf_path_for(account_id: &str) -> PathBuf {
    resolve_state_dir()
        .join("accounts")
        .join(format!("{account_id}.sync.json"))
}

pub fn get_sync_buf_file_path(account_id: &str) -> PathBuf {
    sync_buf_path_for(&normalize_account_id(account_id))
}

pub fn get_sync_buf_file_path_candidates(account_id: &str) -> Vec<PathBuf> {
    let mut ids = vec![normalize_account_id(account_id)];
    let raw = account_id.trim().to_string();
    if !ids.iter().any(|id| id == &raw) {
        ids.push(raw);
    }
    if let Some(raw) = derive_raw_account_id(account_id)
        && !ids.iter().any(|id| id == &raw)
    {
        ids.push(raw);
    }
    ids.into_iter().map(|id| sync_buf_path_for(&id)).collect()
}

pub fn load_get_updates_buf(path: impl AsRef<Path>) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("get_updates_buf")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
}

pub fn save_get_updates_buf(path: impl AsRef<Path>, buf: &str) -> crate::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::json!({"get_updates_buf": buf}).to_string(),
    )?;
    Ok(())
}
