use std::path::Path;

use crate::{
    api::WeixinApiOptions,
    cdn::{
        download_remote_image_to_temp, upload_file_attachment_to_weixin, upload_file_to_weixin,
        upload_video_to_weixin,
    },
};

use super::send::{
    SendResult, send_file_message_weixin, send_image_message_weixin, send_video_message_weixin,
};

pub async fn send_weixin_media_file(
    file_path: impl AsRef<Path>,
    to: &str,
    text: &str,
    opts: &WeixinApiOptions,
    cdn_base_url: &str,
    context_token: Option<&str>,
) -> crate::Result<SendResult> {
    let path = file_path.as_ref();
    let mime = crate::media::get_mime_from_filename(path);
    if mime.starts_with("video/") {
        let uploaded = upload_video_to_weixin(path, to, opts, cdn_base_url).await?;
        send_video_message_weixin(to, text, &uploaded, opts, context_token).await
    } else if mime.starts_with("image/") {
        let uploaded = upload_file_to_weixin(path, to, opts, cdn_base_url).await?;
        send_image_message_weixin(to, text, &uploaded, opts, context_token).await
    } else {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file.bin");
        let uploaded = upload_file_attachment_to_weixin(path, name, to, opts, cdn_base_url).await?;
        send_file_message_weixin(to, text, name, &uploaded, opts, context_token).await
    }
}

pub async fn send_media_url(
    url: &str,
    to: &str,
    text: &str,
    opts: &WeixinApiOptions,
    cdn_base_url: &str,
    context_token: Option<&str>,
) -> crate::Result<SendResult> {
    let dir = std::env::temp_dir().join("weixin-bot-remote");
    let path = download_remote_image_to_temp(url, &dir).await?;
    send_weixin_media_file(path, to, text, opts, cdn_base_url, context_token).await
}
