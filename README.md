# wx-bot-sdk

独立的微信 Bot Rust SDK，从 [@tencent-weixin/openclaw-weixin](https://github.com/Tencent/openclaw-weixin) 提取而来。

该 SDK 提供：

- 微信 Bot 扫码登录
- 单账号 / 多账号消息监听
- 文本、图片、视频、文件发送
- 媒体消息下载到本地临时文件
- 账号凭据、`getUpdates` 同步游标、context token 本地持久化

> 当前项目仍处于早期版本，接口可能继续调整。

## 安装

在你的 Rust 项目中添加依赖：

```toml
[dependencies]
wx-bot-sdk = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal"] }
```

本仓库内运行示例：

```bash
cargo run --example echo -- <WEIXIN_BOT_TOKEN>
```

## 账号与状态目录

SDK 默认把运行状态保存在当前目录的 `.weixin-bot/` 下：

```text
.weixin-bot/
  accounts.json
  accounts/
    <account-id>.json
    <account-id>.sync.json
    <account-id>.context-tokens.json
```

账号 ID 会按 OpenClaw 兼容格式归一化保存。例如：

```text
e93e6ce56d3c@im.bot
```

会保存为：

```text
e93e6ce56d3c-im-bot.json
e93e6ce56d3c-im-bot.sync.json
e93e6ce56d3c-im-bot.context-tokens.json
```

可通过环境变量修改状态目录：

```bash
WEIXIN_BOT_STATE_DIR=/path/to/state cargo run --example qr_login
```

PowerShell：

```powershell
$env:WEIXIN_BOT_STATE_DIR=".weixin-bot-dev"
cargo run --example qr_login
```

## 扫码登录

运行扫码登录示例：

```bash
cargo run --example qr_login
```

PowerShell：

```powershell
cargo run --example qr_login
```

终端会显示二维码。扫码并确认后会输出：

```text
Token: ...
Account ID: ...
Base URL: ...
User ID: ...
```

登录成功后，账号 token、base URL、user ID 会保存到 `.weixin-bot/accounts/`。后续可用 `WEIXIN_ACCOUNT_ID` 复用本地登录凭据。

### 为什么第二次扫码会直接进入聊天界面？

扫码登录请求会携带本地最近保存的 bot token 列表。服务端识别到该微信账号已经绑定过当前 Bot 时，手机端可能会直接跳转到聊天界面，而不是再次提示绑定。这与原始 `openclaw-weixin` 行为一致。

如需重新测试首次绑定流程，可以使用新的状态目录：

```powershell
$env:WEIXIN_BOT_STATE_DIR=".weixin-bot-fresh"
cargo run --example qr_login
```

## Echo Bot

`examples/echo.rs` 展示了最基础的消息监听与自动回复。

### 使用 token 启动

```bash
WEIXIN_BOT_TOKEN="your-token" cargo run --example echo
```

或：

```bash
cargo run --example echo -- your-token
```

PowerShell：

```powershell
$env:WEIXIN_BOT_TOKEN="your-token"
cargo run --example echo
```

### 判断消息类型

`WeixinMsgContext` 中包含：

| 字段 | 说明 |
| --- | --- |
| `ctx.message_type` | 消息类型：`text` / `image` / `video` / `voice` / `file` / `unknown` |
| `ctx.body` | 文本内容；语音消息可能包含识别文本 |
| `ctx.from` | 发送者 user id |
| `ctx.account_id` | 当前 Bot 账号 ID |
| `ctx.media_path` | 媒体消息下载后的本地路径 |
| `ctx.media_type` | 媒体 MIME 类型，例如 `audio/silk`、`image/*` |
| `ctx.context_token` | 会话 context token，SDK 发送回复时会自动使用 |

注意：语音消息可能带有识别文本，但 `ctx.message_type` 仍会是 `voice`，不要仅通过 `ctx.body` 是否为空判断消息类型。

示例处理逻辑：

```rust
on_message: handler(|ctx| async move {
    match ctx.message_type.as_str() {
        "text" => Ok(Some(format!("你说了: {}", ctx.body))),
        "voice" => Ok(Some(format!("收到语音，识别文本: {}", ctx.body))),
        "image" => Ok(Some("收到图片".into())),
        "video" => Ok(Some("收到视频".into())),
        "file" => Ok(Some("收到文件".into())),
        _ => Ok(Some("收到消息".into())),
    }
})
```

## 发送消息

### 发送文本

```rust
use wx_bot_sdk::{WeixinBot, WeixinBotOptions};

#[tokio::main]
async fn main() -> wx_bot_sdk::Result<()> {
    let bot = WeixinBot::new(WeixinBotOptions {
        token: std::env::var("WEIXIN_BOT_TOKEN")?,
        base_url: None,
        cdn_base_url: None,
        state_dir: None,
        account_id: None,
        user_id: None,
    });

    bot.send_text("user@im.bot", "hello").await?;
    Ok(())
}
```

### 发送文件 / deck.pptx

仓库内提供了 `examples/send_deck.rs`，默认发送根目录的 `deck.pptx`：

```bash
cargo run --example send_deck -- <to_user_id>
```

指定文件路径：

```bash
cargo run --example send_deck -- <to_user_id> /path/to/deck.pptx
```

PowerShell 示例：

```powershell
cargo run --example send_deck -- 6263701457de@im.bot
```

`send_deck` 会按以下顺序创建 Bot：

1. 如果设置了 `WEIXIN_ACCOUNT_ID`，使用本地已保存账号；
2. 如果设置了 `WEIXIN_BOT_TOKEN`，使用 token；
3. 否则进入扫码登录。

复用已扫码账号：

```powershell
$env:WEIXIN_ACCOUNT_ID="e93e6ce56d3c@im.bot"
cargo run --example send_deck -- 6263701457de@im.bot
```

使用 token：

```powershell
$env:WEIXIN_BOT_TOKEN="your-token"
cargo run --example send_deck -- 6263701457de@im.bot
```

## 多账号 Echo

`examples/multi_echo.rs` 支持同时启动多个账号：

```bash
WEIXIN_BOT_TOKENS="token1,token2" cargo run --example multi_echo
```

或：

```bash
cargo run --example multi_echo -- token1 token2
```

收到消息时会打印对应 `ctx.account_id`。

## 常用环境变量

| 环境变量 | 说明 |
| --- | --- |
| `WEIXIN_BOT_TOKEN` | Bot token，适用于单账号示例 |
| `WEIXIN_BOT_TOKENS` | 多账号 token 列表，用逗号分隔 |
| `WEIXIN_ACCOUNT_ID` | 使用扫码登录后保存的本地账号 |
| `WEIXIN_TO_USER` | `send_deck` 的默认收件人 |
| `WEIXIN_API_BASE_URL` | 自定义 API base URL，默认 `https://ilinkai.weixin.qq.com` |
| `WEIXIN_BOT_STATE_DIR` | 自定义状态目录，默认 `.weixin-bot` |

## 主要 API

### `WeixinBot`

创建方式：

```rust
let bot = WeixinBot::new(WeixinBotOptions { /* ... */ });
let bot = WeixinBot::from_account("e93e6ce56d3c@im.bot")?;
let bot = WeixinBot::login_interactive(None).await?;
```

常用方法：

| 方法 | 说明 |
| --- | --- |
| `WeixinBot::new(opts)` | 使用 token 创建 Bot |
| `WeixinBot::from_account(account_id)` | 从本地保存账号创建 Bot |
| `WeixinBot::login_interactive(api_base_url)` | 终端扫码登录 |
| `bot.start(StartOptions)` | 启动消息监听 |
| `bot.stop()` | 停止监听 |
| `bot.send_text(to, text)` | 发送文本 |
| `bot.send_image(to, path, caption)` | 发送图片 |
| `bot.send_video(to, path, caption)` | 发送视频 |
| `bot.send_file(to, path, caption)` | 发送文件附件 |
| `bot.send_media_url(to, url, caption)` | 下载远程媒体并发送 |
| `bot.account_id()` | 当前 Bot 账号 ID |
| `bot.user_id()` | 扫码用户 ID，可能为空 |
| `bot.token()` | 当前 token |

### `handler`

`handler` 用于把 async 闭包转换为消息处理器：

```rust
use wx_bot_sdk::{StartOptions, bot::handler};

bot.start(StartOptions {
    long_poll_timeout_ms: None,
    on_message: handler(|ctx| async move {
        println!("from={} type={} body={}", ctx.from, ctx.message_type, ctx.body);
        Ok(Some("收到".to_string()))
    }),
}).await?;
```

返回值说明：

- `Ok(Some(text))`：自动回复文本
- `Ok(None)`：不回复
- `Err(err)`：处理失败，错误会向上传递

## 媒体文件说明

收到图片、视频、语音、文件时，SDK 会尝试下载并解密到系统临时目录：

```rust
if let Some(path) = ctx.media_path.as_deref() {
    println!("media saved to {path}");
}
```

发送本地媒体时，SDK 会根据扩展名判断类型：

- `image/*` → 图片消息
- `video/*` → 视频消息
- 其他 → 文件附件

例如 `.pptx` 会作为文件附件发送。

## 开发与验证

格式化：

```bash
cargo fmt
```

检查所有示例：

```bash
cargo check --examples
```

运行测试：

```bash
cargo test
```

## License

MIT
