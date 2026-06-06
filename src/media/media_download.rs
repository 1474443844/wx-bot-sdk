use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::path::PathBuf;

use crate::{
    api::{MessageItem, MessageItemType},
    cdn::{download_and_decrypt_buffer, download_plain_cdn_buffer},
    messaging::send::WeixinInboundMediaOpts,
};

use super::get_mime_from_filename;

pub async fn download_media_from_item_to_file(
    item: &MessageItem,
    cdn_base_url: &str,
    dest_path_for: impl Fn(&str) -> PathBuf,
) -> crate::Result<WeixinInboundMediaOpts> {
    let mut result = WeixinInboundMediaOpts::default();
    match item.item_type {
        Some(x) if x == MessageItemType::Image as i32 => {
            let Some(img) = &item.image_item else {
                return Ok(result);
            };
            let Some(media) = &img.media else {
                return Ok(result);
            };
            if media.encrypt_query_param.is_none() && media.full_url.is_none() {
                return Ok(result);
            }
            let aes_key = if let Some(hex_key) = &img.aeskey {
                Some(STANDARD.encode(hex::decode(hex_key)?))
            } else {
                media.aes_key.clone()
            };
            let buf = if let Some(key) = aes_key {
                download_and_decrypt_buffer(
                    media.encrypt_query_param.as_deref().unwrap_or(""),
                    &key,
                    cdn_base_url,
                    "inbound image",
                    media.full_url.as_deref(),
                )
                .await?
            } else {
                download_plain_cdn_buffer(
                    media.encrypt_query_param.as_deref().unwrap_or(""),
                    cdn_base_url,
                    "inbound image-plain",
                    media.full_url.as_deref(),
                )
                .await?
            };
            let path = dest_path_for("image/jpeg");
            tokio::fs::write(&path, buf).await?;
            result.decrypted_pic_path = Some(path.to_string_lossy().to_string());
        }
        Some(x) if x == MessageItemType::Voice as i32 => {
            let Some(voice) = &item.voice_item else {
                return Ok(result);
            };
            let Some(media) = &voice.media else {
                return Ok(result);
            };
            let Some(key) = &media.aes_key else {
                return Ok(result);
            };
            let buf = download_and_decrypt_buffer(
                media.encrypt_query_param.as_deref().unwrap_or(""),
                key,
                cdn_base_url,
                "inbound voice",
                media.full_url.as_deref(),
            )
            .await?;
            let path = dest_path_for("audio/silk");
            tokio::fs::write(&path, buf).await?;
            result.decrypted_voice_path = Some(path.to_string_lossy().to_string());
            result.voice_media_type = Some("audio/silk".into());
        }
        Some(x) if x == MessageItemType::File as i32 => {
            let Some(file) = &item.file_item else {
                return Ok(result);
            };
            let Some(media) = &file.media else {
                return Ok(result);
            };
            let Some(key) = &media.aes_key else {
                return Ok(result);
            };
            let buf = download_and_decrypt_buffer(
                media.encrypt_query_param.as_deref().unwrap_or(""),
                key,
                cdn_base_url,
                "inbound file",
                media.full_url.as_deref(),
            )
            .await?;
            let mime = get_mime_from_filename(file.file_name.as_deref().unwrap_or("file.bin"));
            let path = dest_path_for(&mime);
            tokio::fs::write(&path, buf).await?;
            result.decrypted_file_path = Some(path.to_string_lossy().to_string());
            result.file_media_type = Some(mime);
        }
        Some(x) if x == MessageItemType::Video as i32 => {
            let Some(video) = &item.video_item else {
                return Ok(result);
            };
            let Some(media) = &video.media else {
                return Ok(result);
            };
            let Some(key) = &media.aes_key else {
                return Ok(result);
            };
            let buf = download_and_decrypt_buffer(
                media.encrypt_query_param.as_deref().unwrap_or(""),
                key,
                cdn_base_url,
                "inbound video",
                media.full_url.as_deref(),
            )
            .await?;
            let path = dest_path_for("video/mp4");
            tokio::fs::write(&path, buf).await?;
            result.decrypted_video_path = Some(path.to_string_lossy().to_string());
        }
        _ => {}
    }
    Ok(result)
}
