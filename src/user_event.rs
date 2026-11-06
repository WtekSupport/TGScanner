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
    /// Serialized [`grammers_session::PackedChat`] for worker operations.
    pub packed_hex: String,
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

/// Node in the group membership graph (users and group chats).
#[derive(Clone, Debug)]
pub struct GroupMapNode {
    /// Stable id: positive = user id, negative = encoded chat id (see `group_map_layout`).
    pub id: i64,
    pub label: String,
    /// 0 = self, 1 = user, 2 = group/megagroup (Slint `kind`).
    pub kind: i32,
}

#[derive(Clone, Debug)]
pub struct GroupMapEdge {
    pub user_id: i64,
    pub chat_id: i64,
}

#[derive(Clone, Debug)]
pub struct GroupMapReady {
    pub nodes: Vec<GroupMapNode>,
    pub edges: Vec<GroupMapEdge>,
    /// Short description of the anchor (resolved group/channel/user), for the stats line.
    pub anchor_label: String,
    /// Expansion depth requested (1 = anchor participants + their shared groups; 2+ adds more shared groups among participants).
    pub depth: u32,
    pub dialogs_scanned: usize,
    pub groups_processed: usize,
    pub users_found: usize,
    pub errors: Vec<String>,
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
    /// Progress while scanning/deleting own messages in a common chat (does not end busy state).
    DeleteMessagesProgress {
        chat_title: String,
        scanned: u64,
        deleted: u64,
        flood_wait_secs: Option<u32>,
    },
    DeleteMessagesFinished {
        chat_title: String,
        scanned: u64,
        deleted: u64,
        failed_batches: u32,
    },
    /// Building group map from dialogs (keeps busy; progress lines go to log).
    GroupMapProgress {
        message: String,
        dialogs_scanned: usize,
        groups_done: usize,
        users_seen: usize,
    },
    GroupMapReady(GroupMapReady),
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
