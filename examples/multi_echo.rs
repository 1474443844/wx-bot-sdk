use wx_bot_sdk::{
    BotAccountOptions, MultiStartOptions, MultiWeixinBot, MultiWeixinBotOptions, bot::handler,
};

#[tokio::main]
async fn main() -> wx_bot_sdk::Result<()> {
    let tokens = read_tokens();
    if tokens.is_empty() {
        eprintln!("Usage:");
        eprintln!("  WEIXIN_BOT_TOKENS=\"token1,token2\" cargo run --example multi_echo");
        eprintln!("  cargo run --example multi_echo -- token1 token2");
        std::process::exit(1);
    }

    let multi = MultiWeixinBot::new(MultiWeixinBotOptions {
        accounts: tokens
            .into_iter()
            .map(|token| BotAccountOptions {
                token,
                account_id: None,
                base_url: None,
                cdn_base_url: None,
            })
            .collect(),
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
