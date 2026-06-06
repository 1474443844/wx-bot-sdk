use std::sync::{Arc, Mutex};

use tokio::task::JoinHandle;

use crate::{
    auth::accounts::{CDN_BASE_URL, DEFAULT_BASE_URL},
    bot::{StartOptions, WeixinBot, WeixinBotOptions},
    messaging::process_message::MessageHandler,
};

#[derive(Clone, Debug)]
pub struct BotAccountOptions {
    pub token: String,
    pub account_id: Option<String>,
    pub base_url: Option<String>,
    pub cdn_base_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MultiWeixinBotOptions {
    pub accounts: Vec<BotAccountOptions>,
    pub state_dir: Option<String>,
}

pub struct MultiStartOptions {
    pub on_message: MessageHandler,
    pub long_poll_timeout_ms: Option<u64>,
}

#[derive(Clone)]
pub struct MultiWeixinBot {
    bots: Vec<WeixinBot>,
    handles: Arc<Mutex<Vec<JoinHandle<crate::Result<()>>>>>,
}

impl MultiWeixinBot {
    pub fn new(opts: MultiWeixinBotOptions) -> Self {
        let bots = opts
            .accounts
            .into_iter()
            .map(|account| {
                WeixinBot::new(WeixinBotOptions {
                    token: account.token,
                    base_url: account
                        .base_url
                        .or_else(|| Some(DEFAULT_BASE_URL.to_string())),
                    cdn_base_url: account
                        .cdn_base_url
                        .or_else(|| Some(CDN_BASE_URL.to_string())),
                    state_dir: opts.state_dir.clone(),
                    account_id: account.account_id,
                    user_id: None,
                })
            })
            .collect();
        Self {
            bots,
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start(&self, opts: MultiStartOptions) -> crate::Result<()> {
        let mut handles = self.handles.lock().expect("multi bot handles poisoned");
        if !handles.is_empty() {
            return Ok(());
        }

        for bot in &self.bots {
            let bot = bot.clone();
            let on_message = opts.on_message.clone();
            let long_poll_timeout_ms = opts.long_poll_timeout_ms;
            handles.push(tokio::spawn(async move {
                bot.start(StartOptions {
                    on_message,
                    long_poll_timeout_ms,
                })
                .await
            }));
        }
        Ok(())
    }

    pub async fn stop(&self) -> crate::Result<()> {
        for bot in &self.bots {
            bot.stop().await?;
        }
        Ok(())
    }

    pub async fn join(&self) -> crate::Result<()> {
        let handles = {
            let mut locked = self.handles.lock().expect("multi bot handles poisoned");
            locked.drain(..).collect::<Vec<_>>()
        };

        for handle in handles {
            handle.await??;
        }
        Ok(())
    }

    pub fn bots(&self) -> &[WeixinBot] {
        &self.bots
    }

    pub fn account_ids(&self) -> Vec<String> {
        self.bots
            .iter()
            .map(|bot| bot.account_id().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_one_bot_per_token() {
        let bot = MultiWeixinBot::new(MultiWeixinBotOptions {
            accounts: vec![
                BotAccountOptions {
                    token: "token-a".into(),
                    account_id: None,
                    base_url: None,
                    cdn_base_url: None,
                },
                BotAccountOptions {
                    token: "token-b".into(),
                    account_id: None,
                    base_url: None,
                    cdn_base_url: None,
                },
            ],
            state_dir: None,
        });
        assert_eq!(bot.bots().len(), 2);
        let ids = bot.account_ids();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn keeps_explicit_account_ids() {
        let bot = MultiWeixinBot::new(MultiWeixinBotOptions {
            accounts: vec![BotAccountOptions {
                token: "token-a".into(),
                account_id: Some("account-a".into()),
                base_url: None,
                cdn_base_url: None,
            }],
            state_dir: None,
        });
        assert_eq!(bot.account_ids(), vec!["account-a".to_string()]);
    }
}
