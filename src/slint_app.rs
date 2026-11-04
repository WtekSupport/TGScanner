use crate::browser::URL_MY_TELEGRAM_APPS;
use crate::config::{AppConfig, ConfigLoadResult};
use crate::i18n::{self, workdir_line, Lang, I18nKey};
use crate::profile_model;
use crate::telegram::{TelegramHandle, WorkerCommand};
use crate::user_event::{ErrorKind, UserEvent};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use tokio::sync::mpsc;

slint::include_modules!();

struct SharedState {
    base_dir: PathBuf,
    telegram: TelegramHandle,
    result_rx: RefCell<mpsc::Receiver<UserEvent>>,
    busy: Cell<bool>,
    lang: Cell<Lang>,
    disclaimer_version: Cell<u32>,
    pending_phone_lookup: Cell<Option<String>>,
}

pub fn run(
    base_dir: PathBuf,
    loaded: ConfigLoadResult,
    telegram: TelegramHandle,
    result_rx: mpsc::Receiver<UserEvent>,
) {
    let config = loaded.config;
    let lang = config.lang();

    if !show_disclaimer(lang) {
        return;
    }

    let ui = MainWindow::new().expect("main window");
    let shared = Rc::new(SharedState {
        base_dir: base_dir.clone(),
        telegram,
        result_rx: RefCell::new(result_rx),
        busy: Cell::new(false),
        lang: Cell::new(lang),
        disclaimer_version: Cell::new(config.disclaimer_version),
        pending_phone_lookup: Cell::new(None),
    });

    let workdir = base_dir.display().to_string();
    apply_main_strings(&ui, lang, &workdir, &config);
    ui.set_lookup_enabled(false);
    sync_phone_mode(&ui);

    let ui_handle = ui.as_weak();
    let shared_poll = shared.clone();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(200),
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                poll_worker(&ui, &shared_poll);
            }
        },
    );

    wire_callbacks(&ui, shared.clone(), workdir.clone());

    let welcome = format!(
        "{}{}\n",
        i18n::startup_banner(lang, &workdir),
        i18n::t(lang, I18nKey::WelcomeLog)
    );
    let mut journal = welcome;
    if let Some(err) = loaded.vault_error {
        journal.push_str(&format!("{err}\n"));
    }
    ui.set_journal_text(journal.into());

    if config.connect_fields().is_some() && AppConfig::has_stored_session(&base_dir) {
        append_log(&ui, i18n::t(lang, I18nKey::AutoConnectLog));
        if let Some((api_id, api_hash, phone)) = config.connect_fields() {
            send_cmd(
                &shared,
                &ui,
                WorkerCommand::Connect {
                    api_id,
                    api_hash,
                    phone,
                },
            );
        }
    }

    ui.run().expect("Slint event loop");
}

fn show_disclaimer(lang: Lang) -> bool {
    let win = DisclaimerWindow::new().expect("disclaimer");
    win.set_window_title(i18n::window_title().into());
    win.set_title_text(i18n::t(lang, I18nKey::DisclaimerTitle).into());
    win.set_subtitle_text(i18n::t(lang, I18nKey::DisclaimerSubtitle).into());
    win.set_body_text(i18n::disclaimer_document(lang).into());
    win.set_accept_label(i18n::t(lang, I18nKey::DisclaimerBtnAccept).into());
    win.set_decline_label(i18n::t(lang, I18nKey::DisclaimerBtnDecline).into());
    win.set_checkbox_label(i18n::t(lang, I18nKey::DisclaimerAcceptCheckbox).into());
    win.set_agreed(false);

    let accepted = Rc::new(Cell::new(false));
    let acc = accepted.clone();
    let win_weak = win.as_weak();
    win.on_accepted(move || {
        acc.set(true);
        if let Some(w) = win_weak.upgrade() {
            let _ = w.hide();
        }
    });
    let acc2 = accepted.clone();
    let win_weak2 = win.as_weak();
    win.on_declined(move || {
        acc2.set(false);
        if let Some(w) = win_weak2.upgrade() {
            let _ = w.hide();
        }
    });

    win.run().expect("disclaimer loop");
    accepted.get()
}

fn apply_main_strings(ui: &MainWindow, lang: Lang, workdir: &str, config: &AppConfig) {
    ui.set_window_title(i18n::window_title().into());
    ui.set_tab_connect(i18n::t(lang, I18nKey::TabConnect).into());
    ui.set_tab_lookup(i18n::t(lang, I18nKey::TabLookup).into());
    ui.set_tab_settings(i18n::t(lang, I18nKey::TabSettings).into());
    ui.set_lbl_api_id(i18n::t(lang, I18nKey::ApiId).into());
    ui.set_lbl_api_hash(i18n::t(lang, I18nKey::ApiHash).into());
    ui.set_lbl_phone(i18n::t(lang, I18nKey::Phone).into());
    ui.set_hint_portal(i18n::t(lang, I18nKey::HintApiPortal).into());
    ui.set_btn_connect(i18n::t(lang, I18nKey::BtnConnect).into());
    ui.set_lbl_code(i18n::t(lang, I18nKey::Code).into());
    ui.set_btn_code(i18n::t(lang, I18nKey::BtnSendCode).into());
    ui.set_lbl_2fa(i18n::t(lang, I18nKey::TwoFa).into());
    ui.set_btn_password(i18n::t(lang, I18nKey::BtnSendPassword).into());
    ui.set_lbl_mode(i18n::t(lang, I18nKey::Mode).into());
    ui.set_lbl_query(i18n::t(lang, I18nKey::Query).into());
    ui.set_phone_confirm_label(i18n::t(lang, I18nKey::PhoneConfirm).into());
    ui.set_phone_risk_label(i18n::t(lang, I18nKey::PhoneRisk).into());
    ui.set_btn_lookup(i18n::t(lang, I18nKey::BtnLookup).into());
    ui.set_lbl_output(i18n::t(lang, I18nKey::OutputDir).into());
    ui.set_btn_save(i18n::t(lang, I18nKey::BtnSave).into());
    ui.set_lbl_language(i18n::t(lang, I18nKey::Language).into());
    ui.set_lbl_workdir(workdir_line(lang, workdir).into());
    ui.set_lbl_security(i18n::t(lang, I18nKey::SecuritySection).into());
    ui.set_sec_config(i18n::t(lang, I18nKey::SecurityBulletConfig).into());
    ui.set_sec_session(i18n::t(lang, I18nKey::SecurityBulletSession).into());
    ui.set_sec_output(i18n::t(lang, I18nKey::SecurityBulletOutput).into());
    ui.set_sec_unix(i18n::t(lang, I18nKey::SecurityBulletUnix).into());
    ui.set_sec_windows(i18n::t(lang, I18nKey::SecurityBulletWindows).into());
    ui.set_lbl_journal(i18n::t(lang, I18nKey::Journal).into());
    ui.set_busy_label(i18n::t(lang, I18nKey::PreloaderWorking).into());
    ui.set_lookup_modes(lookup_modes_model(lang));
    ui.set_language_options(language_model(lang));
    ui.set_phone_dialog_title(i18n::t(lang, I18nKey::PhoneDialogTitle).into());
    ui.set_phone_dialog_body(i18n::t(lang, I18nKey::PhoneDialogBody).into());
    ui.set_phone_dialog_ok(i18n::t(lang, I18nKey::DialogOk).into());
    ui.set_phone_dialog_cancel(i18n::t(lang, I18nKey::DialogCancel).into());

    ui.set_api_id(config.api_id.clone().into());
    ui.set_api_hash(config.api_hash.clone().into());
    ui.set_phone(config.phone.clone().into());
    ui.set_output_dir(config.output_dir.clone().into());
    ui.set_language_index(if lang == Lang::Ru { 1 } else { 0 });
}

fn lookup_modes_model(lang: Lang) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(vec![
        i18n::t(lang, I18nKey::LookupUsername).into(),
        i18n::t(lang, I18nKey::LookupId).into(),
        i18n::t(lang, I18nKey::LookupPhone).into(),
        i18n::t(lang, I18nKey::LookupUrl).into(),
    ]))
}

fn language_model(lang: Lang) -> ModelRc<SharedString> {
    let _ = lang;
    ModelRc::new(VecModel::from(vec![
        i18n::t(Lang::En, I18nKey::LangEn).into(),
        i18n::t(Lang::Ru, I18nKey::LangRu).into(),
    ]))
}

fn lang_from_index(idx: i32) -> Lang {
    if idx == 1 {
        Lang::Ru
    } else {
        Lang::En
    }
}

fn config_from_ui(ui: &MainWindow, disclaimer_version: u32) -> AppConfig {
    AppConfig {
        api_id: ui.get_api_id().to_string(),
        api_hash: ui.get_api_hash().to_string(),
        phone: ui.get_phone().to_string(),
        output_dir: ui.get_output_dir().to_string(),
        language: lang_from_index(ui.get_language_index()).to_config().to_string(),
        disclaimer_version,
    }
}

fn sync_phone_mode(ui: &MainWindow) {
    ui.set_phone_mode_active(ui.get_lookup_mode_index() == 2);
}

fn append_log(ui: &MainWindow, line: &str) {
    let mut s = ui.get_journal_text().to_string();
    s.push_str(line);
    if !line.ends_with('\n') {
        s.push('\n');
    }
    ui.set_journal_text(s.into());
}

fn wire_callbacks(ui: &MainWindow, shared: Rc<SharedState>, workdir: String) {
    ui.on_open_api_portal(move || {
        crate::browser::open_url(URL_MY_TELEGRAM_APPS);
    });

    let s = shared.clone();
    let ui_w = ui.as_weak();
    ui.on_connect_clicked(move || {
        if let Some(ui) = ui_w.upgrade() {
            let lang = lang_from_index(ui.get_language_index());
            s.lang.set(lang);
            let cfg = config_from_ui(&ui, s.disclaimer_version.get());
            let api_id = match cfg.parsed_api_id() {
                Ok(v) => v,
                Err(e) => {
                    append_log(&ui, &format!("{e}"));
                    return;
                }
            };
            if cfg.api_hash.trim().is_empty() || cfg.phone.trim().is_empty() {
                append_log(&ui, i18n::t(lang, I18nKey::FillHashPhone));
                return;
            }
            if let Err(e) = cfg.save(&s.base_dir) {
                append_log(&ui, &format!("{e:#}"));
            }
            send_cmd(
                &s,
                &ui,
                WorkerCommand::Connect {
                    api_id,
                    api_hash: cfg.api_hash.trim().to_string(),
                    phone: cfg.phone.trim().to_string(),
                },
            );
        }
    });

    let s = shared.clone();
    let ui_w = ui.as_weak();
    ui.on_code_clicked(move || {
        if let Some(ui) = ui_w.upgrade() {
            send_cmd(
                &s,
                &ui,
                WorkerCommand::SubmitCode(ui.get_code().to_string()),
            );
        }
    });

    let s = shared.clone();
    let ui_w = ui.as_weak();
    ui.on_password_clicked(move || {
        if let Some(ui) = ui_w.upgrade() {
            send_cmd(
                &s,
                &ui,
                WorkerCommand::SubmitPassword(ui.get_password().to_string()),
            );
        }
    });

    let s = shared.clone();
    let ui_w = ui.as_weak();
    ui.on_lookup_clicked(move || {
        if let Some(ui) = ui_w.upgrade() {
            let lang = s.lang.get();
            let trimmed = ui.get_lookup_query().trim().to_string();
            if trimmed.is_empty() {
                append_log(&ui, i18n::t(lang, I18nKey::EmptyLookup));
                return;
            }
            match ui.get_lookup_mode_index() {
                0 => send_cmd(&s, &ui, WorkerCommand::LookupUsername(trimmed)),
                1 => match trimmed.parse::<i64>() {
                    Ok(id) => send_cmd(&s, &ui, WorkerCommand::LookupId(id)),
                    Err(_) => append_log(&ui, i18n::t(lang, I18nKey::IdMustBeNumber)),
                },
                2 => {
                    if !ui.get_phone_confirm() || !ui.get_phone_risk() {
                        append_log(&ui, i18n::t(lang, I18nKey::PhoneCheckboxes));
                        return;
                    }
                    s.pending_phone_lookup.set(Some(trimmed));
                    ui.set_phone_dialog_visible(true);
                }
                3 => send_cmd(&s, &ui, WorkerCommand::LookupUrl(trimmed)),
                _ => {}
            }
        }
    });

    let s = shared.clone();
    let ui_w = ui.as_weak();
    ui.on_phone_dialog_result(move |ok| {
        if let Some(ui) = ui_w.upgrade() {
            if ok {
                if let Some(phone) = s.pending_phone_lookup.take() {
                    send_cmd(&s, &ui, WorkerCommand::LookupPhone(phone));
                }
            } else {
                s.pending_phone_lookup.set(None);
            }
        }
    });

    let s = shared.clone();
    let ui_w = ui.as_weak();
    let workdir_save = workdir.clone();
    ui.on_save_clicked(move || {
        if let Some(ui) = ui_w.upgrade() {
            let lang = lang_from_index(ui.get_language_index());
            s.lang.set(lang);
            let cfg = config_from_ui(&ui, s.disclaimer_version.get());
            if let Err(e) = cfg.save(&s.base_dir) {
                append_log(&ui, &format!("{e:#}"));
            }
            apply_main_strings(&ui, lang, &workdir_save, &cfg);
            sync_phone_mode(&ui);
            append_log(&ui, i18n::t(lang, I18nKey::SettingsSaved));
        }
    });

    let s = shared.clone();
    let ui_w = ui.as_weak();
    ui.on_language_changed(move |idx| {
        if let Some(ui) = ui_w.upgrade() {
            let lang = lang_from_index(idx);
            s.lang.set(lang);
            let cfg = config_from_ui(&ui, s.disclaimer_version.get());
            if let Err(e) = cfg.save(&s.base_dir) {
                append_log(&ui, &format!("{e:#}"));
            }
            apply_main_strings(&ui, lang, &workdir, &cfg);
            ui.set_busy_label(i18n::t(lang, I18nKey::PreloaderWorking).into());
        }
    });

    let ui_w = ui.as_weak();
    ui.on_lookup_mode_changed(move |_idx| {
        if let Some(ui) = ui_w.upgrade() {
            sync_phone_mode(&ui);
            if ui.get_lookup_mode_index() != 2 {
                ui.set_phone_confirm(false);
                ui.set_phone_risk(false);
            }
        }
    });
}

fn send_cmd(state: &SharedState, ui: &MainWindow, cmd: WorkerCommand) {
    let lang = state.lang.get();
    if state.busy.get() {
        append_log(ui, i18n::t(lang, I18nKey::Busy));
        return;
    }
    state.busy.set(true);
    ui.set_busy(true);
    if state.telegram.tx.try_send(cmd).is_err() {
        append_log(ui, i18n::t(lang, I18nKey::WorkerSendFailed));
        state.busy.set(false);
        ui.set_busy(false);
    }
}

fn poll_worker(ui: &MainWindow, state: &SharedState) {
    let lang = state.lang.get();
    while let Ok(ev) = state.result_rx.borrow_mut().try_recv() {
        match &ev {
            UserEvent::Ready { .. } => ui.set_lookup_enabled(true),
            UserEvent::SessionLost => ui.set_lookup_enabled(false),
            UserEvent::Error { kind, detail } => {
                if matches!(kind, ErrorKind::Connect | ErrorKind::NotConnected)
                    || detail.contains("IO failed")
                    || detail.contains("10054")
                {
                    ui.set_lookup_enabled(false);
                }
            }
            UserEvent::UserProfile(p) => profile_model::open_profile_card(lang, p),
            UserEvent::PhoneLookupComplete { profile, .. } => {
                if let UserEvent::UserProfile(p) = profile.as_ref() {
                    profile_model::open_profile_card(lang, p);
                }
            }
            _ => {}
        }
        append_log(ui, &i18n::format_event(lang, &ev).as_str());
        state.busy.set(false);
    }
    if !state.busy.get() {
        ui.set_busy(false);
    }
}
