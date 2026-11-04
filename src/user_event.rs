use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum PhotoResult {
    None,
    Saved(PathBuf),
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct CommonChatSummary {
    pub title: String,
    pub kind: String,
    pub telegram_id: i64,
    pub link: Option<String>,
}

#[derive(Clone, Debug)]
pub struct UserProfile {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub name: String,
    pub username: Option<String>,
    pub usernames: Vec<String>,
    pub phone: Option<String>,
    pub bio: String,
    pub status: String,
    pub lang_code: Option<String>,
    pub is_bot: bool,
    pub verified: bool,
    pub premium: bool,
    pub scam: bool,
    pub fake: bool,
    pub deleted: bool,
    pub restricted: bool,
    pub support: bool,
    pub contact: bool,
    pub mutual_contact: bool,
    pub is_self: bool,
    pub restriction_reasons: Vec<String>,
    pub common_chats_count: Option<i32>,
    pub common_chats: Vec<CommonChatSummary>,
    pub phone_calls_available: Option<bool>,
    pub video_calls_available: Option<bool>,
    pub photo: PhotoResult,
    pub json_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct GroupProfile {
    pub id: i64,
    pub title: String,
    pub is_megagroup: bool,
    pub username: Option<String>,
    pub participants_count: Option<i32>,
    pub about: String,
    pub json_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct ChannelProfile {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub usernames: Vec<String>,
    pub about: String,
    pub participants_count: Option<i32>,
    pub verified: bool,
    pub scam: bool,
    pub fake: bool,
    pub restricted: bool,
    pub megagroup: bool,
    pub broadcast: bool,
    pub restriction_reasons: Vec<String>,
    pub photo: PhotoResult,
    pub json_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum UserEvent {
    /// Session verified (e.g. via `get_me`); lookups are allowed.
    Ready {
        account: String,
    },
    /// Connection lost or session no longer usable.
    SessionLost,
    CodeSent,
    Need2fa,
    UserProfile(UserProfile),
    GroupProfile(GroupProfile),
    ChannelProfile(ChannelProfile),
    MessageExport {
        msg_id: i32,
        text: String,
        json_path: PathBuf,
    },
    PhoneNotFound {
        phone: String,
    },
    PhoneLookupComplete {
        profile: Box<UserEvent>,
        contact_removed: bool,
    },
    Error {
        kind: ErrorKind,
        detail: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    NotConnected,
    RequestCodeFirst,
    ClientNotConnected,
    NoPasswordRequest,
    AccountNotRegistered,
    Session,
    Connect,
    RequestCode,
    SignIn,
    CheckPassword,
    SaveSession,
    NotFoundUsername,
    UnsupportedId,
    UnpackChat,
    ChannelNotFound,
    MessageUnavailable,
    InvalidUrl,
    InvalidUrlPath,
    InvalidChannelName,
    InvalidMessageId,
    InvalidPhoneE164,
    ImportContacts,
    Generic,
}

impl UserEvent {
    pub fn err(kind: ErrorKind, detail: impl Into<String>) -> Self {
        UserEvent::Error {
            kind,
            detail: detail.into(),
        }
    }
}
