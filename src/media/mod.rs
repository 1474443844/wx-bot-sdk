pub mod media_download;

pub use media_download::*;

use std::path::Path;

pub fn get_mime_from_filename(filename: impl AsRef<Path>) -> String {
    let ext = filename
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_ascii_lowercase()))
        .unwrap_or_default();
    match ext.as_str() {
        ".jpg" | ".jpeg" => "image/jpeg",
        ".png" => "image/png",
        ".gif" => "image/gif",
        ".bmp" => "image/bmp",
        ".webp" => "image/webp",
        ".svg" => "image/svg+xml",
        ".ico" => "image/x-icon",
        ".tiff" | ".tif" => "image/tiff",
        ".heic" => "image/heic",
        ".heif" => "image/heif",
        ".avif" => "image/avif",
        ".mp3" => "audio/mpeg",
        ".wav" => "audio/wav",
        ".ogg" => "audio/ogg",
        ".flac" => "audio/flac",
        ".aac" => "audio/aac",
        ".m4a" => "audio/mp4",
        ".amr" => "audio/amr",
        ".silk" => "audio/silk",
        ".mp4" | ".m4v" => "video/mp4",
        ".avi" => "video/x-msvideo",
        ".mov" => "video/quicktime",
        ".wmv" => "video/x-ms-wmv",
        ".flv" => "video/x-flv",
        ".mkv" => "video/x-matroska",
        ".webm" => "video/webm",
        ".3gp" => "video/3gpp",
        ".pdf" => "application/pdf",
        ".doc" => "application/msword",
        ".docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ".xls" => "application/vnd.ms-excel",
        ".xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ".ppt" => "application/vnd.ms-powerpoint",
        ".pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ".txt" => "text/plain",
        ".csv" => "text/csv",
        ".json" => "application/json",
        ".xml" => "application/xml",
        ".html" | ".htm" => "text/html",
        ".css" => "text/css",
        ".js" => "application/javascript",
        ".ts" => "text/typescript",
        ".zip" => "application/zip",
        ".rar" => "application/vnd.rar",
        ".7z" => "application/x-7z-compressed",
        ".tar" => "application/x-tar",
        ".gz" => "application/gzip",
        _ => "application/octet-stream",
    }
    .to_string()
}

pub fn get_extension_from_mime(mime_type: &str) -> String {
    match mime_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "image/jpeg" => ".jpg",
        "image/png" => ".png",
        "image/gif" => ".gif",
        "image/webp" => ".webp",
        "audio/mpeg" => ".mp3",
        "audio/wav" => ".wav",
        "audio/silk" => ".silk",
        "video/mp4" => ".mp4",
        "application/pdf" => ".pdf",
        "text/plain" => ".txt",
        "text/csv" => ".csv",
        "application/json" => ".json",
        "text/html" => ".html",
        _ => ".bin",
    }
    .to_string()
}

pub fn get_extension_from_content_type_or_url(
    content_type: Option<&str>,
    url: Option<&str>,
) -> String {
    if let Some(ct) = content_type {
        let ext = get_extension_from_mime(ct);
        if ext != ".bin" {
            return ext;
        }
    }
    if let Some(url) = url
        && let Ok(parsed) = url::Url::parse(url)
        && let Some(seg) = parsed.path_segments().and_then(|mut s| s.next_back())
        && let Some(dot) = seg.rfind('.')
    {
        let ext = &seg[dot..];
        if ext.len() <= 6 {
            return ext.to_string();
        }
    }
    ".bin".to_string()
}
