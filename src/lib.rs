pub mod api;
pub mod auth;
pub mod bot;
pub mod cdn;
pub mod media;
pub mod messaging;
pub mod monitor;
pub mod multi_bot;
pub mod storage;
pub mod util;

pub use api::{
    GetUpdatesReq, GetUpdatesResp, SendMessageReq, WeixinApiOptions, WeixinMessage, get_config,
    get_updates, get_upload_url, notify_start, notify_stop, send_message, send_typing,
};
pub use auth::accounts::{CDN_BASE_URL, DEFAULT_BASE_URL, resolve_weixin_account};
pub use auth::login_qr::{display_qr_code, start_weixin_login_with_qr, wait_for_weixin_login};
pub use bot::{StartOptions, WeixinBot, WeixinBotOptions};
pub use cdn::upload::{
    UploadedFileInfo, upload_file_attachment_to_weixin, upload_file_to_weixin,
    upload_video_to_weixin,
};
pub use messaging::send::{
    WeixinMsgContext, send_file_message_weixin, send_image_message_weixin, send_message_weixin,
    send_video_message_weixin,
};
pub use multi_bot::{BotAccountOptions, MultiStartOptions, MultiWeixinBot, MultiWeixinBotOptions};

pub type Result<T> = api::Result<T>;
