use reqwest::header::HeaderMap;

use super::{aes_ecb::encrypt_aes_ecb, cdn_url::build_cdn_upload_url};

#[derive(Clone, Debug)]
pub struct UploadBufferToCdnResult {
    pub download_param: String,
}

pub async fn upload_buffer_to_cdn(
    buf: &[u8],
    upload_full_url: Option<&str>,
    upload_param: Option<&str>,
    filekey: &str,
    cdn_base_url: &str,
    aeskey: &[u8],
    label: &str,
) -> crate::Result<UploadBufferToCdnResult> {
    let url = match upload_full_url.filter(|s| !s.trim().is_empty()) {
        Some(u) => u.to_string(),
        None => build_cdn_upload_url(
            cdn_base_url,
            upload_param.ok_or("missing upload_param")?,
            filekey,
        ),
    };
    let encrypted = encrypt_aes_ecb(buf, aeskey)?;
    let res = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/octet-stream")
        .body(encrypted)
        .send()
        .await?;
    let status = res.status();
    let headers = res.headers().clone();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("{label} CDN upload HTTP {status}: {body}").into());
    }
    let download_param = encrypted_param_from_headers(&headers)
        .ok_or_else(|| format!("{label}: CDN response missing x-encrypted-param"))?;
    Ok(UploadBufferToCdnResult { download_param })
}

fn encrypted_param_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-encrypted-param")
        .or_else(|| headers.get("X-Encrypted-Param"))
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}
