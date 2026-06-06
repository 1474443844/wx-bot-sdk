use std::{
    fs,
    path::{Path, PathBuf},
};

use super::state_dir::resolve_state_dir;

pub fn get_sync_buf_file_path(account_id: &str) -> PathBuf {
    resolve_state_dir()
        .join("accounts")
        .join(format!("{account_id}.sync.json"))
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
