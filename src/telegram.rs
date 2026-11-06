use crate::config::AppConfig;
use crate::crash_log;
use crate::secure_store;
use crate::user_event::{
    ChannelProfile, CommonChatSummary, ErrorKind, GroupMapEdge, GroupMapNode, GroupMapReady,
    GroupProfile, PhotoResult, UserEvent, UserProfile,
};
use anyhow::anyhow;
use grammers_client::types::{Channel, Chat, Downloadable, Group, Media, User};
use grammers_client::{Client, Config, SignInError};
use grammers_client::InvocationError;
use grammers_session::{PackedChat, PackedType, Session};
use grammers_tl_types as tl;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

const PHONE_DELAY_SECS: u64 = 3;
const TELEGRAM_CHANNEL_BASE: i64 = 1_000_000_000_000;

/// Limits for group-map scan (participants / expansions).
const GROUP_MAP_MAX_PARTICIPANTS_PER_CHAT: usize = 500;
const GROUP_MAP_API_DELAY_MS: u64 = 50;
/// If Telegram/network stalls on participant pagination, stop waiting so the UI can recover.
const GROUP_MAP_PARTICIPANT_ITER_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(120);

type OpResult = std::result::Result<UserEvent, UserEvent>;
type IoResult<T> = std::result::Result<T, UserEvent>;
type ChatResult = std::result::Result<Chat, UserEvent>;

#[derive(Clone, Debug)]
struct ConnectParams {
    api_id: i32,
    api_hash: String,
    phone: String,
}

#[derive(Clone, Debug)]
enum LookupOp {
    Username(String),
    Id(i64),
    Phone(String),
    Url(String),
}

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
    DeleteOwnMessages {
        packed_hex: String,
        chat_title: String,
    },
    /// Build a focused membership graph: resolve **Lookup** query (same modes as lookup) as anchor,
    /// then depth 1 = that chat's participants + edges to anchor and shared groups among them; depth ≥ 2 adds more shared groups.
    BuildGroupMap {
        lookup_mode: i32,
        query: String,
        depth: u32,
    },
}

pub struct TelegramHandle {
    pub tx: mpsc::Sender<WorkerCommand>,
}

pub fn spawn_worker(base_dir: PathBuf) -> (TelegramHandle, mpsc::Receiver<UserEvent>) {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<WorkerCommand>(32);
    // Group map can emit many progress lines; a small channel blocks the worker on `send().await`.
    let (result_tx, result_rx) = mpsc::channel::<UserEvent>(512);
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
            let connect_params: Arc<Mutex<Option<ConnectParams>>> = Arc::new(Mutex::new(None));
            let mut login_token = None::<grammers_client::types::LoginToken>;
            let mut password_token = None::<grammers_client::types::PasswordToken>;

            while let Some(cmd) = cmd_rx.recv().await {
                let result = match cmd {
                    WorkerCommand::Connect {
                        api_id,
                        api_hash,
                        phone,
                    } => {
                        *connect_params.lock().await = Some(ConnectParams {
                            api_id,
                            api_hash: api_hash.clone(),
                            phone: phone.clone(),
                        });
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
                        resilient_lookup(
                            &base_dir,
                            &client,
                            &connect_params,
                            &mut login_token,
                            LookupOp::Username(u),
                            output_dir_for(&base_dir),
                        )
                        .await
                    }
                    WorkerCommand::LookupId(id) => {
                        resilient_lookup(
                            &base_dir,
                            &client,
                            &connect_params,
                            &mut login_token,
                            LookupOp::Id(id),
                            output_dir_for(&base_dir),
                        )
                        .await
                    }
                    WorkerCommand::LookupPhone(p) => {
                        enforce_phone_delay(&phone_state_path).await;
                        resilient_lookup(
                            &base_dir,
                            &client,
                            &connect_params,
                            &mut login_token,
                            LookupOp::Phone(p),
                            output_dir_for(&base_dir),
                        )
                        .await
                    }
                    WorkerCommand::LookupUrl(url) => {
                        resilient_lookup(
                            &base_dir,
                            &client,
                            &connect_params,
                            &mut login_token,
                            LookupOp::Url(url),
                            output_dir_for(&base_dir),
                        )
                        .await
                    }
                    WorkerCommand::DeleteOwnMessages {
                        packed_hex,
                        chat_title,
                    } => {
                        delete_own_messages_with_progress(
                            &client,
                            packed_hex,
                            chat_title,
                            &result_tx,
                        )
                        .await
                    }
                    WorkerCommand::BuildGroupMap {
                        lookup_mode,
                        query,
                        depth,
                    } => {
                        build_group_map(&base_dir, &client, &result_tx, lookup_mode, query, depth)
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

async fn take_client(client_slot: &Arc<Mutex<Option<Client>>>) -> std::result::Result<Client, UserEvent> {
    client_slot
        .lock()
        .await
        .as_ref()
        .cloned()
        .ok_or_else(|| UserEvent::err(ErrorKind::NotConnected, ""))
}

fn is_transport_error_detail(detail: &str) -> bool {
    let d = detail.to_ascii_lowercase();
    d.contains("io failed")
        || d.contains("read error")
        || d.contains("write error")
        || d.contains("forcibly closed")
        || d.contains("connection reset")
        || d.contains("broken pipe")
        || d.contains("10054")
        || d.contains("timed out")
}

fn result_needs_reconnect(result: &OpResult) -> bool {
    match result {
        Err(UserEvent::Error { detail, .. }) => is_transport_error_detail(detail),
        Err(UserEvent::SessionLost) => true,
        _ => false,
    }
}

async fn drop_client(client_slot: &Arc<Mutex<Option<Client>>>) {
    *client_slot.lock().await = None;
}

async fn run_lookup_op(client: Client, op: LookupOp, output_dir: PathBuf) -> OpResult {
    match op {
        LookupOp::Username(u) => lookup_username(client, u, output_dir).await,
        LookupOp::Id(id) => lookup_id(client, id, output_dir).await,
        LookupOp::Phone(p) => lookup_phone(client, p, output_dir).await,
        LookupOp::Url(url) => lookup_url(client, url, output_dir).await,
    }
}

async fn resilient_lookup(
    base_dir: &Path,
    client_slot: &Arc<Mutex<Option<Client>>>,
    params: &Arc<Mutex<Option<ConnectParams>>>,
    login_token: &mut Option<grammers_client::types::LoginToken>,
    op: LookupOp,
    output_dir: PathBuf,
) -> OpResult {
    let first = run_lookup_op(take_client(client_slot).await?, op.clone(), output_dir.clone()).await;
    if !result_needs_reconnect(&first) {
        return first;
    }
    drop_client(client_slot).await;
    let Some(stored) = params.lock().await.clone() else {
        return Err(UserEvent::SessionLost);
    };
    match connect_client(
        base_dir,
        stored.api_id,
        stored.api_hash,
        &stored.phone,
        client_slot,
        login_token,
    )
    .await
    {
        Ok(UserEvent::Ready { .. }) => {}
        Ok(UserEvent::CodeSent) | Ok(UserEvent::Need2fa) => {
            return Err(UserEvent::SessionLost);
        }
        Ok(_) => {}
        Err(_) => return Err(UserEvent::SessionLost),
    }
    let second = run_lookup_op(take_client(client_slot).await?, op, output_dir).await;
    if result_needs_reconnect(&second) {
        drop_client(client_slot).await;
        Err(UserEvent::SessionLost)
    } else {
        second
    }
}

async fn validate_ready(client: &Client, base_dir: &Path) -> OpResult {
    let me = client
        .get_me()
        .await
        .map_err(|e| UserEvent::err(ErrorKind::Connect, format!("validate session: {e}")))?;
    save_session(client, base_dir)?;
    Ok(UserEvent::Ready {
        account: me.full_name(),
    })
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
    *client_slot.lock().await = None;

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
        let ready = validate_ready(&client, base_dir).await?;
        *client_slot.lock().await = Some(client);
        return Ok(ready);
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
        Ok(_) => validate_ready(&client, base_dir).await,
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
    validate_ready(&client, base_dir).await
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
        if let Some(from_contacts) = packed_from_contacts(&client, id).await {
            if usable_access_hash(from_contacts.access_hash).is_some() {
                packed = from_contacts;
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
    let (_n, p) = packed_from_dialogs_counted(client, telegram_id).await;
    p
}

/// Like [`packed_from_dialogs`], but returns how many dialog rows were scanned.
async fn packed_from_dialogs_counted(
    client: &Client,
    telegram_id: i64,
) -> (usize, Option<PackedChat>) {
    let mut scanned = 0usize;
    let mut dialogs = client.iter_dialogs();
    loop {
        let dialog = match dialogs.next().await {
            Ok(Some(d)) => d,
            Ok(None) => break,
            Err(_) => break,
        };
        scanned += 1;
        let chat = dialog.chat();
        if telegram_display_id(&chat) == telegram_id {
            return (scanned, Some(chat.pack()));
        }
    }
    (scanned, None)
}

fn chat_title_for_anchor(chat: &Chat) -> String {
    match chat {
        Chat::User(u) => u.full_name(),
        Chat::Group(g) => g.title().to_string(),
        Chat::Channel(c) => c.title().to_string(),
    }
}

/// Supergroups and small groups — both support participant iteration. (`Chat::Channel` here is broadcast-only.)
fn is_group_or_megagroup_chat(chat: &Chat) -> bool {
    matches!(chat, Chat::Group(_))
}

async fn resolve_phone_to_user_chat(client: &Client, phone: &str) -> Result<Chat, UserEvent> {
    let phone = normalize_phone_e164(phone)?;
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
        .map_err(|e| UserEvent::err(ErrorKind::ImportContacts, format!("group map import: {e}")))?;

    let users = match imported {
        tl::enums::contacts::ImportedContacts::Contacts(c) => c.users,
    };
    if users.is_empty() {
        return Err(UserEvent::err(ErrorKind::Generic, "group map: phone not found"));
    }

    let user = User::from_raw(users[0].clone());
    if let Some(input) = user.pack().try_to_input_user() {
        let _ = client
            .invoke(&tl::functions::contacts::DeleteContacts { id: vec![input] })
            .await;
    }

    Ok(Chat::User(user))
}

/// Resolve the Lookup field the same way as a profile lookup, plus `t.me/username` URLs (channel part only).
async fn resolve_group_map_anchor_chat(
    client: &Client,
    base_dir: &Path,
    lookup_mode: i32,
    query: &str,
) -> Result<(Chat, String, usize), UserEvent> {
    let q = query.trim();
    if q.is_empty() {
        return Err(UserEvent::err(
            ErrorKind::Generic,
            "group map: enter username, ID, phone, or URL in Lookup",
        ));
    }

    match lookup_mode {
        0 => {
            let name = q.trim_start_matches('@');
            let chat = client
                .resolve_username(name)
                .await
                .map_err(|e| UserEvent::err(ErrorKind::Generic, format!("{e}")))?
                .ok_or_else(|| UserEvent::err(ErrorKind::NotFoundUsername, format!("@{name}")))?;
            let title = chat_title_for_anchor(&chat);
            Ok((chat, title, 0))
        }
        1 => {
            let id: i64 = q
                .parse()
                .map_err(|_| UserEvent::err(ErrorKind::Generic, "group map: ID must be a number"))?;
            let mut packed = packed_from_numeric_id(id)
                .ok_or_else(|| UserEvent::err(ErrorKind::UnsupportedId, ""))?;
            let mut dialogs_scanned = 0usize;
            if packed_needs_access_hash(&packed) && usable_access_hash(packed.access_hash).is_none() {
                let (n, from_dialogs) = packed_from_dialogs_counted(client, id).await;
                dialogs_scanned = dialogs_scanned.saturating_add(n);
                if let Some(p) = from_dialogs {
                    if usable_access_hash(p.access_hash).is_some() {
                        packed = p;
                    }
                }
            }
            if packed_needs_access_hash(&packed) && usable_access_hash(packed.access_hash).is_none() {
                if let Some(from_contacts) = packed_from_contacts(client, id).await {
                    if usable_access_hash(from_contacts.access_hash).is_some() {
                        packed = from_contacts;
                    }
                }
            }
            if packed_needs_access_hash(&packed) && usable_access_hash(packed.access_hash).is_none() {
                return Err(UserEvent::err(ErrorKind::UnpackChat, ""));
            }
            let chat = fetch_chat_by_packed(client, packed).await?;
            let title = chat_title_for_anchor(&chat);
            Ok((chat, title, dialogs_scanned))
        }
        2 => {
            enforce_phone_delay(base_dir.join(".tguser_phone_lookup_ts").as_path()).await;
            let chat = resolve_phone_to_user_chat(client, q).await?;
            let title = chat_title_for_anchor(&chat);
            Ok((chat, title, 0))
        }
        3 => {
            let (channel, _msg_id) = parse_tme_url(q)?;
            let chat = client
                .resolve_username(&channel)
                .await
                .map_err(|e| UserEvent::err(ErrorKind::Generic, format!("{e}")))?
                .ok_or_else(|| UserEvent::err(ErrorKind::NotFoundUsername, channel))?;
            let title = chat_title_for_anchor(&chat);
            Ok((chat, title, 0))
        }
        _ => Err(UserEvent::err(ErrorKind::Generic, "group map: invalid lookup mode")),
    }
}

async fn packed_from_contacts(client: &Client, telegram_id: i64) -> Option<PackedChat> {
    user_from_contacts(client, telegram_id)
        .await
        .map(|u| u.pack())
}

async fn fetch_common_chats(client: &Client, user: &User) -> Vec<CommonChatSummary> {
    let input = match user.pack().try_to_input_user() {
        Some(i) => i,
        None => return Vec::new(),
    };
    let mut max_id = 0i64;
    let mut out = Vec::new();
    for _ in 0..20 {
        let Ok(resp) = client
            .invoke(&tl::functions::messages::GetCommonChats {
                user_id: input.clone(),
                max_id,
                limit: 100,
            })
            .await
        else {
            break;
        };
        let chats = match resp {
            tl::enums::messages::Chats::Chats(c) => c.chats,
            tl::enums::messages::Chats::Slice(c) => c.chats,
        };
        if chats.is_empty() {
            break;
        }
        let page_len = chats.len();
        let mut last_page_id = 0i64;
        for raw in chats {
            last_page_id = common_chat_tl_id(&raw);
            out.push(summarize_common_chat(Chat::from_raw(raw)));
        }
        max_id = last_page_id;
        if page_len < 100 {
            break;
        }
    }
    out
}

fn common_chat_tl_id(chat: &tl::enums::Chat) -> i64 {
    use tl::enums::Chat as C;
    match chat {
        C::Chat(c) => c.id,
        C::Channel(c) => c.id,
        C::ChannelForbidden(c) => c.id,
        C::Empty(c) => c.id,
        C::Forbidden(c) => c.id,
    }
}

fn summarize_common_chat(chat: Chat) -> CommonChatSummary {
    let (title, kind) = match &chat {
        Chat::User(u) => (u.full_name(), "user"),
        Chat::Group(g) => {
            if g.is_megagroup() {
                (g.title().to_string(), "supergroup")
            } else {
                (g.title().to_string(), "group")
            }
        }
        Chat::Channel(c) => (c.title().to_string(), "channel"),
    };
    CommonChatSummary {
        title,
        kind: kind.to_string(),
        telegram_id: telegram_display_id(&chat),
        link: chat_public_link(&chat),
        packed_hex: chat.pack().to_hex(),
    }
}

fn chat_public_link(chat: &Chat) -> Option<String> {
    if let Some(username) = match chat {
        Chat::Channel(c) => c.username(),
        Chat::Group(g) => g.username(),
        _ => None,
    } {
        return Some(format!("https://t.me/{username}"));
    }
    let display_id = telegram_display_id(chat);
    if display_id <= -TELEGRAM_CHANNEL_BASE {
        let internal = (-display_id) - TELEGRAM_CHANNEL_BASE;
        return Some(format!("https://t.me/c/{internal}"));
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
    let profile =
        describe_chat_with_phone_hint(&client, chat, &output_dir, Some(phone.clone())).await?;

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
    describe_chat_with_phone_hint(client, chat, output_dir, None).await
}

async fn describe_chat_with_phone_hint(
    client: &Client,
    chat: Chat,
    output_dir: &Path,
    phone_hint: Option<String>,
) -> OpResult {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| UserEvent::err(ErrorKind::Generic, e.to_string()))?;
    match chat {
        Chat::User(user) => describe_user(client, &user, output_dir, phone_hint).await,
        Chat::Group(group) => describe_group(client, &group, output_dir).await,
        Chat::Channel(channel) => describe_channel(client, &channel, output_dir).await,
    }
}

async fn describe_user(
    client: &Client,
    user: &User,
    output_dir: &Path,
    phone_hint: Option<String>,
) -> OpResult {
    let bundle = load_user_full_bundle(client, user).await.ok();
    let info = bundle
        .as_ref()
        .and_then(|b| b.user.as_ref())
        .unwrap_or(user);

    let bio = bundle
        .as_ref()
        .and_then(|b| b.full.about.clone())
        .unwrap_or_default();
    let common_chats_count = bundle.as_ref().map(|b| b.full.common_chats_count);
    let phone_calls_available = bundle.as_ref().map(|b| b.full.phone_calls_available);
    let video_calls_available = bundle.as_ref().map(|b| b.full.video_calls_available);

    let usernames: Vec<String> = info
        .usernames()
        .into_iter()
        .map(str::to_string)
        .collect();
    let restriction_reasons: Vec<String> = info
        .restriction_reason()
        .into_iter()
        .map(|r| format!("{}: {}", r.reason, r.text))
        .collect();

    let phone = resolve_user_phone(
        client,
        user,
        bundle.as_ref().and_then(|b| b.user.as_ref()),
        phone_hint,
    )
    .await;

    let common_chats = fetch_common_chats(client, info).await;

    let photo = download_first_profile(client, info.pack(), info.id(), output_dir).await;

    let mut profile = UserProfile {
        id: info.id(),
        first_name: info.first_name().to_string(),
        last_name: info.last_name().map(str::to_string),
        name: info.full_name(),
        username: info.username().map(str::to_string),
        usernames,
        phone,
        bio,
        status: format_user_status(info.status()),
        lang_code: info.lang_code().map(str::to_string),
        is_bot: info.is_bot(),
        verified: info.verified(),
        premium: info.raw.premium,
        scam: info.scam(),
        fake: info.raw.fake,
        deleted: info.deleted(),
        restricted: info.restricted(),
        support: info.support(),
        contact: info.contact(),
        mutual_contact: info.mutual_contact(),
        is_self: info.is_self(),
        restriction_reasons,
        common_chats_count,
        common_chats,
        phone_calls_available,
        video_calls_available,
        photo,
        json_path: None,
    };

    profile.json_path = write_user_profile_json(output_dir, &profile).ok();
    Ok(UserEvent::UserProfile(profile))
}

async fn describe_group(
    client: &Client,
    group: &Group,
    output_dir: &Path,
) -> OpResult {
    let (about, participants_count) = if group.is_megagroup() {
        megagroup_full_meta(client, group).await.unwrap_or_default()
    } else {
        basic_group_full_meta(client, group.id())
            .await
            .unwrap_or_default()
    };

    let mut profile = GroupProfile {
        id: group.id(),
        title: group.title().to_string(),
        is_megagroup: group.is_megagroup(),
        username: group.username().map(str::to_string),
        participants_count,
        about,
        json_path: None,
    };
    profile.json_path = write_group_profile_json(output_dir, &profile).ok();
    Ok(UserEvent::GroupProfile(profile))
}

async fn describe_channel(
    client: &Client,
    channel: &Channel,
    output_dir: &Path,
) -> OpResult {
    let full = load_channel_full(client, channel).await.ok();
    let about = full.as_ref().map(|f| f.about.clone()).unwrap_or_default();
    let participants_count = full.as_ref().and_then(|f| f.participants_count);
    let photo = download_first_profile(client, channel.pack(), channel.id(), output_dir).await;

    let usernames: Vec<String> = channel
        .usernames()
        .into_iter()
        .map(str::to_string)
        .collect();
    let restriction_reasons: Vec<String> = channel
        .raw
        .restriction_reason
        .as_ref()
        .map(|reasons| reasons.iter().map(restriction_reason_line).collect())
        .unwrap_or_default();

    let raw = &channel.raw;
    let mut profile = ChannelProfile {
        id: channel.id(),
        title: channel.title().to_string(),
        username: channel.username().map(str::to_string),
        usernames,
        about,
        participants_count,
        verified: raw.verified,
        scam: raw.scam,
        fake: raw.fake,
        restricted: raw.restricted,
        megagroup: raw.megagroup,
        broadcast: raw.broadcast,
        restriction_reasons,
        photo,
        json_path: None,
    };
    profile.json_path = write_channel_profile_json(output_dir, &profile).ok();
    Ok(UserEvent::ChannelProfile(profile))
}

struct UserFullBundle {
    full: tl::types::UserFull,
    user: Option<User>,
}

async fn load_user_full_bundle(client: &Client, user: &User) -> anyhow::Result<UserFullBundle> {
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
    let merged_user = f
        .users
        .into_iter()
        .map(User::from_raw)
        .find(|u| u.id() == user.id());
    Ok(UserFullBundle {
        full: u,
        user: merged_user,
    })
}

fn phone_from_user(user: &User) -> Option<String> {
    user.phone()
        .map(str::to_string)
        .filter(|p| !p.is_empty())
}

/// Walk every API source that can expose a contact's phone until one succeeds.
async fn resolve_user_phone(
    client: &Client,
    lookup_user: &User,
    full_user: Option<&User>,
    phone_hint: Option<String>,
) -> Option<String> {
    let user_id = lookup_user.id();
    let mut seen_hashes: Vec<i64> = Vec::new();

    if let Some(p) = phone_from_user(lookup_user) {
        return Some(p);
    }
    push_user_hash(&mut seen_hashes, lookup_user);

    if let Some(hash) = usable_access_hash(lookup_user.raw.access_hash) {
        if let Some(u) = fetch_user_by_id_hash(client, user_id, hash).await {
            if let Some(p) = phone_from_user(&u) {
                return Some(p);
            }
            push_user_hash(&mut seen_hashes, &u);
        }
    }

    if let Some(u) = full_user {
        if let Some(p) = phone_from_user(u) {
            return Some(p);
        }
        push_user_hash(&mut seen_hashes, u);
    }

    if let Some(p) = phone_hint.filter(|s| !s.is_empty()) {
        return Some(p);
    }

    if let Some(u) = user_from_contacts(client, user_id).await {
        if let Some(p) = phone_from_user(&u) {
            return Some(p);
        }
        push_user_hash(&mut seen_hashes, &u);
    }

    if let Some(name) = lookup_user.username() {
        if let Some(u) = user_from_contacts_search(client, name, user_id).await {
            if let Some(p) = phone_from_user(&u) {
                return Some(p);
            }
            push_user_hash(&mut seen_hashes, &u);
        }
    }

    if let Some(u) = user_from_dialogs(client, user_id).await {
        if let Some(p) = phone_from_user(&u) {
            return Some(p);
        }
        push_user_hash(&mut seen_hashes, &u);
    }

    for hash in seen_hashes.clone() {
        if let Some(u) = fetch_user_by_id_hash(client, user_id, hash).await {
            if let Some(p) = phone_from_user(&u) {
                return Some(p);
            }
            push_user_hash(&mut seen_hashes, &u);
        }
    }

    if let Some(u) = user_from_contacts(client, user_id).await {
        if let Some(hash) = usable_access_hash(u.raw.access_hash) {
            if let Some(fetched) = fetch_user_by_id_hash(client, user_id, hash).await {
                if let Some(p) = phone_from_user(&fetched) {
                    return Some(p);
                }
            }
        }
    }

    let mut dialogs = client.iter_dialogs();
    while let Ok(Some(dialog)) = dialogs.next().await {
        let Chat::User(u) = dialog.chat() else {
            continue;
        };
        if u.id() != user_id {
            continue;
        }
        if let Some(p) = phone_from_user(&u) {
            return Some(p);
        }
        if let Some(hash) = usable_access_hash(u.raw.access_hash) {
            if let Some(fetched) = fetch_user_by_id_hash(client, user_id, hash).await {
                if let Some(p) = phone_from_user(&fetched) {
                    return Some(p);
                }
            }
        }
    }

    None
}

fn push_user_hash(seen: &mut Vec<i64>, user: &User) {
    if let Some(h) = usable_access_hash(user.raw.access_hash) {
        if !seen.contains(&h) {
            seen.push(h);
        }
    }
}

async fn user_from_contacts(client: &Client, user_id: i64) -> Option<User> {
    let resp = client
        .invoke(&tl::functions::contacts::GetContacts { hash: 0 })
        .await
        .ok()?;
    let users = match resp {
        tl::enums::contacts::Contacts::Contacts(c) => c.users,
        tl::enums::contacts::Contacts::NotModified => return None,
    };
    users
        .into_iter()
        .map(User::from_raw)
        .find(|u| u.id() == user_id)
}

async fn user_from_dialogs(client: &Client, user_id: i64) -> Option<User> {
    let mut dialogs = client.iter_dialogs();
    while let Ok(Some(dialog)) = dialogs.next().await {
        if let Chat::User(u) = dialog.chat() {
            if u.id() == user_id {
                return Some(u.clone());
            }
        }
    }
    None
}

async fn user_from_contacts_search(client: &Client, query: &str, user_id: i64) -> Option<User> {
    let resp = client
        .invoke(&tl::functions::contacts::Search {
            q: query.to_string(),
            limit: 20,
        })
        .await
        .ok()?;
    let tl::enums::contacts::Found::Found(f) = resp;
    f.users
        .into_iter()
        .map(User::from_raw)
        .find(|u| u.id() == user_id)
}

async fn fetch_user_by_id_hash(client: &Client, user_id: i64, access_hash: i64) -> Option<User> {
    let users = client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::User(tl::types::InputUser {
                user_id,
                access_hash,
            })],
        })
        .await
        .ok()?;
    users
        .into_iter()
        .next()
        .map(User::from_raw)
        .filter(|u| u.id() == user_id)
}

async fn load_channel_full(
    client: &Client,
    channel: &Channel,
) -> anyhow::Result<tl::types::ChannelFull> {
    let input = channel
        .pack()
        .try_to_input_channel()
        .ok_or_else(|| anyhow!("no input channel"))?;
    let full = client
        .invoke(&tl::functions::channels::GetFullChannel { channel: input })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    let tl::enums::messages::ChatFull::Full(f) = full;
    match f.full_chat {
        tl::enums::ChatFull::ChannelFull(c) => Ok(c),
        _ => Err(anyhow!("not a channel full chat")),
    }
}

async fn basic_group_full_meta(client: &Client, chat_id: i64) -> anyhow::Result<(String, Option<i32>)> {
    let full = client
        .invoke(&tl::functions::messages::GetFullChat { chat_id })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    let tl::enums::messages::ChatFull::Full(f) = full;
    match f.full_chat {
        tl::enums::ChatFull::Full(c) => {
            let count = match c.participants {
                tl::enums::ChatParticipants::Participants(p) => Some(p.participants.len() as i32),
                tl::enums::ChatParticipants::Forbidden(p) => {
                    if p.self_participant.is_some() {
                        Some(1)
                    } else {
                        None
                    }
                }
            };
            Ok((c.about.clone(), count))
        }
        _ => Ok((String::new(), None)),
    }
}

async fn megagroup_full_meta(client: &Client, group: &Group) -> anyhow::Result<(String, Option<i32>)> {
    let input = group
        .pack()
        .try_to_input_channel()
        .ok_or_else(|| anyhow!("no input channel"))?;
    let full = client
        .invoke(&tl::functions::channels::GetFullChannel { channel: input })
        .await
        .map_err(|e| anyhow!("{e}"))?;
    let tl::enums::messages::ChatFull::Full(f) = full;
    match f.full_chat {
        tl::enums::ChatFull::ChannelFull(c) => Ok((c.about.clone(), c.participants_count)),
        _ => Ok((String::new(), None)),
    }
}

fn restriction_reason_line(r: &tl::enums::RestrictionReason) -> String {
    let tl::enums::RestrictionReason::Reason(rr) = r;
    format!("{}: {}", rr.reason, rr.text)
}

fn format_user_status(status: &tl::enums::UserStatus) -> String {
    use tl::enums::UserStatus as S;
    match status {
        S::Empty => "hidden".into(),
        S::Online(_) => "online".into(),
        S::Offline(o) => format!("offline (unix {})", o.was_online),
        S::Recently(_) => "recently".into(),
        S::LastWeek(_) => "last week".into(),
        S::LastMonth(_) => "last month".into(),
    }
}

fn photo_json(photo: &PhotoResult) -> serde_json::Value {
    match photo {
        PhotoResult::None => serde_json::Value::Null,
        PhotoResult::Saved(p) => serde_json::json!(p.display().to_string()),
        PhotoResult::Failed(e) => serde_json::json!({ "error": e }),
    }
}

fn write_user_profile_json(output_dir: &Path, p: &UserProfile) -> anyhow::Result<PathBuf> {
    let path = output_dir.join(format!("{}_user.json", p.id));
    let value = serde_json::json!({
        "type": "user",
        "id": p.id,
        "first_name": p.first_name,
        "last_name": p.last_name,
        "name": p.name,
        "username": p.username,
        "usernames": p.usernames,
        "phone": p.phone,
        "bio": p.bio,
        "status": p.status,
        "lang_code": p.lang_code,
        "is_bot": p.is_bot,
        "verified": p.verified,
        "premium": p.premium,
        "scam": p.scam,
        "fake": p.fake,
        "deleted": p.deleted,
        "restricted": p.restricted,
        "support": p.support,
        "contact": p.contact,
        "mutual_contact": p.mutual_contact,
        "is_self": p.is_self,
        "restriction_reasons": p.restriction_reasons,
        "common_chats_count": p.common_chats_count,
        "common_chats": p.common_chats.iter().map(|c| serde_json::json!({
            "title": c.title,
            "kind": c.kind,
            "telegram_id": c.telegram_id,
            "link": c.link,
            "packed_hex": c.packed_hex,
        })).collect::<Vec<_>>(),
        "phone_calls_available": p.phone_calls_available,
        "video_calls_available": p.video_calls_available,
        "photo": photo_json(&p.photo),
    });
    std::fs::write(&path, serde_json::to_string_pretty(&value)?)?;
    Ok(path)
}

fn write_group_profile_json(output_dir: &Path, p: &GroupProfile) -> anyhow::Result<PathBuf> {
    let path = output_dir.join(format!("{}_group.json", p.id));
    let value = serde_json::json!({
        "type": if p.is_megagroup { "supergroup" } else { "group" },
        "id": p.id,
        "title": p.title,
        "username": p.username,
        "participants_count": p.participants_count,
        "about": p.about,
    });
    std::fs::write(&path, serde_json::to_string_pretty(&value)?)?;
    Ok(path)
}

fn write_channel_profile_json(output_dir: &Path, p: &ChannelProfile) -> anyhow::Result<PathBuf> {
    let path = output_dir.join(format!("{}_channel.json", p.id));
    let value = serde_json::json!({
        "type": if p.broadcast { "channel" } else { "megagroup" },
        "id": p.id,
        "title": p.title,
        "username": p.username,
        "usernames": p.usernames,
        "about": p.about,
        "participants_count": p.participants_count,
        "verified": p.verified,
        "scam": p.scam,
        "fake": p.fake,
        "restricted": p.restricted,
        "megagroup": p.megagroup,
        "broadcast": p.broadcast,
        "restriction_reasons": p.restriction_reasons,
        "photo": photo_json(&p.photo),
    });
    std::fs::write(&path, serde_json::to_string_pretty(&value)?)?;
    Ok(path)
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

const DELETE_BATCH_SIZE: usize = 100;
const DELETE_PROGRESS_EVERY: u64 = 100;

async fn send_group_map_progress(
    result_tx: &mpsc::Sender<UserEvent>,
    message: &str,
    dialogs_scanned: usize,
    groups_done: usize,
    users_seen: usize,
) {
    let _ = result_tx
        .send(UserEvent::GroupMapProgress {
            message: message.to_string(),
            dialogs_scanned,
            groups_done,
            users_seen,
        })
        .await;
}

/// For depth ≥ 2: max “other” megagroups/supergroups to add via GetCommonChats per participant.
const GROUP_MAP_EXPAND_MAX_CHATS_PER_USER: usize = 24;
/// For depth ≥ 2: max participants for whom we call GetCommonChats (rest stay only on anchor edges).
const GROUP_MAP_EXPAND_MAX_USERS: usize = 120;
/// Cap total extra groups discovered at depth ≥ 2 (besides the anchor).
const GROUP_MAP_MAX_EXPAND_OTHER_CHATS: usize = 48;

async fn build_group_map(
    base_dir: &Path,
    client_slot: &Arc<Mutex<Option<Client>>>,
    result_tx: &mpsc::Sender<UserEvent>,
    lookup_mode: i32,
    query: String,
    depth: u32,
) -> OpResult {
    let client = take_client(client_slot).await?;

    let me = client
        .get_me()
        .await
        .map_err(|e| UserEvent::err(ErrorKind::Generic, format!("group map get_me: {e}")))?;
    let self_id = me.id();
    let self_name = me.full_name();

    let depth = depth.max(1).min(5);

    let mut errors: Vec<String> = Vec::new();
    let mut dialogs_scanned: usize = 0;

    send_group_map_progress(result_tx, "resolving anchor…", 0, 0, 0).await;

    let (mut anchor_chat, mut anchor_label, d_pre) =
        resolve_group_map_anchor_chat(&client, base_dir, lookup_mode, &query).await?;
    dialogs_scanned = dialogs_scanned.saturating_add(d_pre);

    // If the user picked a person (or phone lookup resolved to a user), pick one supergroup/megagroup from common chats.
    if let Chat::User(ref u) = anchor_chat {
        let commons = fetch_common_chats(&client, u).await;
        let mut candidates: Vec<Chat> = Vec::new();
        for s in commons {
            let Ok(packed) = PackedChat::from_hex(s.packed_hex.trim()) else {
                continue;
            };
            let Ok(ch) = fetch_chat_by_packed(&client, packed).await else {
                continue;
            };
            if is_group_or_megagroup_chat(&ch) {
                candidates.push(ch);
            }
        }
        candidates.sort_by(|a, b| chat_title_for_anchor(a).cmp(&chat_title_for_anchor(b)));
        let Some(ch) = candidates.into_iter().next() else {
            return Err(UserEvent::err(
                ErrorKind::Generic,
                "group map: no supergroup/megagroup in common chats with this user — open a shared group in Telegram first or enter a group/channel directly",
            ));
        };
        anchor_label = format!("{} → {}", u.full_name(), chat_title_for_anchor(&ch));
        anchor_chat = ch;
    }

    if !is_group_or_megagroup_chat(&anchor_chat) {
        return Err(UserEvent::err(
            ErrorKind::Generic,
            "group map: anchor must be a supergroup or megagroup channel (not a broadcast-only channel)",
        ));
    }

    let anchor_packed = anchor_chat.pack();
    let anchor_graph_id = telegram_display_id(&anchor_chat);
    let anchor_title = chat_title_for_anchor(&anchor_chat);

    let mut chat_titles: HashMap<i64, String> = HashMap::new();
    chat_titles.insert(anchor_graph_id, anchor_title.clone());

    let mut user_labels: HashMap<i64, String> = HashMap::new();
    user_labels.insert(self_id, self_name.clone());

    let mut participants: Vec<User> = Vec::new();
    let mut edge_keys: HashSet<(i64, i64)> = HashSet::new();
    let mut edges: Vec<GroupMapEdge> = Vec::new();

    send_group_map_progress(
        result_tx,
        &format!("participants: {anchor_title}"),
        dialogs_scanned,
        0,
        0,
    )
    .await;

    let mut p_iter = client.iter_participants(anchor_packed);
    let mut count = 0usize;
    loop {
        let next = match tokio::time::timeout(GROUP_MAP_PARTICIPANT_ITER_TIMEOUT, p_iter.next()).await
        {
            Ok(r) => r,
            Err(_) => {
                errors.push(format!(
                    "{anchor_title}: listing participants timed out after {}s (network or Telegram stalled)",
                    GROUP_MAP_PARTICIPANT_ITER_TIMEOUT.as_secs()
                ));
                break;
            }
        };
        let next = match next {
            Ok(n) => n,
            Err(e) => {
                if let Some(secs) = flood_wait_seconds(&e) {
                    let _ = send_group_map_progress(
                        result_tx,
                        &format!("FloodWait {secs}s — {anchor_title}"),
                        dialogs_scanned,
                        0,
                        user_labels.len(),
                    )
                    .await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(secs as u64 + 1)).await;
                    continue;
                }
                errors.push(format!("{anchor_title}: {e}"));
                break;
            }
        };
        let Some(part) = next else { break };
        let u = part.user;
        if u.is_bot() {
            continue;
        }
        let uid = u.id();
        let label = u.full_name();
        user_labels.insert(uid, label);
        participants.push(u);
        if edge_keys.insert((uid, anchor_graph_id)) {
            edges.push(GroupMapEdge {
                user_id: uid,
                chat_id: anchor_graph_id,
            });
        }
        count += 1;
        if count >= GROUP_MAP_MAX_PARTICIPANTS_PER_CHAT {
            break;
        }
        if count % 40 == 0 {
            let _ = send_group_map_progress(
                result_tx,
                &format!("{anchor_title} ({count})"),
                dialogs_scanned,
                0,
                user_labels.len(),
            )
            .await;
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(GROUP_MAP_API_DELAY_MS)).await;

    let mut group_ids: HashSet<i64> = HashSet::new();
    group_ids.insert(anchor_graph_id);

    let mut groups_processed: usize = 1;

    if depth >= 2 {
        let max_users = participants.len().min(GROUP_MAP_EXPAND_MAX_USERS);
        let mut extra_added: usize = 0;
        for u in participants.iter().take(max_users) {
            if extra_added >= GROUP_MAP_MAX_EXPAND_OTHER_CHATS {
                break;
            }
            send_group_map_progress(
                result_tx,
                &format!("common chats: {}", u.full_name()),
                dialogs_scanned,
                groups_processed,
                user_labels.len(),
            )
            .await;

            let commons = fetch_common_chats(&client, u).await;
            let mut added_for_user = 0usize;
            for s in commons {
                if extra_added >= GROUP_MAP_MAX_EXPAND_OTHER_CHATS {
                    break;
                }
                if added_for_user >= GROUP_MAP_EXPAND_MAX_CHATS_PER_USER {
                    break;
                }
                let Ok(packed) = PackedChat::from_hex(s.packed_hex.trim()) else {
                    continue;
                };
                let Ok(ch) = fetch_chat_by_packed(&client, packed).await else {
                    continue;
                };
                if !is_group_or_megagroup_chat(&ch) {
                    continue;
                }
                let gid = telegram_display_id(&ch);
                if gid == anchor_graph_id {
                    continue;
                }
                if group_ids.contains(&gid) {
                    continue;
                }
                group_ids.insert(gid);
                chat_titles
                    .entry(gid)
                    .or_insert_with(|| chat_title_for_anchor(&ch));
                if edge_keys.insert((u.id(), gid)) {
                    edges.push(GroupMapEdge {
                        user_id: u.id(),
                        chat_id: gid,
                    });
                }
                extra_added += 1;
                added_for_user += 1;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(GROUP_MAP_API_DELAY_MS)).await;
        }

        // Fetch participants for discovered groups (so users who share a group but were not in anchor list still appear).
        let to_scan: Vec<i64> = group_ids
            .iter()
            .copied()
            .filter(|&g| g != anchor_graph_id)
            .collect();
        for gid in to_scan {
            let Some(packed) = packed_from_numeric_id(gid) else {
                continue;
            };
            let packed = if packed_needs_access_hash(&packed) && usable_access_hash(packed.access_hash).is_none()
            {
                match packed_from_dialogs(&client, gid).await {
                    Some(p) => p,
                    None => {
                        errors.push(format!("group map: skip chat {gid} (need access hash / not in dialogs)"));
                        continue;
                    }
                }
            } else {
                packed
            };
            if packed_needs_access_hash(&packed) && usable_access_hash(packed.access_hash).is_none() {
                errors.push(format!("group map: skip chat {gid} (no access hash)"));
                continue;
            }

            let title = chat_titles.get(&gid).cloned().unwrap_or_else(|| format!("chat {gid}"));
            send_group_map_progress(
                result_tx,
                &format!("participants: {title}"),
                dialogs_scanned,
                groups_processed,
                user_labels.len(),
            )
            .await;

            let mut inner = client.iter_participants(packed);
            let mut inner_n = 0usize;
            loop {
                let next = match tokio::time::timeout(GROUP_MAP_PARTICIPANT_ITER_TIMEOUT, inner.next())
                    .await
                {
                    Ok(r) => r,
                    Err(_) => {
                        errors.push(format!(
                            "{title}: listing participants timed out after {}s",
                            GROUP_MAP_PARTICIPANT_ITER_TIMEOUT.as_secs()
                        ));
                        break;
                    }
                };
                let next = match next {
                    Ok(n) => n,
                    Err(e) => {
                        if let Some(secs) = flood_wait_seconds(&e) {
                            let _ = send_group_map_progress(
                                result_tx,
                                &format!("FloodWait {secs}s — {title}"),
                                dialogs_scanned,
                                groups_processed,
                                user_labels.len(),
                            )
                            .await;
                            tokio::time::sleep(tokio::time::Duration::from_secs(secs as u64 + 1)).await;
                            continue;
                        }
                        errors.push(format!("{title}: {e}"));
                        break;
                    }
                };
                let Some(part) = next else { break };
                let u = part.user;
                if u.is_bot() {
                    continue;
                }
                let uid = u.id();
                user_labels.insert(uid, u.full_name());
                if edge_keys.insert((uid, gid)) {
                    edges.push(GroupMapEdge {
                        user_id: uid,
                        chat_id: gid,
                    });
                }
                inner_n += 1;
                if inner_n >= GROUP_MAP_MAX_PARTICIPANTS_PER_CHAT {
                    break;
                }
            }
            groups_processed += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(GROUP_MAP_API_DELAY_MS)).await;
        }
    }

    let mut nodes: Vec<GroupMapNode> = Vec::new();
    nodes.push(GroupMapNode {
        id: self_id,
        label: self_name,
        kind: 0,
    });
    for (&uid, name) in &user_labels {
        if uid == self_id {
            continue;
        }
        nodes.push(GroupMapNode {
            id: uid,
            label: name.clone(),
            kind: 1,
        });
    }
    for &cid in &group_ids {
        let label = chat_titles
            .get(&cid)
            .cloned()
            .unwrap_or_else(|| format!("chat {cid}"));
        nodes.push(GroupMapNode {
            id: cid,
            label,
            kind: 2,
        });
    }

    let users_found = user_labels.len();

    Ok(UserEvent::GroupMapReady(GroupMapReady {
        nodes,
        edges,
        anchor_label,
        depth,
        dialogs_scanned,
        groups_processed,
        users_found,
        errors,
    }))
}

fn flood_wait_seconds(err: &InvocationError) -> Option<u32> {
    let InvocationError::Rpc(rpc) = err else {
        return None;
    };
    if !rpc.name.contains("FLOOD") {
        return None;
    }
    if let Some(v) = rpc.value {
        return Some(v);
    }
    rpc.name
        .strip_prefix("FLOOD_WAIT_")
        .and_then(|s| s.parse().ok())
}

async fn send_delete_progress(
    result_tx: &mpsc::Sender<UserEvent>,
    chat_title: &str,
    scanned: u64,
    deleted: u64,
    flood_wait_secs: Option<u32>,
) {
    let _ = result_tx
        .send(UserEvent::DeleteMessagesProgress {
            chat_title: chat_title.to_string(),
            scanned,
            deleted,
            flood_wait_secs,
        })
        .await;
}

async fn delete_batch_with_retry(
    client: &Client,
    packed: PackedChat,
    ids: &[i32],
    chat_title: &str,
    result_tx: &mpsc::Sender<UserEvent>,
    scanned: u64,
    deleted_so_far: u64,
) -> Result<usize, ()> {
    let mut attempt = 0u32;
    loop {
        match client.delete_messages(packed, ids).await {
            Ok(n) => return Ok(n),
            Err(e) => {
                if let Some(secs) = flood_wait_seconds(&e) {
                    send_delete_progress(
                        result_tx,
                        chat_title,
                        scanned,
                        deleted_so_far,
                        Some(secs),
                    )
                    .await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(secs as u64 + 1)).await;
                    attempt += 1;
                    if attempt > 100 {
                        return Err(());
                    }
                    continue;
                }
                return Err(());
            }
        }
    }
}

async fn delete_own_messages_with_progress(
    client_slot: &Arc<Mutex<Option<Client>>>,
    packed_hex: String,
    chat_title: String,
    result_tx: &mpsc::Sender<UserEvent>,
) -> OpResult {
    let client = take_client(client_slot).await?;
    let packed = PackedChat::from_hex(packed_hex.trim())
        .map_err(|_| UserEvent::err(ErrorKind::Generic, "invalid chat reference"))?;

    let mut scanned: u64 = 0;
    let mut deleted: u64 = 0;
    let mut failed_batches: u32 = 0;
    let mut pending: Vec<i32> = Vec::with_capacity(DELETE_BATCH_SIZE);

    send_delete_progress(result_tx, &chat_title, 0, 0, None).await;

    let mut iter = client.iter_messages(packed);
    loop {
        let msg = match iter.next().await {
            Ok(Some(m)) => m,
            Ok(None) => break,
            Err(e) => {
                if let Some(secs) = flood_wait_seconds(&e) {
                    send_delete_progress(
                        result_tx,
                        &chat_title,
                        scanned,
                        deleted,
                        Some(secs),
                    )
                    .await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(secs as u64 + 1)).await;
                    continue;
                }
                return Err(UserEvent::err(
                    ErrorKind::Generic,
                    format!("read history: {e}"),
                ));
            }
        };
        scanned += 1;
        if msg.outgoing() {
            pending.push(msg.id());
            if pending.len() >= DELETE_BATCH_SIZE {
                match delete_batch_with_retry(
                    &client,
                    packed,
                    &pending,
                    &chat_title,
                    result_tx,
                    scanned,
                    deleted,
                )
                .await
                {
                    Ok(n) => deleted += n as u64,
                    Err(()) => failed_batches += 1,
                }
                pending.clear();
                send_delete_progress(result_tx, &chat_title, scanned, deleted, None).await;
            }
        }
        if scanned % DELETE_PROGRESS_EVERY == 0 {
            send_delete_progress(result_tx, &chat_title, scanned, deleted, None).await;
        }
    }

    if !pending.is_empty() {
        match delete_batch_with_retry(
            &client,
            packed,
            &pending,
            &chat_title,
            result_tx,
            scanned,
            deleted,
        )
        .await
        {
            Ok(n) => deleted += n as u64,
            Err(()) => failed_batches += 1,
        }
    }

    send_delete_progress(result_tx, &chat_title, scanned, deleted, None).await;

    Ok(UserEvent::DeleteMessagesFinished {
        chat_title,
        scanned,
        deleted,
        failed_batches,
    })
}
