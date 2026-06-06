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
        user_id: None,
    });

    bot.start(StartOptions {
        long_poll_timeout_ms: None,
        on_message: handler(|ctx| async move {
            let msg_type = ctx.message_type.as_str();
            println!(
                "[{}] 来自 {}: type={}, body={}",
                ctx.account_id, ctx.from, msg_type, ctx.body
            );

            if let Some(path) = &ctx.media_path {
                println!("媒体文件已下载到: {path}");
            }
            if let Some(media_type) = &ctx.media_type {
                println!("媒体 MIME: {media_type}");
            }

            let reply = match msg_type {
                "text" => format!("你发了一条文本消息: {}", ctx.body),
                "image" => "收到图片消息".to_string(),
                "video" => "收到视频消息".to_string(),
                "voice" => {
                    if ctx.body.is_empty() {
                        "收到语音消息".to_string()
                    } else {
                        format!("收到语音消息，识别文本: {}", ctx.body)
                    }
                }
                "file" => "收到文件消息".to_string(),
                _ => format!("收到 {} 消息", msg_type),
            };

            Ok(Some(reply))
        }),
    })
    .await
}
