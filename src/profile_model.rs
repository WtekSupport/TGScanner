use crate::i18n::{self, Lang, I18nKey};
use crate::user_event::{PhotoResult, UserProfile};
use crate::slint_app::{ProfileCardWindow, ProfileChatLine, ProfileField};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel};

pub fn open_profile_card(lang: Lang, profile: &UserProfile) {
    let ui = ProfileCardWindow::new().expect("profile window");
    populate_profile_window(&ui, lang, profile);
    let ui_weak = ui.as_weak();
    ui.on_open_url(|url| {
        crate::browser::open_url(url.as_str());
    });
    ui.on_close_window(move || {
        if let Some(w) = ui_weak.upgrade() {
            w.hide().ok();
        }
    });
    ui.show().ok();
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
        }];
    }
    profile
        .common_chats
        .iter()
        .map(|c| {
            let kind = i18n::chat_kind_label(lang, &c.kind);
            let title = c.title.clone();
            match c.link.as_deref() {
                Some(url) => ProfileChatLine {
                    display: format!("{kind} · {title}").into(),
                    url: url.into(),
                    clickable: true,
                },
                None => ProfileChatLine {
                    display: format!(
                        "{kind} · {title}  {}",
                        i18n::t(lang, I18nKey::CardNoPublicLink)
                    )
                    .into(),
                    url: SharedString::default(),
                    clickable: false,
                },
            }
        })
        .collect()
}
