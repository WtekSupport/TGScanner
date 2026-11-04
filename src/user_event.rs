use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum PhotoResult {
    None,
    Saved(PathBuf),
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    pub username: Option<String>,
    pub bio: String,
    pub photo: PhotoResult,
}

#[derive(Clone, Debug)]
pub struct GroupProfile {
    pub id: i64,
    pub title: String,
    pub is_megagroup: bool,
}

#[derive(Clone, Debug)]
pub struct ChannelProfile {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub about: String,
    pub photo: PhotoResult,
}

#[derive(Clone, Debug)]
pub enum UserEvent {
    ConnectedAlready,
    CodeSent,
    SignedIn,
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
