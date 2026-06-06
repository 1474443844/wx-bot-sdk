use std::{
    future::Future,
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex},
};

use sha2::{Digest, Sha256};
use tokio::sync::watch;

use crate::{
    api::{WeixinApiOptions, notify_start, notify_stop},
    auth::{
        accounts::{CDN_BASE_URL, DEFAULT_BASE_URL, resolve_weixin_account},
        login_qr::{display_qr_code, start_weixin_login_with_qr, wait_for_weixin_login},
    },
    messaging::{
        process_message::MessageHandler,
        send::{
            SendResult, WeixinMsgContext, get_context_token, restore_context_tokens,
            send_message_weixin,
        },
        send_media::{send_media_url, send_weixin_media_file},
    },
    monitor::{MonitorWeixinOpts, monitor_weixin_provider},
};

#[derive(Clone, Debug)]
pub struct WeixinBotOptions {
    pub token: String,
    pub base_url: Option<String>,
    pub cdn_base_url: Option<String>,
    pub state_dir: Option<String>,
    pub account_id: Option<String>,
}

pub struct StartOptions {
    pub on_message: MessageHandler,
    pub long_poll_timeout_ms: Option<u64>,
}

#[derive(Clone)]
pub struct WeixinBot {
    token: String,
    base_url: String,
    cdn_base_url: String,
    account_id: String,
    stop_tx: Arc<Mutex<Option<watch::Sender<bool>>>>,
}

impl WeixinBot {
    pub fn new(opts: WeixinBotOptions) -> Self {
        let _state_dir = opts.state_dir;
        let account_id = opts
            .account_id
            .unwrap_or_else(|| derive_account_id(&opts.token));
        restore_context_tokens(&account_id);
        Self {
            token: opts.token,
            base_url: opts.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.into()),
            cdn_base_url: opts.cdn_base_url.unwrap_or_else(|| CDN_BASE_URL.into()),
            account_id,
            stop_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_account(account_id: &str) -> crate::Result<Self> {
        let account = resolve_weixin_account(account_id)?;
        let token = account.token.ok_or("account is not configured")?;
        Ok(Self::new(WeixinBotOptions {
            token,
            base_url: Some(account.base_url),
            cdn_base_url: Some(account.cdn_base_url),
            state_dir: None,
            account_id: Some(account.account_id),
        }))
    }

    pub async fn login_interactive(api_base_url: Option<&str>) -> crate::Result<Self> {
        let api_base_url = api_base_url.unwrap_or(DEFAULT_BASE_URL);
        let start = start_weixin_login_with_qr(api_base_url, None, None, false).await?;
        if let Some(url) = &start.qrcode_url {
            display_qr_code(url)?;
        }
        let waited = wait_for_weixin_login(&start.session_key, api_base_url, None, None).await?;
        if !waited.connected {
            return Err(waited.message.into());
        }
        Ok(Self::new(WeixinBotOptions {
            token: waited.bot_token.ok_or("login returned no token")?,
            base_url: waited.base_url,
            cdn_base_url: Some(CDN_BASE_URL.into()),
            state_dir: None,
            account_id: waited.account_id,
        }))
    }

    pub async fn start(&self, opts: StartOptions) -> crate::Result<()> {
        if self.is_running() {
            return Ok(());
        }
        let (tx, rx) = watch::channel(false);
        *self.stop_tx.lock().unwrap() = Some(tx);
        let api_opts = self.api_opts();
        let _ = notify_start(&api_opts).await;
        monitor_weixin_provider(
            MonitorWeixinOpts {
                base_url: self.base_url.clone(),
                cdn_base_url: self.cdn_base_url.clone(),
                token: Some(self.token.clone()),
                account_id: self.account_id.clone(),
                long_poll_timeout_ms: opts.long_poll_timeout_ms,
                on_message: opts.on_message,
            },
            rx,
        )
        .await
    }

    pub async fn stop(&self) -> crate::Result<()> {
        if let Some(tx) = self.stop_tx.lock().unwrap().take() {
            let _ = tx.send(true);
        }
        let _ = notify_stop(&self.api_opts()).await;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.stop_tx.lock().unwrap().is_some()
    }
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    fn api_opts(&self) -> WeixinApiOptions {
        WeixinApiOptions::new(self.base_url.clone(), Some(self.token.clone()))
    }
    fn context_token(&self, to: &str) -> Option<String> {
        get_context_token(&self.account_id, to)
    }

    pub async fn send_text(&self, to: &str, text: &str) -> crate::Result<SendResult> {
        send_message_weixin(
            to,
            text,
            &self.api_opts(),
            self.context_token(to).as_deref(),
        )
        .await
    }
    pub async fn send_image(
        &self,
        to: &str,
        file_path: impl AsRef<Path>,
        caption: Option<&str>,
    ) -> crate::Result<SendResult> {
        send_weixin_media_file(
            file_path,
            to,
            caption.unwrap_or(""),
            &self.api_opts(),
            &self.cdn_base_url,
            self.context_token(to).as_deref(),
        )
        .await
    }
    pub async fn send_video(
        &self,
        to: &str,
        file_path: impl AsRef<Path>,
        caption: Option<&str>,
    ) -> crate::Result<SendResult> {
        send_weixin_media_file(
            file_path,
            to,
            caption.unwrap_or(""),
            &self.api_opts(),
            &self.cdn_base_url,
            self.context_token(to).as_deref(),
        )
        .await
    }
    pub async fn send_file(
        &self,
        to: &str,
        file_path: impl AsRef<Path>,
        caption: Option<&str>,
    ) -> crate::Result<SendResult> {
        send_weixin_media_file(
            file_path,
            to,
            caption.unwrap_or(""),
            &self.api_opts(),
            &self.cdn_base_url,
            self.context_token(to).as_deref(),
        )
        .await
    }
    pub async fn send_media_url(
        &self,
        to: &str,
        url: &str,
        caption: Option<&str>,
    ) -> crate::Result<SendResult> {
        send_media_url(
            url,
            to,
            caption.unwrap_or(""),
            &self.api_opts(),
            &self.cdn_base_url,
            self.context_token(to).as_deref(),
        )
        .await
    }
}

pub fn handler<F, Fut>(f: F) -> MessageHandler
where
    F: Fn(WeixinMsgContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = crate::Result<Option<String>>> + Send + 'static,
{
    Arc::new(move |ctx| {
        Box::pin(f(ctx)) as Pin<Box<dyn Future<Output = crate::Result<Option<String>>> + Send>>
    })
}

fn derive_account_id(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!("bot-{}", hex::encode(&digest[..8]))
}
