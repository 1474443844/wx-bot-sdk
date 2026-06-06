use std::{env, path::PathBuf};

pub fn resolve_state_dir() -> PathBuf {
    env::var_os("WEIXIN_BOT_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".weixin-bot"))
}
