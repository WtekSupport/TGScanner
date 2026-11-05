use crate::i18n::{self, Lang, I18nKey};
use crate::user_event::{PhotoResult, UserProfile};
use crate::slint_app::{ProfileCardWindow, ProfileChatLine, ProfileField};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel, Weak};
use std::rc::Rc;

pub struct ProfileCardHooks {
    pub register_window: Rc<dyn Fn(Weak<ProfileCardWindow>)>,
    pub on_delete_confirmed: Rc<dyn Fn(String, String)>,
}

pub fn open_profile_card(lang: Lang, profile: &UserProfile, hooks: &ProfileCardHooks) {
    let ui = ProfileCardWindow::new().expect("profile window");
    apply_profile_strings(&ui, lang);
    populate_profile_window(&ui, lang, profile);
    (hooks.register_window)(ui.as_weak());

    let ui_weak_close = ui.as_weak();
    ui.on_open_url(|url| {
        crate::browser::open_url(url.as_str());
    });
    ui.on_close_window(move || {
        if let Some(w) = ui_weak_close.upgrade() {
            w.hide().ok();
        }
    });

    let hooks_delete = hooks.on_delete_confirmed.clone();
    let ui_weak_delete = ui.as_weak();
    ui.on_delete_click(move |hex, title| {
        if let Some(w) = ui_weak_delete.upgrade() {
            w.set_pending_delete_hex(hex);
            w.set_pending_delete_title(title);
            w.set_delete_dialog_visible(true);
        }
    });

    let ui_weak2 = ui.as_weak();
    ui.on_delete_confirmed(move |ok| {
        if !ok {
            return;
        }
        if let Some(w) = ui_weak2.upgrade() {
            let hex = w.get_pending_delete_hex().to_string();
            let title = w.get_pending_delete_title().to_string();
            if !hex.is_empty() {
                hooks_delete(hex, title);
            }
        }
    });

    ui.show().ok();
}

pub fn apply_profile_strings(ui: &ProfileCardWindow, lang: Lang) {
    ui.set_delete_hint(i18n::t(lang, I18nKey::CardDeleteHint).into());
    ui.set_delete_dialog_title(i18n::t(lang, I18nKey::CardDeleteDialogTitle).into());
    ui.set_delete_dialog_body(i18n::t(lang, I18nKey::CardDeleteDialogBody).into());
    ui.set_delete_dialog_ok(i18n::t(lang, I18nKey::DialogOk).into());
    ui.set_delete_dialog_cancel(i18n::t(lang, I18nKey::DialogCancel).into());
}

pub fn update_delete_progress(
    ui: &ProfileCardWindow,
    lang: Lang,
    chat_title: &str,
    scanned: u64,
    deleted: u64,
    flood_wait_secs: Option<u32>,
) {
    ui.set_delete_running(true);
    ui.set_progress_visible(true);
    ui.set_progress_title(
        i18n::format_delete_progress_title(lang, chat_title).into(),
    );
    ui.set_progress_scanned_label(i18n::format_delete_scanned(lang, scanned).into());
    ui.set_progress_deleted_label(i18n::format_delete_deleted(lang, deleted).into());
    ui.set_progress_wait_label(match flood_wait_secs {
        Some(s) => i18n::format_delete_flood_wait(lang, s).into(),
        None => SharedString::default(),
    });
}

pub fn finish_delete_progress(ui: &ProfileCardWindow) {
    ui.set_delete_running(false);
    ui.set_progress_visible(false);
    ui.set_progress_wait_label("".into());
}

fn populate_profile_window(ui: &ProfileCardWindow, lang: Lang, profile: &UserProfile) {
    ui.set_window_title(format!("{} — TGScanner", profile.name).into());
    ui.set_header_name(profile.name.clone().into());
    let username = profile
        .username
        .as_ref()
        .map(|u| format!("@{u}"))
        .unwrap_or_else(|| "—".to_string());
    ui.set_header_sub(format!("{username}  ·  ID {}", profile.id).into());
    ui.set_badges(i18n::format_user_badges(lang, profile).into());
    ui.set_section_details(i18n::t(lang, I18nKey::CardSectionDetails).into());
    let chat_header = if profile.common_chats.is_empty() {
        i18n::t(lang, I18nKey::CardSectionCommonChats).to_string()
    } else {
        i18n::format_common_chats_section(lang, profile.common_chats.len())
    };
    ui.set_section_chats(chat_header.into());
    ui.set_open_tme_label(i18n::t(lang, I18nKey::CardOpenTme).into());
    ui.set_close_label(i18n::t(lang, I18nKey::CardClose).into());
    ui.set_delete_running(false);
    ui.set_progress_visible(false);
    ui.set_delete_dialog_visible(false);

    if let PhotoResult::Saved(path) = &profile.photo {
        if let Ok(img) = Image::load_from_path(path) {
            ui.set_profile_photo(img);
            ui.set_has_photo(true);
        } else {
            ui.set_has_photo(false);
        }
    } else {
        ui.set_has_photo(false);
    }

    if let Some(u) = &profile.username {
        ui.set_tme_url(format!("https://t.me/{u}").into());
        ui.set_tme_enabled(true);
    } else {
        ui.set_tme_url("".into());
        ui.set_tme_enabled(false);
    }

    let fields = build_fields(lang, profile);
    ui.set_fields(ModelRc::new(VecModel::from(fields)));

    let chats = build_chats(lang, profile);
    ui.set_chats(ModelRc::new(VecModel::from(chats)));

    ui.set_json_path(
        profile
            .json_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
            .into(),
    );
}

fn build_fields(lang: Lang, profile: &UserProfile) -> Vec<ProfileField> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<ProfileField>, label: &str, value: String| {
        out.push(ProfileField {
            label: label.into(),
            value: value.into(),
        });
    };
    push(
        &mut out,
        i18n::t(lang, I18nKey::CardLabelPhone),
        i18n::display_opt(lang, &profile.phone),
    );
    push(
        &mut out,
        i18n::t(lang, I18nKey::CardLabelStatus),
        profile.status.clone(),
    );
    push(
        &mut out,
        i18n::t(lang, I18nKey::CardLabelLanguage),
        i18n::display_opt(lang, &profile.lang_code),
    );
    push(
        &mut out,
        i18n::t(lang, I18nKey::CardLabelCalls),
        i18n::format_calls_pair(lang, profile),
    );
    push(
        &mut out,
        i18n::t(lang, I18nKey::CardLabelFirstName),
        profile.first_name.clone(),
    );
    push(
        &mut out,
        i18n::t(lang, I18nKey::CardLabelLastName),
        i18n::display_opt(lang, &profile.last_name),
    );
    push(
        &mut out,
        i18n::t(lang, I18nKey::CardLabelUsernames),
        i18n::format_usernames(profile),
    );
    if !profile.bio.is_empty() {
        push(&mut out, i18n::t(lang, I18nKey::CardLabelBio), profile.bio.clone());
    }
    let flags = i18n::format_profile_flags(lang, profile);
    if !flags.is_empty() {
        push(&mut out, i18n::t(lang, I18nKey::CardLabelFlags), flags);
    }
    out
}

fn build_chats(lang: Lang, profile: &UserProfile) -> Vec<ProfileChatLine> {
    if profile.common_chats.is_empty() {
        return vec![ProfileChatLine {
            display: i18n::t(lang, I18nKey::CardCommonChatsNone).into(),
            url: SharedString::default(),
            clickable: false,
            packed_hex: SharedString::default(),
            chat_title: SharedString::default(),
            deletable: false,
        }];
    }
    profile
        .common_chats
        .iter()
        .map(|c| {
            let kind = i18n::chat_kind_label(lang, &c.kind);
            let title = c.title.clone();
            let (display, url, clickable) = match c.link.as_deref() {
                Some(url) => (
                    format!("{kind} · {title}"),
                    url.to_string(),
                    true,
                ),
                None => (
                    format!(
                        "{kind} · {title}  {}",
                        i18n::t(lang, I18nKey::CardNoPublicLink)
                    ),
                    String::new(),
                    false,
                ),
            };
            ProfileChatLine {
                display: display.into(),
                url: url.into(),
                clickable,
                packed_hex: c.packed_hex.clone().into(),
                chat_title: title.into(),
                deletable: true,
            }
        })
        .collect()
}
