use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{messaging::send::clear_context_tokens_for_account, storage::resolve_state_dir};

pub const DEFAULT_BASE_URL: &str = "https://ilinkai.weixin.qq.com";
pub const CDN_BASE_URL: &str = "https://novac2c.cdn.weixin.qq.com/c2c";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WeixinAccountData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ResolvedWeixinAccount {
    pub account_id: String,
    pub base_url: String,
    pub cdn_base_url: String,
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub configured: bool,
}

fn accounts_dir() -> PathBuf {
    resolve_state_dir().join("accounts")
}
fn account_path(account_id: &str) -> PathBuf {
    accounts_dir().join(format!("{account_id}.json"))
}
fn account_index_path() -> PathBuf {
    resolve_state_dir().join("accounts.json")
}

pub(crate) fn normalize_account_id(account_id: &str) -> String {
    account_id
        .trim()
        .replace('@', "-")
        .replace('.', "-")
        .to_ascii_lowercase()
}

pub(crate) fn derive_raw_account_id(normalized_id: &str) -> Option<String> {
    normalized_id
        .strip_suffix("-im-bot")
        .map(|prefix| format!("{prefix}@im.bot"))
        .or_else(|| {
            normalized_id
                .strip_suffix("-im-wechat")
                .map(|prefix| format!("{prefix}@im.wechat"))
        })
}

fn account_path_candidates(account_id: &str) -> Vec<PathBuf> {
    let mut ids = vec![account_id.trim().to_string()];
    let normalized = normalize_account_id(account_id);
    if !ids.iter().any(|id| id == &normalized) {
        ids.push(normalized);
    }
    if let Some(raw) = derive_raw_account_id(account_id)
        && !ids.iter().any(|id| id == &raw)
    {
        ids.push(raw);
    }
    ids.into_iter().map(|id| account_path(&id)).collect()
}

pub fn list_weixin_account_ids() -> Vec<String> {
    fs::read_to_string(account_index_path())
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|id| !id.trim().is_empty())
        .collect()
}

pub fn register_weixin_account_id(account_id: &str) -> crate::Result<()> {
    fs::create_dir_all(resolve_state_dir())?;
    let mut ids = list_weixin_account_ids();
    if !ids.iter().any(|id| id == account_id) {
        ids.push(account_id.to_string());
        fs::write(account_index_path(), serde_json::to_string_pretty(&ids)?)?;
    }
    Ok(())
}

pub fn unregister_weixin_account_id(account_id: &str) -> crate::Result<()> {
    let existing = list_weixin_account_ids();
    let updated: Vec<_> = existing.into_iter().filter(|id| id != account_id).collect();
    fs::write(
        account_index_path(),
        serde_json::to_string_pretty(&updated)?,
    )?;
    Ok(())
}

pub fn load_weixin_account(account_id: &str) -> Option<WeixinAccountData> {
    for path in account_path_candidates(account_id) {
        if let Some(data) = fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            return Some(data);
        }
    }
    None
}

pub fn save_weixin_account(
    account_id: &str,
    token: Option<&str>,
    base_url: Option<&str>,
    user_id: Option<&str>,
) -> crate::Result<()> {
    fs::create_dir_all(accounts_dir())?;
    let account_id = normalize_account_id(account_id);
    let existing = load_weixin_account(&account_id).unwrap_or_default();
    let token = token
        .and_then(non_empty)
        .map(ToOwned::to_owned)
        .or(existing.token);
    let base_url = base_url
        .and_then(non_empty)
        .map(ToOwned::to_owned)
        .or(existing.base_url);
    let user_id = user_id
        .and_then(non_empty)
        .map(ToOwned::to_owned)
        .or(existing.user_id);
    let data = WeixinAccountData {
        saved_at: token.as_ref().map(|_| Utc::now().to_rfc3339()),
        token,
        base_url,
        user_id,
    };
    fs::write(
        account_path(&account_id),
        serde_json::to_string_pretty(&data)?,
    )?;
    register_weixin_account_id(&account_id)?;
    Ok(())
}

fn non_empty(s: &str) -> Option<&str> {
    let t = s.trim();
    (!t.is_empty()).then_some(t)
}

pub fn clear_weixin_account(account_id: &str) {
    for file in [
        format!("{account_id}.json"),
        format!("{account_id}.sync.json"),
        format!("{account_id}.context-tokens.json"),
    ] {
        let _ = fs::remove_file(accounts_dir().join(file));
    }
}

pub fn clear_stale_accounts_for_user_id(
    current_account_id: &str,
    user_id: &str,
) -> crate::Result<()> {
    if user_id.is_empty() {
        return Ok(());
    }
    for id in list_weixin_account_ids() {
        if id == current_account_id {
            continue;
        }
        if load_weixin_account(&id)
            .and_then(|d| d.user_id)
            .map(|u| u.trim() == user_id)
            .unwrap_or(false)
        {
            clear_context_tokens_for_account(&id);
            clear_weixin_account(&id);
            unregister_weixin_account_id(&id)?;
        }
    }
    Ok(())
}

pub fn resolve_weixin_account(account_id: &str) -> crate::Result<ResolvedWeixinAccount> {
    if account_id.trim().is_empty() {
        return Err("accountId is required".into());
    }
    let account_id = normalize_account_id(account_id);
    let data = load_weixin_account(&account_id);
    let token = data
        .as_ref()
        .and_then(|d| d.token.as_ref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok(ResolvedWeixinAccount {
        account_id: account_id.to_string(),
        base_url: data
            .as_ref()
            .and_then(|d| d.base_url.as_ref())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        cdn_base_url: CDN_BASE_URL.to_string(),
        user_id: data
            .as_ref()
            .and_then(|d| d.user_id.as_ref())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        configured: token.is_some(),
        token,
    })
}
