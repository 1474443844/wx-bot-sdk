use base64::{Engine as _, engine::general_purpose::STANDARD};

use super::{aes_ecb::decrypt_aes_ecb, cdn_url::build_cdn_download_url};

pub async fn download_plain_cdn_buffer(
    encrypted_query_param: &str,
    cdn_base_url: &str,
    label: &str,
    full_url: Option<&str>,
) -> crate::Result<Vec<u8>> {
    let url = full_url
        .filter(|s| !s.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| build_cdn_download_url(encrypted_query_param, cdn_base_url));
    let res = reqwest::Client::new().get(url).send().await?;
    let status = res.status();
    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = res.bytes().await?;
    if !status.is_success() {
        return Err(format!(
            "{label} CDN download HTTP {status}: {}",
            String::from_utf8_lossy(&bytes)
        )
        .into());
    }
    if content_type.contains("text/html") {
        return Err(format!("{label} CDN returned html instead of media").into());
    }
    Ok(bytes.to_vec())
}

pub async fn download_and_decrypt_buffer(
    encrypted_query_param: &str,
    aes_key_base64: &str,
    cdn_base_url: &str,
    label: &str,
    full_url: Option<&str>,
) -> crate::Result<Vec<u8>> {
    let encrypted =
        download_plain_cdn_buffer(encrypted_query_param, cdn_base_url, label, full_url).await?;
    let key = STANDARD.decode(aes_key_base64)?;
    decrypt_aes_ecb(&encrypted, &key)
}
