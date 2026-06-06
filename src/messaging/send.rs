use base64::{Engine as _, engine::general_purpose::STANDARD};
use once_cell::sync::Lazy;
use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};

use crate::{
    api::{
        CdnMedia, FileItem, ImageItem, MessageItem, MessageItemType, MessageState, MessageType,
        SendMessageReq, TextItem, VideoItem, WeixinApiOptions, WeixinMessage,
        send_message as send_message_api,
    },
    cdn::UploadedFileInfo,
    storage::resolve_state_dir,
    util::generate_id,
};

#[derive(Clone, Debug, Default)]
pub struct WeixinInboundMediaOpts {
    pub decrypted_pic_path: Option<String>,
    pub decrypted_voice_path: Option<String>,
    pub voice_media_type: Option<String>,
    pub decrypted_file_path: Option<String>,
    pub file_media_type: Option<String>,
    pub decrypted_video_path: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct WeixinMsgContext {
    pub body: String,
    pub from: String,
    pub to: String,
    pub account_id: String,
    pub message_sid: String,
    pub timestamp: Option<i64>,
    pub chat_type: String,
    pub context_token: Option<String>,
    pub media_url: Option<String>,
    pub media_path: Option<String>,
    pub media_type: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SendResult {
    pub message_id: String,
}

static CONTEXT_TOKEN_STORE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn context_token_key(account_id: &str, user_id: &str) -> String {
    format!("{account_id}:{user_id}")
}
fn context_token_file(account_id: &str) -> PathBuf {
    resolve_state_dir()
        .join("accounts")
        .join(format!("{account_id}.context-tokens.json"))
}

fn persist_context_tokens(account_id: &str) {
    let Ok(store) = CONTEXT_TOKEN_STORE.lock() else {
        return;
    };
    let prefix = format!("{account_id}:");
    let tokens: HashMap<String, String> = store
        .iter()
        .filter_map(|(k, v)| k.strip_prefix(&prefix).map(|u| (u.to_string(), v.clone())))
        .collect();
    let path = context_token_file(account_id);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, serde_json::to_string(&tokens).unwrap_or_default());
}

pub fn restore_context_tokens(account_id: &str) {
    let Ok(text) = fs::read_to_string(context_token_file(account_id)) else {
        return;
    };
    let Ok(tokens) = serde_json::from_str::<HashMap<String, String>>(&text) else {
        return;
    };
    if let Ok(mut store) = CONTEXT_TOKEN_STORE.lock() {
        for (user_id, token) in tokens {
            if !token.is_empty() {
                store.insert(context_token_key(account_id, &user_id), token);
            }
        }
    }
}

pub fn clear_context_tokens_for_account(account_id: &str) {
    if let Ok(mut store) = CONTEXT_TOKEN_STORE.lock() {
        let prefix = format!("{account_id}:");
        store.retain(|k, _| !k.starts_with(&prefix));
    }
    let _ = fs::remove_file(context_token_file(account_id));
}

pub fn set_context_token(account_id: &str, user_id: &str, token: &str) {
    if let Ok(mut store) = CONTEXT_TOKEN_STORE.lock() {
        store.insert(context_token_key(account_id, user_id), token.to_string());
    }
    persist_context_tokens(account_id);
}

pub fn get_context_token(account_id: &str, user_id: &str) -> Option<String> {
    CONTEXT_TOKEN_STORE
        .lock()
        .ok()?
        .get(&context_token_key(account_id, user_id))
        .cloned()
}

fn generate_client_id() -> String {
    generate_id("weixin-bot")
}

pub fn is_media_item(item: &MessageItem) -> bool {
    matches!(item.item_type, Some(x) if x == MessageItemType::Image as i32 || x == MessageItemType::Video as i32 || x == MessageItemType::File as i32 || x == MessageItemType::Voice as i32)
}

pub fn body_from_item_list(item_list: Option<&[MessageItem]>) -> String {
    let Some(items) = item_list else {
        return String::new();
    };
    for item in items {
        if item.item_type == Some(MessageItemType::Text as i32)
            && let Some(text) = item.text_item.as_ref().and_then(|t| t.text.as_ref())
        {
            if let Some(ref_msg) = &item.ref_msg {
                if ref_msg
                    .message_item
                    .as_deref()
                    .map(is_media_item)
                    .unwrap_or(false)
                {
                    return text.clone();
                }
                let mut parts = Vec::new();
                if let Some(title) = &ref_msg.title {
                    parts.push(title.clone());
                }
                if let Some(mi) = ref_msg.message_item.as_deref() {
                    let b = body_from_item_list(Some(std::slice::from_ref(mi)));
                    if !b.is_empty() {
                        parts.push(b);
                    }
                }
                if !parts.is_empty() {
                    return format!("[引用: {}]\n{text}", parts.join(" | "));
                }
            }
            return text.clone();
        }
        if item.item_type == Some(MessageItemType::Voice as i32)
            && let Some(text) = item.voice_item.as_ref().and_then(|v| v.text.as_ref())
        {
            return text.clone();
        }
    }
    String::new()
}

pub fn weixin_message_to_msg_context(
    msg: &WeixinMessage,
    account_id: &str,
    opts: Option<WeixinInboundMediaOpts>,
) -> WeixinMsgContext {
    let from = msg.from_user_id.clone().unwrap_or_default();
    let mut ctx = WeixinMsgContext {
        body: body_from_item_list(msg.item_list.as_deref()),
        from: from.clone(),
        to: from,
        account_id: account_id.to_string(),
        message_sid: generate_client_id(),
        timestamp: msg.create_time_ms,
        chat_type: "direct".into(),
        context_token: msg.context_token.clone(),
        ..Default::default()
    };
    if let Some(o) = opts {
        if let Some(p) = o.decrypted_pic_path {
            ctx.media_path = Some(p);
            ctx.media_type = Some("image/*".into());
        } else if let Some(p) = o.decrypted_video_path {
            ctx.media_path = Some(p);
            ctx.media_type = Some("video/mp4".into());
        } else if let Some(p) = o.decrypted_file_path {
            ctx.media_path = Some(p);
            ctx.media_type = Some(
                o.file_media_type
                    .unwrap_or_else(|| "application/octet-stream".into()),
            );
        } else if let Some(p) = o.decrypted_voice_path {
            ctx.media_path = Some(p);
            ctx.media_type = Some(o.voice_media_type.unwrap_or_else(|| "audio/wav".into()));
        }
    }
    ctx
}

fn text_req(to: &str, text: &str, context_token: Option<&str>, client_id: &str) -> SendMessageReq {
    let item_list = (!text.is_empty()).then(|| {
        vec![MessageItem {
            item_type: Some(MessageItemType::Text as i32),
            text_item: Some(TextItem {
                text: Some(text.to_string()),
            }),
            ..Default::default()
        }]
    });
    SendMessageReq {
        msg: Some(WeixinMessage {
            from_user_id: Some(String::new()),
            to_user_id: Some(to.to_string()),
            client_id: Some(client_id.to_string()),
            message_type: Some(MessageType::Bot as i32),
            message_state: Some(MessageState::Finish as i32),
            item_list,
            context_token: context_token.map(ToOwned::to_owned),
            ..Default::default()
        }),
    }
}

async fn send_media_items(
    to: &str,
    text: &str,
    media_item: MessageItem,
    opts: &WeixinApiOptions,
    context_token: Option<&str>,
) -> crate::Result<SendResult> {
    let mut items = Vec::new();
    if !text.is_empty() {
        items.push(MessageItem {
            item_type: Some(MessageItemType::Text as i32),
            text_item: Some(TextItem {
                text: Some(text.to_string()),
            }),
            ..Default::default()
        });
    }
    items.push(media_item);
    let mut last = String::new();
    for item in items {
        last = generate_client_id();
        let req = SendMessageReq {
            msg: Some(WeixinMessage {
                from_user_id: Some(String::new()),
                to_user_id: Some(to.to_string()),
                client_id: Some(last.clone()),
                message_type: Some(MessageType::Bot as i32),
                message_state: Some(MessageState::Finish as i32),
                item_list: Some(vec![item]),
                context_token: context_token.map(ToOwned::to_owned),
                ..Default::default()
            }),
        };
        send_message_api(req, opts).await?;
    }
    Ok(SendResult { message_id: last })
}

pub async fn send_message_weixin(
    to: &str,
    text: &str,
    opts: &WeixinApiOptions,
    context_token: Option<&str>,
) -> crate::Result<SendResult> {
    let client_id = generate_client_id();
    send_message_api(text_req(to, text, context_token, &client_id), opts).await?;
    Ok(SendResult {
        message_id: client_id,
    })
}

fn media(aes_hex: &str, param: &str) -> CdnMedia {
    CdnMedia {
        encrypt_query_param: Some(param.to_string()),
        aes_key: Some(STANDARD.encode(hex::decode(aes_hex).unwrap_or_default())),
        encrypt_type: Some(1),
        full_url: None,
    }
}

pub async fn send_image_message_weixin(
    to: &str,
    text: &str,
    uploaded: &UploadedFileInfo,
    opts: &WeixinApiOptions,
    context_token: Option<&str>,
) -> crate::Result<SendResult> {
    let item = MessageItem {
        item_type: Some(MessageItemType::Image as i32),
        image_item: Some(ImageItem {
            media: Some(media(
                &uploaded.aeskey,
                &uploaded.download_encrypted_query_param,
            )),
            mid_size: Some(uploaded.file_size_ciphertext),
            ..Default::default()
        }),
        ..Default::default()
    };
    send_media_items(to, text, item, opts, context_token).await
}

pub async fn send_video_message_weixin(
    to: &str,
    text: &str,
    uploaded: &UploadedFileInfo,
    opts: &WeixinApiOptions,
    context_token: Option<&str>,
) -> crate::Result<SendResult> {
    let item = MessageItem {
        item_type: Some(MessageItemType::Video as i32),
        video_item: Some(VideoItem {
            media: Some(media(
                &uploaded.aeskey,
                &uploaded.download_encrypted_query_param,
            )),
            video_size: Some(uploaded.file_size_ciphertext),
            ..Default::default()
        }),
        ..Default::default()
    };
    send_media_items(to, text, item, opts, context_token).await
}

pub async fn send_file_message_weixin(
    to: &str,
    text: &str,
    file_name: &str,
    uploaded: &UploadedFileInfo,
    opts: &WeixinApiOptions,
    context_token: Option<&str>,
) -> crate::Result<SendResult> {
    let item = MessageItem {
        item_type: Some(MessageItemType::File as i32),
        file_item: Some(FileItem {
            media: Some(media(
                &uploaded.aeskey,
                &uploaded.download_encrypted_query_param,
            )),
            file_name: Some(file_name.to_string()),
            len: Some(uploaded.file_size.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    send_media_items(to, text, item, opts, context_token).await
}
