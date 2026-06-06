pub const ENABLE_CDN_URL_FALLBACK: bool = true;

pub fn build_cdn_download_url(encrypted_query_param: &str, cdn_base_url: &str) -> String {
    format!(
        "{}/download?encrypted_query_param={}",
        cdn_base_url.trim_end_matches('/'),
        urlencoding(encrypted_query_param)
    )
}

pub fn build_cdn_upload_url(cdn_base_url: &str, upload_param: &str, filekey: &str) -> String {
    format!(
        "{}/upload?encrypted_query_param={}&filekey={}",
        cdn_base_url.trim_end_matches('/'),
        urlencoding(upload_param),
        urlencoding(filekey)
    )
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
