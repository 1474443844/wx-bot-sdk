use wx_bot_sdk::{StartOptions, WeixinBot, bot::handler};

#[tokio::main]
async fn main() -> wx_bot_sdk::Result<()> {
    let api_base_url = std::env::var("WEIXIN_API_BASE_URL").ok();

    println!("正在获取微信扫码登录二维码...");
    let bot = WeixinBot::login_interactive(api_base_url.as_deref()).await?;
    println!("登录成功");
    println!(
        "Token: {}\nAccount ID: {}\nBase URL: {}\nUser ID: {}",
        bot.token(),
        bot.account_id(),
        bot.base_url(),
        bot.user_id().unwrap_or("")
    );
    println!("启动 echo bot，按 Ctrl+C 退出。\n");

    bot.start(StartOptions {
        long_poll_timeout_ms: None,
        on_message: handler(|ctx| async move {
            println!("来自 {}: {}", ctx.from, ctx.body);
            Ok(Some(format!("你说了: {}", ctx.body)))
        }),
    })
    .await
}
