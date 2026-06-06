pub mod config_cache;
pub mod session_guard;

pub use config_cache::{CachedConfig, WeixinConfigManager};
pub use session_guard::{SESSION_EXPIRED_ERRCODE, get_remaining_pause_ms, pause_session};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::util::random_uint32_base64;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const APP_ID: &str = "bot";
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LONG_POLL_TIMEOUT_MS: u64 = 35_000;
const DEFAULT_API_TIMEOUT_MS: u64 = 15_000;
const DEFAULT_CONFIG_TIMEOUT_MS: u64 = 10_000;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BaseInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_agent: Option<String>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UploadMediaType {
    Image = 1,
    Video = 2,
    File = 3,
    Voice = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    None = 0,
    User = 1,
    Bot = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageItemType {
    None = 0,
    Text = 1,
    Image = 2,
    Voice = 3,
    File = 4,
    Video = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageState {
    New = 0,
    Generating = 1,
    Finish = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypingStatus {
    Typing = 1,
    Cancel = 2,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TextItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CdnMedia {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt_query_param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aes_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImageItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CdnMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_media: Option<CdnMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aeskey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mid_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_height: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_width: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd_size: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VoiceItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CdnMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encode_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits_per_sample: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playtime: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FileItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CdnMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub len: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VideoItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CdnMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_media: Option<CdnMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_height: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_width: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RefMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_item: Option<Box<MessageItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MessageItem {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_msg: Option<RefMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_item: Option<TextItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_item: Option<ImageItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_item: Option<VoiceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_item: Option<FileItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_item: Option<VideoItem>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WeixinMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_time_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_state: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_list: Option<Vec<MessageItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GetUpdatesReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_updates_buf: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GetUpdatesResp {
    pub ret: Option<i32>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub msgs: Option<Vec<WeixinMessage>>,
    pub get_updates_buf: Option<String>,
    pub longpolling_timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SendMessageReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<WeixinMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GetUploadUrlReq {
    pub filekey: Option<String>,
    pub media_type: Option<i32>,
    pub to_user_id: Option<String>,
    pub rawsize: Option<usize>,
    pub rawfilemd5: Option<String>,
    pub filesize: Option<usize>,
    pub thumb_rawsize: Option<usize>,
    pub thumb_rawfilemd5: Option<String>,
    pub thumb_filesize: Option<usize>,
    pub no_need_thumb: Option<bool>,
    pub aeskey: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GetUploadUrlResp {
    pub upload_param: Option<String>,
    pub thumb_upload_param: Option<String>,
    pub upload_full_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SendTypingReq {
    pub ilink_user_id: Option<String>,
    pub typing_ticket: Option<String>,
    pub status: Option<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SendTypingResp {
    pub ret: Option<i32>,
    pub errmsg: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GetConfigResp {
    pub ret: Option<i32>,
    pub errmsg: Option<String>,
    pub typing_ticket: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NotifyStopResp {
    pub ret: Option<i32>,
    pub errmsg: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NotifyStartResp {
    pub ret: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Clone, Debug)]
pub struct WeixinApiOptions {
    pub base_url: String,
    pub token: Option<String>,
    pub timeout_ms: Option<u64>,
    pub long_poll_timeout_ms: Option<u64>,
}

impl WeixinApiOptions {
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token,
            timeout_ms: None,
            long_poll_timeout_ms: None,
        }
    }
}

fn encode_version(version: &str) -> u32 {
    let parts: Vec<u32> = version.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    ((parts.first().copied().unwrap_or(0) & 0xff) << 16)
        | ((parts.get(1).copied().unwrap_or(0) & 0xff) << 8)
        | (parts.get(2).copied().unwrap_or(0) & 0xff)
}

pub fn build_base_info() -> BaseInfo {
    BaseInfo {
        channel_version: Some(PACKAGE_VERSION.to_string()),
        bot_agent: Some(format!("weixin-bot-sdk/{PACKAGE_VERSION}")),
    }
}

pub fn sanitize_bot_agent(raw: &str) -> String {
    let trimmed = raw.trim();
    let cleaned: String = trimmed
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .take(256)
        .collect();
    if cleaned.is_empty() {
        format!("weixin-bot-sdk/{PACKAGE_VERSION}")
    } else {
        cleaned
    }
}

fn common_headers(
    req: reqwest::RequestBuilder,
    token: Option<&str>,
    json: bool,
) -> reqwest::RequestBuilder {
    let mut b = req.header("iLink-App-Id", APP_ID).header(
        "iLink-App-ClientVersion",
        encode_version(PACKAGE_VERSION).to_string(),
    );
    if json {
        b = b
            .header("Content-Type", "application/json")
            .header("AuthorizationType", "ilink_bot_token")
            .header("X-WECHAT-UIN", random_uint32_base64());
    }
    if let Some(t) = token.map(str::trim).filter(|t| !t.is_empty()) {
        b = b.header("Authorization", format!("Bearer {t}"));
    }
    b
}

fn endpoint_url(base_url: &str, endpoint: &str) -> Result<String> {
    let base = if base_url.ends_with('/') {
        base_url.to_string()
    } else {
        format!("{base_url}/")
    };
    Ok(url::Url::parse(&base)?.join(endpoint)?.to_string())
}

pub async fn api_post_fetch(
    base_url: &str,
    endpoint: &str,
    body: String,
    token: Option<&str>,
    timeout_ms: Option<u64>,
    label: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let url = endpoint_url(base_url, endpoint)?;
    let res = common_headers(
        client.post(url).timeout(std::time::Duration::from_millis(
            timeout_ms.unwrap_or(DEFAULT_API_TIMEOUT_MS),
        )),
        token,
        true,
    )
    .body(body)
    .send()
    .await?;
    let status = res.status();
    let text = res.text().await?;
    if !status.is_success() {
        return Err(format!("{label} HTTP {status}: {text}").into());
    }
    Ok(text)
}

pub async fn api_get_fetch(
    base_url: &str,
    endpoint: &str,
    timeout_ms: Option<u64>,
    label: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let url = endpoint_url(base_url, endpoint)?;
    let res = common_headers(
        client.get(url).timeout(std::time::Duration::from_millis(
            timeout_ms.unwrap_or(DEFAULT_API_TIMEOUT_MS),
        )),
        None,
        false,
    )
    .send()
    .await?;
    let status = res.status();
    let text = res.text().await?;
    if !status.is_success() {
        return Err(format!("{label} HTTP {status}: {text}").into());
    }
    Ok(text)
}

pub async fn get_updates(params: GetUpdatesReq, opts: &WeixinApiOptions) -> Result<GetUpdatesResp> {
    let timeout = opts
        .long_poll_timeout_ms
        .or(opts.timeout_ms)
        .unwrap_or(DEFAULT_LONG_POLL_TIMEOUT_MS);
    let raw = api_post_fetch(
        &opts.base_url,
        "ilink/bot/getupdates",
        json!({"get_updates_buf": params.get_updates_buf.unwrap_or_default(), "base_info": build_base_info()}).to_string(),
        opts.token.as_deref(),
        Some(timeout),
        "getUpdates",
    ).await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn get_upload_url(
    params: GetUploadUrlReq,
    opts: &WeixinApiOptions,
) -> Result<GetUploadUrlResp> {
    let mut v = serde_json::to_value(params)?;
    v.as_object_mut()
        .unwrap()
        .insert("base_info".into(), serde_json::to_value(build_base_info())?);
    let raw = api_post_fetch(
        &opts.base_url,
        "ilink/bot/getuploadurl",
        v.to_string(),
        opts.token.as_deref(),
        opts.timeout_ms,
        "getUploadUrl",
    )
    .await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn send_message(params: SendMessageReq, opts: &WeixinApiOptions) -> Result<()> {
    let mut v = serde_json::to_value(params)?;
    v.as_object_mut()
        .unwrap()
        .insert("base_info".into(), serde_json::to_value(build_base_info())?);
    api_post_fetch(
        &opts.base_url,
        "ilink/bot/sendmessage",
        v.to_string(),
        opts.token.as_deref(),
        opts.timeout_ms,
        "sendMessage",
    )
    .await?;
    Ok(())
}

pub async fn get_config(
    opts: &WeixinApiOptions,
    ilink_user_id: &str,
    context_token: Option<&str>,
) -> Result<GetConfigResp> {
    let raw = api_post_fetch(
        &opts.base_url,
        "ilink/bot/getconfig",
        json!({
            "ilink_user_id": ilink_user_id,
            "context_token": context_token,
            "base_info": build_base_info(),
        })
        .to_string(),
        opts.token.as_deref(),
        Some(DEFAULT_CONFIG_TIMEOUT_MS),
        "getConfig",
    )
    .await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn send_typing(opts: &WeixinApiOptions, body: SendTypingReq) -> Result<SendTypingResp> {
    let mut v = serde_json::to_value(body)?;
    v.as_object_mut()
        .unwrap()
        .insert("base_info".into(), serde_json::to_value(build_base_info())?);
    let raw = api_post_fetch(
        &opts.base_url,
        "ilink/bot/sendtyping",
        v.to_string(),
        opts.token.as_deref(),
        Some(DEFAULT_CONFIG_TIMEOUT_MS),
        "sendTyping",
    )
    .await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn notify_start(opts: &WeixinApiOptions) -> Result<NotifyStartResp> {
    let raw = api_post_fetch(
        &opts.base_url,
        "ilink/bot/msg/notifystart",
        json!({"base_info": build_base_info()}).to_string(),
        opts.token.as_deref(),
        Some(DEFAULT_CONFIG_TIMEOUT_MS),
        "notifyStart",
    )
    .await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn notify_stop(opts: &WeixinApiOptions) -> Result<NotifyStopResp> {
    let raw = api_post_fetch(
        &opts.base_url,
        "ilink/bot/msg/notifystop",
        json!({"base_info": build_base_info()}).to_string(),
        opts.token.as_deref(),
        Some(DEFAULT_CONFIG_TIMEOUT_MS),
        "notifyStop",
    )
    .await?;
    Ok(serde_json::from_str(&raw)?)
}
