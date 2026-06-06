use wx_bot_sdk::{StartOptions, WeixinBot, WeixinBotOptions, bot::handler};

#[tokio::main]
async fn main() -> wx_bot_sdk::Result<()> {
    let token = std::env::var("WEIXIN_BOT_TOKEN").or_else(|_| {
        std::env::args()
            .nth(1)
            .ok_or(std::env::VarError::NotPresent)
    })?;
    let bot = WeixinBot::new(WeixinBotOptions {
        token,
        base_url: None,
        cdn_base_url: None,
        state_dir: None,
        account_id: None,
    });

    bot.start(StartOptions {
        long_poll_timeout_ms: None,
        on_message: handler(|ctx| async move {
            println!("来自 {}: {}", ctx.from, ctx.body);
            Ok(Some(format!("你说了: {}", ctx.body)))
        }),
    })
    .await
}
