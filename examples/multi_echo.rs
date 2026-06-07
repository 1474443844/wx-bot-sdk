use wx_bot_sdk::{
    BotAccountOptions, MultiStartOptions, MultiWeixinBot, MultiWeixinBotOptions,
    auth::accounts::list_weixin_account_ids, bot::handler, resolve_weixin_account,
};

#[tokio::main]
async fn main() -> wx_bot_sdk::Result<()> {
    let accounts = read_accounts();
    if accounts.is_empty() {
        eprintln!("Usage:");
        eprintln!("  WEIXIN_BOT_TOKENS=\"token1,token2\" cargo run --example multi_echo");
        eprintln!("  cargo run --example multi_echo -- token1 token2");
        eprintln!("\n如果已通过扫码登录保存账号，也可以直接运行：");
        eprintln!("  cargo run --example multi_echo");
        std::process::exit(1);
    }

    let multi = MultiWeixinBot::new(MultiWeixinBotOptions {
        accounts,
        state_dir: None,
    });

    println!("starting accounts: {:?}", multi.account_ids());
    multi
        .start(MultiStartOptions {
            long_poll_timeout_ms: None,
            on_message: handler(|ctx| async move {
                println!("[{}] 来自 {}: {}", ctx.account_id, ctx.from, ctx.body);
                Ok(Some(format!("你说了: {}", ctx.body)))
            }),
        })
        .await?;

    tokio::signal::ctrl_c().await?;
    println!("stopping all accounts...");
    multi.stop().await?;
    multi.join().await?;
    Ok(())
}

fn read_accounts() -> Vec<BotAccountOptions> {
    let tokens = read_tokens();
    if !tokens.is_empty() {
        return tokens
            .into_iter()
            .map(|token| BotAccountOptions {
                token,
                account_id: None,
                base_url: None,
                cdn_base_url: None,
            })
            .collect();
    }

    list_weixin_account_ids()
        .into_iter()
        .filter_map(|account_id| match resolve_weixin_account(&account_id) {
            Ok(account) => account.token.map(|token| BotAccountOptions {
                token,
                account_id: Some(account.account_id),
                base_url: Some(account.base_url),
                cdn_base_url: Some(account.cdn_base_url),
            }),
            Err(err) => {
                eprintln!("跳过账号 {account_id}: {err}");
                None
            }
        })
        .collect()
}

fn read_tokens() -> Vec<String> {
    let mut tokens = std::env::args()
        .skip(1)
        .flat_map(|arg| split_tokens(&arg).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    if tokens.is_empty()
        && let Ok(raw) = std::env::var("WEIXIN_BOT_TOKENS")
    {
        tokens = split_tokens(&raw).collect();
    }

    tokens
}

fn split_tokens(raw: &str) -> impl Iterator<Item = String> + '_ {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}
