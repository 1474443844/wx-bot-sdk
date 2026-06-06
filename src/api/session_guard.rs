use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

pub const SESSION_EXPIRED_ERRCODE: i32 = -10001;
const SESSION_PAUSE_MS: u64 = 10 * 60 * 1000;

static PAUSED: Lazy<Mutex<HashMap<String, Instant>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn pause_session(account_id: &str) {
    if let Ok(mut m) = PAUSED.lock() {
        m.insert(
            account_id.to_string(),
            Instant::now() + Duration::from_millis(SESSION_PAUSE_MS),
        );
    }
}

pub fn get_remaining_pause_ms(account_id: &str) -> u64 {
    let Ok(mut m) = PAUSED.lock() else {
        return 0;
    };
    let Some(deadline) = m.get(account_id).copied() else {
        return 0;
    };
    let now = Instant::now();
    if deadline <= now {
        m.remove(account_id);
        0
    } else {
        (deadline - now).as_millis() as u64
    }
}
