use std::path::PathBuf;

use wx_bot_sdk::{WeixinBot, WeixinBotOptions};

#[tokio::main]
async fn main() -> wx_bot_sdk::Result<()> {
    let to = read_to_user()?;
    let deck_path = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("deck.pptx"));

    if !deck_path.is_file() {
        return Err(format!("文件不存在: {}", deck_path.display()).into());
    }

    let bot = create_bot().await?;
    println!(
        "使用账号 {} 发送 {} 到 {}",
        bot.account_id(),
        deck_path.display(),
        to
    );

    let result = bot
        .send_file(&to, &deck_path, Some("请查收 deck.pptx"))
        .await?;

    println!("发送成功，message_id: {}", result.message_id);
    Ok(())
}

async fn create_bot() -> wx_bot_sdk::Result<WeixinBot> {
    let base_url = std::env::var("WEIXIN_API_BASE_URL").ok();

    if let Ok(account_id) = std::env::var("WEIXIN_ACCOUNT_ID") {
        match WeixinBot::from_account(&account_id) {
            Ok(bot) => return Ok(bot),
            Err(err) => {
                eprintln!(
                    "WEIXIN_ACCOUNT_ID={account_id} 未找到可用登录凭据: {err}\n\
                     将继续尝试 WEIXIN_BOT_TOKEN；如果也未设置，则改用扫码登录。\n"
                );
            }
        }
    }

    if let Ok(token) = std::env::var("WEIXIN_BOT_TOKEN") {
        return Ok(WeixinBot::new(WeixinBotOptions {
            token,
            base_url,
            cdn_base_url: None,
            state_dir: None,
            account_id: None,
            user_id: None,
        }));
    }

    println!("未设置可用的 WEIXIN_ACCOUNT_ID 或 WEIXIN_BOT_TOKEN，将使用扫码登录。\n");
    WeixinBot::login_interactive(base_url.as_deref()).await
}

fn read_to_user() -> wx_bot_sdk::Result<String> {
    std::env::var("WEIXIN_TO_USER")
        .ok()
        .or_else(|| std::env::args().nth(1))
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .ok_or_else(|| {
            "Usage:\n  cargo run --example send_deck -- <to_user_id> [deck_path]\n\n或者设置环境变量：\n  WEIXIN_TO_USER=<to_user_id> cargo run --example send_deck".into()
        })
}
