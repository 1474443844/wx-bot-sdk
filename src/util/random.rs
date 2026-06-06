use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::RngCore;
use uuid::Uuid;

pub fn generate_id(prefix: &str) -> String {
    let millis = chrono::Utc::now().timestamp_millis();
    let short = Uuid::new_v4().simple().to_string();
    format!("{prefix}-{millis}-{}", &short[..12])
}

pub fn temp_file_name(prefix: &str, ext: &str) -> String {
    let ext = if ext.starts_with('.') {
        ext.to_string()
    } else {
        format!(".{ext}")
    };
    format!("{}{}", generate_id(prefix), ext)
}

pub fn random_uint32_base64() -> String {
    let mut bytes = [0_u8; 4];
    rand::thread_rng().fill_bytes(&mut bytes);
    STANDARD.encode(bytes)
}
