use tokio::time::{Duration, sleep};

use crate::{
    api::{
        GetUpdatesReq, SESSION_EXPIRED_ERRCODE, WeixinApiOptions, get_remaining_pause_ms,
        get_updates, pause_session,
    },
    messaging::process_message::{MessageHandler, ProcessMessageDeps, process_one_message},
    storage::{
        get_sync_buf_file_path, get_sync_buf_file_path_candidates, load_get_updates_buf,
        save_get_updates_buf,
    },
};

const DEFAULT_LONG_POLL_TIMEOUT_MS: u64 = 35_000;
const MAX_CONSECUTIVE_FAILURES: u32 = 3;
const BACKOFF_DELAY_MS: u64 = 30_000;
const RETRY_DELAY_MS: u64 = 2_000;

#[derive(Clone)]
pub struct MonitorWeixinOpts {
    pub base_url: String,
    pub cdn_base_url: String,
    pub token: Option<String>,
    pub account_id: String,
    pub long_poll_timeout_ms: Option<u64>,
    pub on_message: MessageHandler,
}

pub async fn monitor_weixin_provider(
    opts: MonitorWeixinOpts,
    mut stop: tokio::sync::watch::Receiver<bool>,
) -> crate::Result<()> {
    let log = crate::util::logger().with_account(&opts.account_id);
    log.info(format!(
        "monitor started: baseUrl={} account={}",
        opts.base_url, opts.account_id
    ));
    let sync_path = get_sync_buf_file_path(&opts.account_id);
    let mut get_updates_buf = get_sync_buf_file_path_candidates(&opts.account_id)
        .into_iter()
        .find_map(load_get_updates_buf)
        .unwrap_or_default();
    let mut next_timeout = opts
        .long_poll_timeout_ms
        .unwrap_or(DEFAULT_LONG_POLL_TIMEOUT_MS);
    let mut failures = 0_u32;

    loop {
        if *stop.borrow() {
            break;
        }
        let api_opts = WeixinApiOptions {
            base_url: opts.base_url.clone(),
            token: opts.token.clone(),
            timeout_ms: None,
            long_poll_timeout_ms: Some(next_timeout),
        };
        let fut = get_updates(
            GetUpdatesReq {
                get_updates_buf: Some(get_updates_buf.clone()),
            },
            &api_opts,
        );
        let resp = tokio::select! {
            _ = stop.changed() => break,
            r = fut => r,
        };
        match resp {
            Ok(resp) => {
                if let Some(ms) = resp.longpolling_timeout_ms.filter(|ms| *ms > 0) {
                    next_timeout = ms;
                }
                let is_error = resp.ret.map(|r| r != 0).unwrap_or(false)
                    || resp.errcode.map(|e| e != 0).unwrap_or(false);
                if is_error {
                    let expired = resp.errcode == Some(SESSION_EXPIRED_ERRCODE)
                        || resp.ret == Some(SESSION_EXPIRED_ERRCODE);
                    if expired {
                        pause_session(&opts.account_id);
                        let pause_ms = get_remaining_pause_ms(&opts.account_id);
                        log.error(format!(
                            "session expired, pausing {} min",
                            pause_ms.div_ceil(60_000)
                        ));
                        failures = 0;
                        tokio::select! { _ = stop.changed() => break, _ = sleep(Duration::from_millis(pause_ms)) => {} }
                    } else {
                        failures += 1;
                        log.error(format!(
                            "getUpdates failed: ret={:?} errcode={:?} ({}/{})",
                            resp.ret, resp.errcode, failures, MAX_CONSECUTIVE_FAILURES
                        ));
                        let delay = if failures >= MAX_CONSECUTIVE_FAILURES {
                            failures = 0;
                            BACKOFF_DELAY_MS
                        } else {
                            RETRY_DELAY_MS
                        };
                        tokio::select! { _ = stop.changed() => break, _ = sleep(Duration::from_millis(delay)) => {} }
                    }
                    continue;
                }
                failures = 0;
                if let Some(buf) = resp.get_updates_buf.filter(|b| !b.is_empty()) {
                    save_get_updates_buf(&sync_path, &buf)?;
                    get_updates_buf = buf;
                }
                for msg in resp.msgs.unwrap_or_default() {
                    let deps = ProcessMessageDeps {
                        account_id: opts.account_id.clone(),
                        base_url: opts.base_url.clone(),
                        cdn_base_url: opts.cdn_base_url.clone(),
                        token: opts.token.clone(),
                        on_message: opts.on_message.clone(),
                    };
                    process_one_message(msg, &deps).await?;
                }
            }
            Err(err) => {
                failures += 1;
                log.error(format!(
                    "getUpdates error ({}/{}): {err:?}",
                    failures, MAX_CONSECUTIVE_FAILURES
                ));
                let delay = if failures >= MAX_CONSECUTIVE_FAILURES {
                    failures = 0;
                    BACKOFF_DELAY_MS
                } else {
                    RETRY_DELAY_MS
                };
                tokio::select! { _ = stop.changed() => break, _ = sleep(Duration::from_millis(delay)) => {} }
            }
        }
    }
    log.info("monitor ended");
    Ok(())
}
