use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    api::{MessageItem, MessageItemType, WeixinApiOptions, WeixinMessage},
    media::download_media_from_item_to_file,
    util::temp_file_name,
};

use super::send::{
    WeixinInboundMediaOpts, WeixinMsgContext, is_media_item, send_message_weixin,
    set_context_token, weixin_message_to_msg_context,
};

pub type MessageHandler = Arc<
    dyn Fn(WeixinMsgContext) -> Pin<Box<dyn Future<Output = crate::Result<Option<String>>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct ProcessMessageDeps {
    pub account_id: String,
    pub base_url: String,
    pub cdn_base_url: String,
    pub token: Option<String>,
    pub on_message: MessageHandler,
}

fn extract_text_body(items: Option<&[MessageItem]>) -> String {
    items
        .and_then(|items| {
            items.iter().find_map(|i| {
                (i.item_type == Some(MessageItemType::Text as i32))
                    .then(|| i.text_item.as_ref()?.text.clone())
                    .flatten()
            })
        })
        .unwrap_or_default()
}

fn has_media(item: &MessageItem) -> bool {
    let has = |m: Option<&crate::api::CdnMedia>| {
        m.map(|m| m.encrypt_query_param.is_some() || m.full_url.is_some())
            .unwrap_or(false)
    };
    match item.item_type {
        Some(x) if x == MessageItemType::Image as i32 => {
            has(item.image_item.as_ref().and_then(|i| i.media.as_ref()))
        }
        Some(x) if x == MessageItemType::Video as i32 => {
            has(item.video_item.as_ref().and_then(|i| i.media.as_ref()))
        }
        Some(x) if x == MessageItemType::File as i32 => {
            has(item.file_item.as_ref().and_then(|i| i.media.as_ref()))
        }
        Some(x) if x == MessageItemType::Voice as i32 => item
            .voice_item
            .as_ref()
            .and_then(|v| v.media.as_ref())
            .map(|m| {
                (m.encrypt_query_param.is_some() || m.full_url.is_some())
                    && item
                        .voice_item
                        .as_ref()
                        .and_then(|v| v.text.as_ref())
                        .is_none()
            })
            .unwrap_or(false),
        _ => false,
    }
}

pub async fn process_one_message(
    full: WeixinMessage,
    deps: &ProcessMessageDeps,
) -> crate::Result<()> {
    let text_body = extract_text_body(full.item_list.as_deref());
    crate::util::logger()
        .with_account(&deps.account_id)
        .info(format!(
            "inbound: from={:?} body=\"{}\" items={}",
            full.from_user_id,
            text_body.chars().take(50).collect::<String>(),
            full.item_list.as_ref().map(|v| v.len()).unwrap_or(0)
        ));

    let mut media_opts = WeixinInboundMediaOpts::default();
    let media_item = full.item_list.as_deref().and_then(|items| {
        items
            .iter()
            .find(|i| i.item_type == Some(MessageItemType::Image as i32) && has_media(i))
            .or_else(|| {
                items
                    .iter()
                    .find(|i| i.item_type == Some(MessageItemType::Video as i32) && has_media(i))
            })
            .or_else(|| {
                items
                    .iter()
                    .find(|i| i.item_type == Some(MessageItemType::File as i32) && has_media(i))
            })
            .or_else(|| {
                items
                    .iter()
                    .find(|i| i.item_type == Some(MessageItemType::Voice as i32) && has_media(i))
            })
            .or_else(|| {
                items.iter().find_map(|i| {
                    (i.item_type == Some(MessageItemType::Text as i32))
                        .then(|| i.ref_msg.as_ref()?.message_item.as_deref())
                        .flatten()
                        .filter(|m| is_media_item(m))
                })
            })
    });
    if let Some(item) = media_item {
        let dir = std::env::temp_dir().join("weixin-bot-media");
        tokio::fs::create_dir_all(&dir).await?;
        media_opts = download_media_from_item_to_file(item, &deps.cdn_base_url, |content_type| {
            let ext = if content_type.contains("image") {
                ".jpg"
            } else if content_type.contains("video") {
                ".mp4"
            } else if content_type.contains("audio") {
                ".silk"
            } else {
                ".bin"
            };
            dir.join(temp_file_name("media", ext))
        })
        .await
        .unwrap_or_default();
    }

    let ctx = weixin_message_to_msg_context(&full, &deps.account_id, Some(media_opts));
    if let (Some(token), Some(from)) = (&full.context_token, &full.from_user_id) {
        set_context_token(&deps.account_id, from, token);
    }

    if let Some(reply) = (deps.on_message)(ctx.clone()).await? {
        let opts = WeixinApiOptions {
            base_url: deps.base_url.clone(),
            token: deps.token.clone(),
            timeout_ms: None,
            long_poll_timeout_ms: None,
        };
        send_message_weixin(&ctx.from, &reply, &opts, ctx.context_token.as_deref()).await?;
    }
    Ok(())
}
