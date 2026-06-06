use once_cell::sync::Lazy;
use qrcode::{QrCode, render::unicode};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{self, Write},
    sync::Mutex,
    time::{Duration, Instant},
};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{
    api::{api_get_fetch, api_post_fetch},
    auth::accounts::{
        DEFAULT_BASE_URL, clear_stale_accounts_for_user_id, list_weixin_account_ids,
        load_weixin_account, save_weixin_account,
    },
};

const ACTIVE_LOGIN_TTL: Duration = Duration::from_secs(5 * 60);
const QR_LONG_POLL_TIMEOUT_MS: u64 = 35_000;
const DEFAULT_ILINK_BOT_TYPE: &str = "3";
const MAX_QR_REFRESH_COUNT: u32 = 3;

#[derive(Clone, Debug)]
struct ActiveLogin {
    qrcode: String,
    qrcode_url: String,
    started_at: Instant,
    current_api_base_url: Option<String>,
    pending_verify_code: Option<String>,
}

static ACTIVE_LOGINS: Lazy<Mutex<HashMap<String, ActiveLogin>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug, Deserialize)]
struct QRCodeResponse {
    qrcode: String,
    qrcode_img_content: String,
}

#[derive(Clone, Debug, Deserialize)]
struct StatusResponse {
    status: String,
    bot_token: Option<String>,
    ilink_bot_id: Option<String>,
    baseurl: Option<String>,
    ilink_user_id: Option<String>,
    redirect_host: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeixinQrStartResult {
    pub qrcode_url: Option<String>,
    pub message: String,
    pub session_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeixinQrWaitResult {
    pub connected: bool,
    pub already_connected: Option<bool>,
    pub bot_token: Option<String>,
    pub account_id: Option<String>,
    pub base_url: Option<String>,
    pub user_id: Option<String>,
    pub message: String,
}

fn fresh(login: &ActiveLogin) -> bool {
    login.started_at.elapsed() < ACTIVE_LOGIN_TTL
}

fn local_bot_token_list() -> Vec<String> {
    list_weixin_account_ids()
        .into_iter()
        .rev()
        .filter_map(|id| load_weixin_account(&id)?.token)
        .filter(|t| !t.trim().is_empty())
        .take(10)
        .collect()
}

async fn fetch_qr_code(api_base_url: &str, bot_type: &str) -> crate::Result<QRCodeResponse> {
    let body = serde_json::json!({"local_token_list": local_bot_token_list()}).to_string();
    let endpoint = format!(
        "ilink/bot/get_bot_qrcode?bot_type={}",
        url::form_urlencoded::byte_serialize(bot_type.as_bytes()).collect::<String>()
    );
    let raw = api_post_fetch(api_base_url, &endpoint, body, None, None, "fetchQRCode").await?;
    Ok(serde_json::from_str(&raw)?)
}

async fn poll_qr_status(
    api_base_url: &str,
    qrcode: &str,
    verify_code: Option<&str>,
) -> StatusResponse {
    let mut endpoint = format!(
        "ilink/bot/get_qrcode_status?qrcode={}",
        url::form_urlencoded::byte_serialize(qrcode.as_bytes()).collect::<String>()
    );
    if let Some(code) = verify_code {
        endpoint.push_str("&verify_code=");
        endpoint
            .push_str(&url::form_urlencoded::byte_serialize(code.as_bytes()).collect::<String>());
    }
    match api_get_fetch(
        api_base_url,
        &endpoint,
        Some(QR_LONG_POLL_TIMEOUT_MS),
        "pollQRStatus",
    )
    .await
    .and_then(|raw| Ok(serde_json::from_str(&raw)?))
    {
        Ok(resp) => resp,
        Err(_) => StatusResponse {
            status: "wait".into(),
            bot_token: None,
            ilink_bot_id: None,
            baseurl: None,
            ilink_user_id: None,
            redirect_host: None,
        },
    }
}

fn read_verify_code_from_stdin(prompt: &str) -> crate::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn display_qr_code(qrcode_url: &str) -> crate::Result<()> {
    if let Ok(code) = QrCode::new(qrcode_url.as_bytes()) {
        let image = code.render::<unicode::Dense1x2>().quiet_zone(false).build();
        println!("{image}");
    }
    println!("若二维码未能显示，访问以下链接：\n{qrcode_url}");
    Ok(())
}

pub async fn start_weixin_login_with_qr(
    api_base_url: &str,
    account_id: Option<&str>,
    bot_type: Option<&str>,
    force: bool,
) -> crate::Result<WeixinQrStartResult> {
    let session_key = account_id
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    if let Ok(mut m) = ACTIVE_LOGINS.lock() {
        m.retain(|_, v| fresh(v));
        if !force && let Some(existing) = m.get(&session_key).filter(|l| fresh(l)) {
            return Ok(WeixinQrStartResult {
                qrcode_url: Some(existing.qrcode_url.clone()),
                message: "二维码已显示，请扫描。".into(),
                session_key,
            });
        }
    }
    let qr = fetch_qr_code(DEFAULT_BASE_URL, bot_type.unwrap_or(DEFAULT_ILINK_BOT_TYPE)).await?;
    let login = ActiveLogin {
        qrcode: qr.qrcode,
        qrcode_url: qr.qrcode_img_content.clone(),
        started_at: Instant::now(),
        current_api_base_url: None,
        pending_verify_code: None,
    };
    ACTIVE_LOGINS
        .lock()
        .unwrap()
        .insert(session_key.clone(), login);
    let _ = api_base_url;
    Ok(WeixinQrStartResult {
        qrcode_url: Some(qr.qrcode_img_content),
        message: "用手机微信扫描二维码。".into(),
        session_key,
    })
}

pub async fn wait_for_weixin_login(
    session_key: &str,
    _api_base_url: &str,
    timeout_ms: Option<u64>,
    bot_type: Option<&str>,
) -> crate::Result<WeixinQrWaitResult> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.unwrap_or(480_000).max(1000));
    let mut qr_refresh_count = 1_u32;
    loop {
        let mut login = {
            let m = ACTIVE_LOGINS.lock().unwrap();
            let Some(login) = m.get(session_key).cloned() else {
                return Ok(WeixinQrWaitResult {
                    connected: false,
                    already_connected: None,
                    bot_token: None,
                    account_id: None,
                    base_url: None,
                    user_id: None,
                    message: "没有进行中的登录。".into(),
                });
            };
            if !fresh(&login) {
                return Ok(WeixinQrWaitResult {
                    connected: false,
                    already_connected: None,
                    bot_token: None,
                    account_id: None,
                    base_url: None,
                    user_id: None,
                    message: "二维码已过期。".into(),
                });
            }
            login
        };
        if Instant::now() >= deadline {
            ACTIVE_LOGINS.lock().unwrap().remove(session_key);
            return Ok(WeixinQrWaitResult {
                connected: false,
                already_connected: None,
                bot_token: None,
                account_id: None,
                base_url: None,
                user_id: None,
                message: "登录超时。".into(),
            });
        }
        let base = login
            .current_api_base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.into());
        let resp = poll_qr_status(&base, &login.qrcode, login.pending_verify_code.as_deref()).await;
        match resp.status.as_str() {
            "wait" => {}
            "scaned" => {
                if login.pending_verify_code.take().is_some() {
                    ACTIVE_LOGINS
                        .lock()
                        .unwrap()
                        .insert(session_key.to_string(), login);
                }
                println!("正在验证");
            }
            "need_verifycode" => {
                let prompt = if login.pending_verify_code.is_some() {
                    "❌ 你输入的数字不匹配，请重新输入："
                } else {
                    "输入手机微信显示的数字，以继续连接："
                };
                login.pending_verify_code = Some(read_verify_code_from_stdin(prompt)?);
                ACTIVE_LOGINS
                    .lock()
                    .unwrap()
                    .insert(session_key.to_string(), login);
                continue;
            }
            "expired" | "verify_code_blocked" => {
                qr_refresh_count += 1;
                if qr_refresh_count > MAX_QR_REFRESH_COUNT {
                    ACTIVE_LOGINS.lock().unwrap().remove(session_key);
                    return Ok(WeixinQrWaitResult {
                        connected: false,
                        already_connected: None,
                        bot_token: None,
                        account_id: None,
                        base_url: None,
                        user_id: None,
                        message: "二维码多次失效。".into(),
                    });
                }
                let qr =
                    fetch_qr_code(DEFAULT_BASE_URL, bot_type.unwrap_or(DEFAULT_ILINK_BOT_TYPE))
                        .await?;
                login.qrcode = qr.qrcode;
                login.qrcode_url = qr.qrcode_img_content;
                login.started_at = Instant::now();
                login.pending_verify_code = None;
                display_qr_code(&login.qrcode_url)?;
                ACTIVE_LOGINS
                    .lock()
                    .unwrap()
                    .insert(session_key.to_string(), login);
            }
            "binded_redirect" => {
                ACTIVE_LOGINS.lock().unwrap().remove(session_key);
                return Ok(WeixinQrWaitResult {
                    connected: false,
                    already_connected: Some(true),
                    bot_token: None,
                    account_id: None,
                    base_url: None,
                    user_id: None,
                    message: "已连接过，无需重复连接。".into(),
                });
            }
            "scaned_but_redirect" => {
                if let Some(host) = resp.redirect_host {
                    login.current_api_base_url = Some(format!("https://{host}"));
                    ACTIVE_LOGINS
                        .lock()
                        .unwrap()
                        .insert(session_key.to_string(), login);
                }
            }
            "confirmed" => {
                ACTIVE_LOGINS.lock().unwrap().remove(session_key);
                let account_id = resp.ilink_bot_id.ok_or("登录失败：服务器未返回 bot ID。")?;
                if let Some(token) = resp.bot_token.as_deref() {
                    save_weixin_account(
                        &account_id,
                        Some(token),
                        resp.baseurl.as_deref(),
                        resp.ilink_user_id.as_deref(),
                    )?;
                }
                if let Some(user_id) = resp.ilink_user_id.as_deref() {
                    let normalized_account_id = account_id
                        .replace('@', "-")
                        .replace('.', "-")
                        .to_ascii_lowercase();
                    clear_stale_accounts_for_user_id(&normalized_account_id, user_id)?;
                }
                return Ok(WeixinQrWaitResult {
                    connected: true,
                    already_connected: None,
                    bot_token: resp.bot_token,
                    account_id: Some(account_id),
                    base_url: resp.baseurl,
                    user_id: resp.ilink_user_id,
                    message: "登录成功。".into(),
                });
            }
            _ => {}
        }
        sleep(Duration::from_millis(1000)).await;
    }
}
