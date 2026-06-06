use std::path::{Path, PathBuf};

use rand::RngCore;
use tokio::fs;

use crate::{
    api::{GetUploadUrlReq, UploadMediaType, WeixinApiOptions, get_upload_url},
    media::get_extension_from_content_type_or_url,
    util::temp_file_name,
};

use super::{aes_ecb::aes_ecb_padded_size, cdn_upload::upload_buffer_to_cdn};

#[derive(Clone, Debug)]
pub struct UploadedFileInfo {
    pub filekey: String,
    pub download_encrypted_query_param: String,
    pub aeskey: String,
    pub file_size: usize,
    pub file_size_ciphertext: usize,
}

async fn upload_media_to_cdn(
    file_path: impl AsRef<Path>,
    to_user_id: &str,
    opts: &WeixinApiOptions,
    cdn_base_url: &str,
    media_type: UploadMediaType,
    label: &str,
) -> crate::Result<UploadedFileInfo> {
    let plaintext = fs::read(file_path.as_ref()).await?;
    let rawsize = plaintext.len();
    let rawfilemd5 = format!("{:x}", md5::compute(&plaintext));
    let filesize = aes_ecb_padded_size(rawsize);
    let mut key_bytes = [0_u8; 16];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let mut filekey_bytes = [0_u8; 16];
    rand::thread_rng().fill_bytes(&mut filekey_bytes);
    let filekey = hex::encode(filekey_bytes);
    let aeskey = hex::encode(key_bytes);

    let resp = get_upload_url(
        GetUploadUrlReq {
            filekey: Some(filekey.clone()),
            media_type: Some(media_type as i32),
            to_user_id: Some(to_user_id.to_string()),
            rawsize: Some(rawsize),
            rawfilemd5: Some(rawfilemd5),
            filesize: Some(filesize),
            no_need_thumb: Some(true),
            aeskey: Some(aeskey.clone()),
            ..Default::default()
        },
        opts,
    )
    .await?;

    let uploaded = upload_buffer_to_cdn(
        &plaintext,
        resp.upload_full_url.as_deref(),
        resp.upload_param.as_deref(),
        &filekey,
        cdn_base_url,
        &key_bytes,
        label,
    )
    .await?;

    Ok(UploadedFileInfo {
        filekey,
        download_encrypted_query_param: uploaded.download_param,
        aeskey,
        file_size: rawsize,
        file_size_ciphertext: filesize,
    })
}

pub async fn download_remote_image_to_temp(
    url: &str,
    dest_dir: impl AsRef<Path>,
) -> crate::Result<PathBuf> {
    let res = reqwest::get(url).await?;
    let status = res.status();
    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);
    let buf = res.bytes().await?;
    if !status.is_success() {
        return Err(format!("remote media download failed: {status} url={url}").into());
    }
    fs::create_dir_all(dest_dir.as_ref()).await?;
    let ext = get_extension_from_content_type_or_url(content_type.as_deref(), Some(url));
    let path = dest_dir
        .as_ref()
        .join(temp_file_name("weixin-remote", &ext));
    fs::write(&path, buf).await?;
    Ok(path)
}

pub async fn upload_file_to_weixin(
    file_path: impl AsRef<Path>,
    to_user_id: &str,
    opts: &WeixinApiOptions,
    cdn_base_url: &str,
) -> crate::Result<UploadedFileInfo> {
    upload_media_to_cdn(
        file_path,
        to_user_id,
        opts,
        cdn_base_url,
        UploadMediaType::Image,
        "uploadFileToWeixin",
    )
    .await
}

pub async fn upload_video_to_weixin(
    file_path: impl AsRef<Path>,
    to_user_id: &str,
    opts: &WeixinApiOptions,
    cdn_base_url: &str,
) -> crate::Result<UploadedFileInfo> {
    upload_media_to_cdn(
        file_path,
        to_user_id,
        opts,
        cdn_base_url,
        UploadMediaType::Video,
        "uploadVideoToWeixin",
    )
    .await
}

pub async fn upload_file_attachment_to_weixin(
    file_path: impl AsRef<Path>,
    _file_name: &str,
    to_user_id: &str,
    opts: &WeixinApiOptions,
    cdn_base_url: &str,
) -> crate::Result<UploadedFileInfo> {
    upload_media_to_cdn(
        file_path,
        to_user_id,
        opts,
        cdn_base_url,
        UploadMediaType::File,
        "uploadFileAttachmentToWeixin",
    )
    .await
}
