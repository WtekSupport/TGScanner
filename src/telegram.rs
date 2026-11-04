use crate::config::AppConfig;
use crate::crash_log;
use crate::secure_store;
use crate::user_event::{
    ChannelProfile, ErrorKind, GroupProfile, PhotoResult, UserEvent, UserProfile,
};
use anyhow::anyhow;
use grammers_client::types::{Channel, Chat, Downloadable, Group, Media, User};
use grammers_client::{Client, Config, SignInError};
use grammers_session::{PackedChat, PackedType, Session};
use grammers_tl_types as tl;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

const PHONE_DELAY_SECS: u64 = 3;
const TELEGRAM_CHANNEL_BASE: i64 = 1_000_000_000_000;

type OpResult = std::result::Result<UserEvent, UserEvent>;
type IoResult<T> = std::result::Result<T, UserEvent>;
type ChatResult = std::result::Result<Chat, UserEvent>;

#[derive(Debug)]
pub enum WorkerCommand {
    Connect {
        api_id: i32,
        api_hash: String,
        phone: String,
    },
    SubmitCode(String),
    SubmitPassword(String),
    LookupUsername(String),
    LookupId(i64),
    LookupPhone(String),
    LookupUrl(String),
}

pub struct TelegramHandle {
    pub tx: mpsc::Sender<WorkerCommand>,
}

pub fn spawn_worker(base_dir: PathBuf) -> (TelegramHandle, mpsc::Receiver<UserEvent>) {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<WorkerCommand>(32);
    let (result_tx, result_rx) = mpsc::channel::<UserEvent>(64);
    let phone_state_path = base_dir.join(".tguser_phone_lookup_ts");

    std::thread::spawn(move || {
        let base_for_panic = base_dir.clone();
        let worker = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async move {
            let client: Arc<Mutex<Option<Client>>> = Arc::new(Mutex::new(None));
            let mut login_token = None::<grammers_client::types::LoginToken>;
            let mut password_token = None::<grammers_client::types::PasswordToken>;

            while let Some(cmd) = cmd_rx.recv().await {
                let result = match cmd {
                    WorkerCommand::Connect {
                        api_id,
                        api_hash,
                        phone,
                    } => {
                        connect_client(
                            &base_dir,
                            api_id,
                            api_hash,
                            &phone,
                            &client,
                            &mut login_token,
                        )
                        .await
                    }
                    WorkerCommand::SubmitCode(code) => {
                        submit_code(
                            &client,
                            &mut login_token,
                            &code,
                            &mut password_token,
                            &base_dir,
                        )
                        .await
                    }
                    WorkerCommand::SubmitPassword(pw) => {
                        submit_password(&client, password_token.take(), &pw, &base_dir).await
                    }
                    WorkerCommand::LookupUsername(u) => {
                        with_client(&client, |c| {
                            let out = output_dir_for(&base_dir);
                            lookup_username(c, u, out)
                        })
                        .await
                    }
                    WorkerCommand::LookupId(id) => {
                        with_client(&client, |c| lookup_id(c, id, output_dir_for(&base_dir)))
                            .await
                    }
                    WorkerCommand::LookupPhone(p) => {
                        enforce_phone_delay(&phone_state_path).await;
                        with_client(&client, |c| lookup_phone(c, p, output_dir_for(&base_dir)))
                            .await
                    }
                    WorkerCommand::LookupUrl(url) => {
                        with_client(&client, |c| lookup_url(c, url, output_dir_for(&base_dir)))
                            .await
                    }
                };

                match result {
                    Ok(ev) => {
                        let _ = result_tx.send(ev).await;
                    }
                    Err(ev) => {
                        let _ = result_tx.send(ev).await;
                    }
                }
            }
        });
        }));
        if let Err(payload) = worker {
            crash_log::log_task_panic(&base_for_panic, "telegram-worker thread", payload);
        }
    });

    (TelegramHandle { tx: cmd_tx }, result_rx)
}

fn output_dir_for(base: &Path) -> PathBuf {
    let cfg = AppConfig::load(&base.to_path_buf());
    PathBuf::from(cfg.output_dir)
}

async fn with_client<F, Fut>(client_slot: &Arc<Mutex<Option<Client>>>, f: F) -> OpResult
where
    F: FnOnce(Client) -> Fut,
    Fut: std::future::Future<Output = OpResult>,
{
    let client = client_slot
        .lock()
        .await
        .as_ref()
        .ok_or(UserEvent::err(ErrorKind::NotConnected, ""))?
        .clone();
    f(client).await
}

async fn connect_client(
    base_dir: &Path,
    api_id: i32,
    api_hash: String,
    phone: &str,
    client_slot: &Arc<Mutex<Option<Client>>>,
    login_token: &mut Option<grammers_client::types::LoginToken>,
) -> OpResult {
    let session = load_session_vault(base_dir)?;
    let client = Client::connect(Config {
        session,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await
    .map_err(|e| UserEvent::err(ErrorKind::Connect, format!("connect: {e}")))?;

    if client
        .is_authorized()
        .await
        .map_err(|e| UserEvent::err(ErrorKind::Connect, format!("{e}")))?
    {
        *client_slot.lock().await = Some(client);
        return Ok(UserEvent::ConnectedAlready);
    }

    let token = client
        .request_login_code(phone)
        .await
        .map_err(|e| UserEvent::err(ErrorKind::RequestCode, format!("request code: {e}")))?;
    *login_token = Some(token);
    *client_slot.lock().await = Some(client);
    Ok(UserEvent::CodeSent)
}

async fn submit_code(
    client_slot: &Arc<Mutex<Option<Client>>>,
    login_token: &mut Option<grammers_client::types::LoginToken>,
    code: &str,
    password_token: &mut Option<grammers_client::types::PasswordToken>,
    base_dir: &Path,
) -> OpResult {
    let token = login_token
        .take()
        .ok_or_else(|| UserEvent::err(ErrorKind::RequestCodeFirst, ""))?;
    let client = client_slot
        .lock()
        .await
        .as_ref()
        .ok_or_else(|| UserEvent::err(ErrorKind::ClientNotConnected, ""))?
        .clone();

    match client.sign_in(&token, code.trim()).await {
        Ok(_) => {
            save_session(&client, base_dir)?;
            Ok(UserEvent::SignedIn)
        }
        Err(SignInError::PasswordRequired(pt)) => {
            *password_token = Some(pt);
            Ok(UserEvent::Need2fa)
        }
        Err(SignInError::SignUpRequired { .. }) => {
            Err(UserEvent::err(ErrorKind::AccountNotRegistered, ""))
        }
        Err(e) => Err(UserEvent::err(ErrorKind::SignIn, format!("sign_in: {e}"))),
    }
}

async fn submit_password(
    client_slot: &Arc<Mutex<Option<Client>>>,
    password_token: Option<grammers_client::types::PasswordToken>,
    password: &str,
    base_dir: &Path,
) -> OpResult {
    let pt = password_token.ok_or_else(|| UserEvent::err(ErrorKind::NoPasswordRequest, ""))?;
    let client = client_slot
        .lock()
        .await
        .as_ref()
        .ok_or_else(|| UserEvent::err(ErrorKind::ClientNotConnected, ""))?
        .clone();
    client
        .check_password(pt, password.trim())
        .await
        .map_err(|e| UserEvent::err(ErrorKind::CheckPassword, format!("check_password: {e}")))?;
    save_session(&client, base_dir)?;
    Ok(UserEvent::SignedIn)
}

fn load_session_vault(base: &Path) -> Result<Session, UserEvent> {
    let enc = secure_store::session_path(base);
    let legacy = secure_store::legacy_session_path(base);

    if enc.is_file() {
        let raw = secure_store::load_plain_or_decrypt(&enc).map_err(|e| {
            UserEvent::err(ErrorKind::Session, format!("session vault: {e}"))
        })?;
        return Session::load(&raw).map_err(|e| UserEvent::err(ErrorKind::Session, format!("{e}")));
    }

    if legacy.is_file() {
        let raw = std::fs::read(&legacy).map_err(|e| {
            UserEvent::err(ErrorKind::Session, format!("read legacy session: {e}"))
        })?;
        let session = Session::load(&raw).map_err(|e| UserEvent::err(ErrorKind::Session, format!("{e}")))?;
        persist_session_file(&session, base)?;
        let backup = base.join(".tguser_session.bak");
        let _ = std::fs::rename(legacy, backup);
        return Ok(session);
    }

    Ok(Session::new())
}

fn persist_session_file(session: &Session, base: &Path) -> Result<(), UserEvent> {
    secure_store::save_encrypted(&secure_store::session_path(base), &session.save()).map_err(|e| {
        UserEvent::err(ErrorKind::SaveSession, format!("{e}"))
    })
}

fn save_session(client: &Client, base: &Path) -> Result<(), UserEvent> {
    persist_session_file(client.session(), base)
}

async fn lookup_username(client: Client, username: String, output_dir: PathBuf) -> OpResult {
    let name = username.trim().trim_start_matches('@');
    let chat = client
        .resolve_username(name)
        .await
        .map_err(|e| UserEvent::err(ErrorKind::Generic, format!("{e}")))?
        .ok_or_else(|| {
            UserEvent::err(
                ErrorKind::NotFoundUsername,
                format!("@{name}"),
            )
        })?;
    describe_chat(&client, chat, &output_dir).await
}

async fn lookup_id(client: Client, id: i64, output_dir: PathBuf) -> OpResult {
    let mut packed = packed_from_numeric_id(id)
        .ok_or_else(|| UserEvent::err(ErrorKind::UnsupportedId, ""))?;
    if packed_needs_access_hash(&packed) && usable_access_hash(packed.access_hash).is_none() {
        if let Some(from_dialogs) = packed_from_dialogs(&client, id).await {
            if usable_access_hash(from_dialogs.access_hash).is_some() {
                packed = from_dialogs;
            }
        }
    }
    if packed_needs_access_hash(&packed) && usable_access_hash(packed.access_hash).is_none() {
        return Err(UserEvent::err(ErrorKind::UnpackChat, ""));
    }
    let chat = fetch_chat_by_packed(&client, packed).await?;
    describe_chat(&client, chat, &output_dir).await
}

fn usable_access_hash(hash: Option<i64>) -> Option<i64> {
    hash.filter(|h| *h != 0)
}

async fn fetch_chat_by_packed(client: &Client, packed: PackedChat) -> ChatResult {
    match packed.ty {
        PackedType::User | PackedType::Bot => fetch_user_chat(client, packed).await,
        PackedType::Chat => fetch_basic_group_chat(client, packed.id).await,
        PackedType::Megagroup | PackedType::Broadcast | PackedType::Gigagroup => {
            fetch_channel_chat(client, packed.id, usable_access_hash(packed.access_hash)).await
        }
    }
}

async fn fetch_user_chat(client: &Client, packed: PackedChat) -> ChatResult {
    let access_hash = usable_access_hash(packed.access_hash)
        .ok_or_else(|| UserEvent::err(ErrorKind::UnpackChat, ""))?;
    let users = client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::User(tl::types::InputUser {
                user_id: packed.id,
                access_hash,
            })],
        })
        .await
        .map_err(|e| UserEvent::err(ErrorKind::UnpackChat, format!("{e}")))?;
    let user = users
        .into_iter()
        .next()
        .ok_or_else(|| UserEvent::err(ErrorKind::UnpackChat, "user not found"))?;
    Ok(Chat::User(User::from_raw(user)))
}

async fn fetch_basic_group_chat(client: &Client, chat_id: i64) -> ChatResult {
    let chats = match client
        .invoke(&tl::functions::messages::GetChats {
            id: vec![chat_id],
        })
        .await
        .map_err(|e| UserEvent::err(ErrorKind::UnpackChat, format!("{e}")))?
    {
        tl::enums::messages::Chats::Chats(c) => c.chats,
        tl::enums::messages::Chats::Slice(c) => c.chats,
    };
    let chat = chats
        .into_iter()
        .next()
        .ok_or_else(|| UserEvent::err(ErrorKind::UnpackChat, "chat not found"))?;
    Ok(Chat::from_raw(chat))
}

async fn fetch_channel_chat(
    client: &Client,
    channel_id: i64,
    access_hash: Option<i64>,
) -> ChatResult {
    let access_hash = access_hash.ok_or_else(|| UserEvent::err(ErrorKind::UnpackChat, ""))?;
    let chats = match client
        .invoke(&tl::functions::channels::GetChannels {
            id: vec![tl::enums::InputChannel::Channel(tl::types::InputChannel {
                channel_id,
                access_hash,
            })],
        })
        .await
        .map_err(|e| UserEvent::err(ErrorKind::UnpackChat, format!("{e}")))?
    {
        tl::enums::messages::Chats::Chats(c) => c.chats,
        tl::enums::messages::Chats::Slice(c) => c.chats,
    };
    let chat = chats
        .into_iter()
        .next()
        .ok_or_else(|| UserEvent::err(ErrorKind::UnpackChat, "channel not found"))?;
    Ok(Chat::from_raw(chat))
}

fn packed_needs_access_hash(packed: &PackedChat) -> bool {
    !matches!(packed.ty, PackedType::Chat)
}

fn telegram_display_id(chat: &Chat) -> i64 {
    match chat {
        Chat::User(user) => user.id(),
        Chat::Group(group) => {
            if group.is_megagroup() {
                -TELEGRAM_CHANNEL_BASE - group.id()
            } else {
                -group.id()
            }
        }
        Chat::Channel(channel) => -TELEGRAM_CHANNEL_BASE - channel.id(),
    }
}

async fn packed_from_dialogs(client: &Client, telegram_id: i64) -> Option<PackedChat> {
    let mut dialogs = client.iter_dialogs();
    while let Ok(Some(dialog)) = dialogs.next().await {
        let chat = dialog.chat();
        if telegram_display_id(&chat) == telegram_id {
            return Some(chat.pack());
        }
    }
    None
}

fn packed_from_numeric_id(id: i64) -> Option<PackedChat> {
    if id > 0 {
        return Some(PackedChat {
            ty: PackedType::User,
            id,
            access_hash: None,
        });
    }
    if id <= -TELEGRAM_CHANNEL_BASE {
        let channel_id = (-id) - TELEGRAM_CHANNEL_BASE;
        return Some(PackedChat {
            ty: PackedType::Megagroup,
            id: channel_id,
            access_hash: None,
        });
    }
    if id < 0 {
        return Some(PackedChat {
            ty: PackedType::Chat,
            id: -id,
            access_hash: None,
        });
    }
    None
}

async fn lookup_url(client: Client, url: String, output_dir: PathBuf) -> OpResult {
    let (channel, msg_id) = parse_tme_url(&url)?;
    let chat = client
        .resolve_username(&channel)
        .await
        .map_err(|e| UserEvent::err(ErrorKind::Generic, format!("{e}")))?
        .ok_or_else(|| UserEvent::err(ErrorKind::ChannelNotFound, ""))?;
    let packed = chat.pack();
    let messages = client
        .get_messages_by_id(packed, &[msg_id])
        .await
        .map_err(|e| UserEvent::err(ErrorKind::Generic, format!("{e}")))?;
    let message = messages
        .into_iter()
        .flatten()
        .next()
        .ok_or_else(|| UserEvent::err(ErrorKind::MessageUnavailable, ""))?;

    std::fs::create_dir_all(&output_dir)
        .map_err(|e| UserEvent::err(ErrorKind::Generic, e.to_string()))?;
    let path = output_dir.join(format!("message_{channel}_{msg_id}.json"));
    let summary = serde_json::json!({
        "id": message.id(),
        "text_preview": message.text().chars().take(200).collect::<String>(),
    });
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&summary)
            .map_err(|e| UserEvent::err(ErrorKind::Generic, e.to_string()))?,
    )
    .map_err(|e| UserEvent::err(ErrorKind::Generic, e.to_string()))?;

    Ok(UserEvent::MessageExport {
        msg_id,
        text: message.text().to_string(),
        json_path: path,
    })
}

async fn lookup_phone(client: Client, phone: String, output_dir: PathBuf) -> OpResult {
    let phone = normalize_phone_e164(&phone)?;
    let contact = tl::types::InputPhoneContact {
        client_id: 0,
        phone: phone.clone(),
        first_name: "Lookup".into(),
        last_name: "Temporary".into(),
    };
    let imported = client
        .invoke(&tl::functions::contacts::ImportContacts {
            contacts: vec![tl::enums::InputContact::InputPhoneContact(contact)],
        })
        .await
        .map_err(|e| UserEvent::err(ErrorKind::ImportContacts, format!("import: {e}")))?;

    let users = match imported {
        tl::enums::contacts::ImportedContacts::Contacts(c) => c.users,
    };
    if users.is_empty() {
        return Ok(UserEvent::PhoneNotFound { phone });
    }

    let user = User::from_raw(users[0].clone());
    let chat = Chat::User(user.clone());
    let profile = describe_chat(&client, chat, &output_dir).await?;

    let mut contact_removed = false;
    if let Some(input) = user.pack().try_to_input_user() {
        if client
            .invoke(&tl::functions::contacts::DeleteContacts { id: vec![input] })
            .await
            .is_ok()
        {
            contact_removed = true;
        }
    }

    Ok(UserEvent::PhoneLookupComplete {
        profile: Box::new(profile),
        contact_removed,
    })
}

async fn describe_chat(client: &Client, chat: Chat, output_dir: &Path) -> OpResult {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| UserEvent::err(ErrorKind::Generic, e.to_string()))?;
    match chat {
        Chat::User(user) => describe_user(client, &user, output_dir).await,
        Chat::Group(group) => Ok(UserEvent::GroupProfile(describe_group(&group))),
        Chat::Channel(channel) => describe_channel(client, &channel, output_dir).await,
    }
}

async fn describe_user(client: &Client, user: &User, output_dir: &Path) -> OpResult {
    let bio = user_bio(client, user).await.unwrap_or_default();
    let photo = download_first_profile(client, user.pack(), user.id(), output_dir).await;

    Ok(UserEvent::UserProfile(UserProfile {
        id: user.id(),
        name: user.full_name(),
        username: user.username().map(str::to_string),
        bio,
        photo,
    }))
}

fn describe_group(group: &Group) -> GroupProfile {
    GroupProfile {
        id: group.id(),
        title: group.title().to_string(),
        is_megagroup: group.is_megagroup(),
    }
}

async fn describe_channel(
    client: &Client,
    channel: &Channel,
    output_dir: &Path,
) -> OpResult {
    let about = channel_about(client, channel).await.unwrap_or_default();
    let photo = download_first_profile(client, channel.pack(), channel.id(), output_dir).await;

    Ok(UserEvent::ChannelProfile(ChannelProfile {
        id: channel.id(),
        title: channel.title().to_string(),
        username: channel.username().map(str::to_string),
        about,
        photo,
    }))
}

async fn user_bio(client: &Client, user: &User) -> anyhow::Result<String> {
    let input = user
        .pack()
        .try_to_input_user()
        .ok_or_else(|| anyhow!("no input user"))?;
    let full = client
        .invoke(&tl::functions::users::GetFullUser { id: input })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    let tl::enums::users::UserFull::Full(f) = full;
    let tl::enums::UserFull::Full(u) = f.full_user;
    Ok(u.about.unwrap_or_default())
}

async fn channel_about(client: &Client, channel: &Channel) -> anyhow::Result<String> {
    let input = channel
        .pack()
        .try_to_input_channel()
        .ok_or_else(|| anyhow!("no input channel"))?;
    let full = client
        .invoke(&tl::functions::channels::GetFullChannel { channel: input })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    let tl::enums::messages::ChatFull::Full(f) = full;
    Ok(f.full_chat.about())
}

async fn download_first_profile(
    client: &Client,
    packed: PackedChat,
    id: i64,
    output_dir: &Path,
) -> PhotoResult {
    let mut photos = client.iter_profile_photos(packed);
    let photo = match photos.next().await {
        Ok(Some(p)) => p,
        Ok(None) => return PhotoResult::None,
        Err(e) => return PhotoResult::Failed(format!("{e}")),
    };
    let path = output_dir.join(format!("{id}_profile.jpg"));
    match client
        .download_media(
            &Downloadable::Media(Media::Photo(photo)),
            &path,
        )
        .await
    {
        Ok(()) => PhotoResult::Saved(path),
        Err(e) => PhotoResult::Failed(format!("{e}")),
    }
}

fn parse_tme_url(url: &str) -> IoResult<(String, i32)> {
    let url = url.trim();
    let lower = url.to_lowercase();
    for prefix in [
        "https://t.me/",
        "http://t.me/",
        "https://telegram.me/",
        "http://telegram.me/",
        "t.me/",
    ] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            return parse_tme_path(rest);
        }
    }
    if let Some(idx) = lower.find("t.me/") {
        return parse_tme_path(&url[idx + 5..]);
    }
    Err(UserEvent::err(ErrorKind::InvalidUrl, ""))
}

fn parse_tme_path(path: &str) -> IoResult<(String, i32)> {
    let mut parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    if parts.first() == Some(&"s") {
        parts.remove(0);
    }
    if parts.len() < 2 {
        return Err(UserEvent::err(ErrorKind::InvalidUrlPath, ""));
    }
    let channel = parts[0].to_string();
    if !channel
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(UserEvent::err(ErrorKind::InvalidChannelName, ""));
    }
    let id: i32 = parts[1].parse().map_err(|_| {
        UserEvent::err(ErrorKind::InvalidMessageId, "message id must be a number")
    })?;
    if id <= 0 {
        return Err(UserEvent::err(ErrorKind::InvalidMessageId, ""));
    }
    Ok((channel, id))
}

fn normalize_phone_e164(phone: &str) -> IoResult<String> {
    let mut p = phone.trim().replace([' ', '-'], "");
    if !p.starts_with('+') {
        p = format!("+{p}");
    }
    if p.len() < 9 || !p[1..].chars().all(|c| c.is_ascii_digit()) {
        return Err(UserEvent::err(ErrorKind::InvalidPhoneE164, ""));
    }
    Ok(p)
}

async fn enforce_phone_delay(path: &Path) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    if path.is_file() {
        if let Ok(s) = std::fs::read_to_string(path) {
            if let Ok(last) = s.trim().parse::<f64>() {
                let wait = PHONE_DELAY_SECS as f64 - (now - last);
                if wait > 0.0 {
                    tokio::time::sleep(tokio::time::Duration::from_secs_f64(wait)).await;
                }
            }
        }
    }
    let _ = std::fs::write(path, format!("{now}"));
}
