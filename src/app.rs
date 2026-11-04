use crate::browser::{self, URL_MY_TELEGRAM_APPS};
use crate::busy_ui::{BusyOverlay, OperationTargets};
use crate::config::{AppConfig, ConfigLoadResult};
use crate::disclaimer;
use crate::i18n::{self, workdir_line, Lang, I18nKey};
use crate::telegram::{TelegramHandle, WorkerCommand};
use crate::ui_style::{
    connect_field_w, field_w, field_x, style_check, style_choice,
    style_field_hint_link, style_form_label, style_hint_display, style_log_display,
    style_primary_button, style_secret_input, style_secondary_button, style_section_title,
    style_subtitle, style_tab_page, style_tabs, style_text_input, style_window, GAP, HINT_BTN_W,
    IN_H, LBL_W, M, ROW, WIN_H, WIN_W,
};
use crate::user_event::UserEvent;
use fltk::{
    app,
    button::{Button, CheckButton},
    dialog,
    enums::Align,
    frame::Frame,
    group::{Group, Tabs},
    input::{Input, SecretInput},
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use tokio::sync::mpsc;

struct SharedState {
    base_dir: PathBuf,
    telegram: TelegramHandle,
    result_rx: RefCell<mpsc::Receiver<UserEvent>>,
    busy: Cell<bool>,
    lang: Cell<Lang>,
    disclaimer_version: Cell<u32>,
}

#[allow(dead_code)]
struct LocaleLabels {
    wind: Window,
    tab_connect: Group,
    tab_lookup: Group,
    tab_settings: Group,
    lbl_api_id: Frame,
    lbl_api_hash: Frame,
    hint_api_id: Button,
    hint_api_hash: Button,
    hint_phone: Button,
    lbl_phone: Frame,
    btn_connect: Button,
    lbl_code: Frame,
    btn_code: Button,
    lbl_2fa: Frame,
    btn_password: Button,
    connect_steps: TextDisplay,
    steps_buf: TextBuffer,
    lbl_mode: Frame,
    lbl_query: Frame,
    phone_confirm: CheckButton,
    phone_risk: CheckButton,
    btn_lookup: Button,
    lbl_output: Frame,
    btn_save: Button,
    lbl_language: Frame,
    lbl_workdir: Frame,
    lbl_security: Frame,
    sec_config: Frame,
    sec_session: Frame,
    sec_output: Frame,
    sec_unix: Frame,
    sec_windows: Frame,
    lbl_journal: Frame,
}

struct Widgets {
    api_id: Input,
    api_hash: SecretInput,
    phone: Input,
    code: Input,
    password: SecretInput,
    lookup_mode: Choice,
    lookup_input: Input,
    phone_confirm: CheckButton,
    phone_risk: CheckButton,
    output_dir: Input,
    language: Choice,
    log: Rc<RefCell<TextBuffer>>,
}

fn log_msg(log: &Rc<RefCell<TextBuffer>>, line: &str) {
    let mut buffer = log.borrow_mut();
    buffer.append(line);
    if !line.ends_with('\n') {
        buffer.append("\n");
    }
}

fn apply_field_hint_labels(labels: &mut LocaleLabels, lang: Lang) {
    let portal = i18n::t(lang, I18nKey::HintApiPortal);
    let tip_api = i18n::t(lang, I18nKey::TooltipApiPortal);
    let tip_phone = i18n::t(lang, I18nKey::TooltipPhone);
    labels.hint_api_id.set_label(portal);
    labels.hint_api_hash.set_label(portal);
    labels.hint_api_id.set_tooltip(tip_api);
    labels.hint_api_hash.set_tooltip(tip_api);
    labels.hint_phone.set_label("?");
    labels.hint_phone.set_tooltip(tip_phone);
}

fn apply_locale(labels: &mut LocaleLabels, lang: Lang, workdir: &str) {
    labels.wind.set_label(i18n::t(lang, I18nKey::WindowTitle));
    apply_field_hint_labels(labels, lang);
    labels.tab_connect.set_label(i18n::t(lang, I18nKey::TabConnect));
    labels.tab_lookup.set_label(i18n::t(lang, I18nKey::TabLookup));
    labels.tab_settings.set_label(i18n::t(lang, I18nKey::TabSettings));
    labels.lbl_api_id.set_label(i18n::t(lang, I18nKey::ApiId));
    labels.lbl_api_hash.set_label(i18n::t(lang, I18nKey::ApiHash));
    labels.lbl_phone.set_label(i18n::t(lang, I18nKey::Phone));
    labels.btn_connect.set_label(i18n::t(lang, I18nKey::BtnConnect));
    labels.lbl_code.set_label(i18n::t(lang, I18nKey::Code));
    labels.btn_code.set_label(i18n::t(lang, I18nKey::BtnSendCode));
    labels.lbl_2fa.set_label(i18n::t(lang, I18nKey::TwoFa));
    labels.btn_password.set_label(i18n::t(lang, I18nKey::BtnSendPassword));
    labels
        .steps_buf
        .set_text(i18n::t(lang, I18nKey::ConnectSteps));
    labels.lbl_mode.set_label(i18n::t(lang, I18nKey::Mode));
    labels.lbl_query.set_label(i18n::t(lang, I18nKey::Query));
    labels.phone_confirm.set_label(i18n::t(lang, I18nKey::PhoneConfirm));
    labels.phone_risk.set_label(i18n::t(lang, I18nKey::PhoneRisk));
    labels.btn_lookup.set_label(i18n::t(lang, I18nKey::BtnLookup));
    labels.lbl_output.set_label(i18n::t(lang, I18nKey::OutputDir));
    labels.btn_save.set_label(i18n::t(lang, I18nKey::BtnSave));
    labels.lbl_language.set_label(i18n::t(lang, I18nKey::Language));
    labels.lbl_workdir.set_label(&workdir_line(lang, workdir));
    labels.lbl_security.set_label(i18n::t(lang, I18nKey::SecuritySection));
    labels.sec_config.set_label(i18n::t(lang, I18nKey::SecurityBulletConfig));
    labels.sec_session.set_label(i18n::t(lang, I18nKey::SecurityBulletSession));
    labels.sec_output.set_label(i18n::t(lang, I18nKey::SecurityBulletOutput));
    labels.sec_unix.set_label(i18n::t(lang, I18nKey::SecurityBulletUnix));
    labels.sec_windows.set_label(i18n::t(lang, I18nKey::SecurityBulletWindows));
    labels.lbl_journal.set_label(i18n::t(lang, I18nKey::Journal));
}

fn refill_lookup_modes(choice: &mut Choice, lang: Lang) {
    choice.clear();
    choice.add_choice(&format!(
        "{}|{}|{}|{}",
        i18n::t(lang, I18nKey::LookupUsername),
        i18n::t(lang, I18nKey::LookupId),
        i18n::t(lang, I18nKey::LookupPhone),
        i18n::t(lang, I18nKey::LookupUrl),
    ));
}

fn lang_from_choice(choice: &Choice) -> Lang {
    if choice.value() == 1 {
        Lang::Ru
    } else {
        Lang::En
    }
}

fn config_from_widgets(w: &Widgets, disclaimer_version: u32) -> AppConfig {
    let lang = lang_from_choice(&w.language);
    AppConfig {
        api_id: w.api_id.value(),
        api_hash: w.api_hash.value(),
        phone: w.phone.value(),
        output_dir: w.output_dir.value(),
        language: lang.to_config().to_string(),
        disclaimer_version,
    }
}

fn persist_widgets_config(
    state: &SharedState,
    w: &Widgets,
    log: &Rc<RefCell<TextBuffer>>,
) -> Lang {
    let lang = lang_from_choice(&w.language);
    state.lang.set(lang);
    let cfg = config_from_widgets(w, state.disclaimer_version.get());
    if let Err(e) = cfg.save(&state.base_dir) {
        log_msg(log, &format!("{e:#}"));
    }
    lang
}

fn hint_x(field_x: i32, input_w: i32) -> i32 {
    field_x + input_w + GAP
}

fn ask_phone_lookup(lang: Lang) -> bool {
    let body = format!(
        "{}\n\n{}",
        i18n::t(lang, I18nKey::PhoneDialogTitle),
        i18n::t(lang, I18nKey::PhoneDialogBody)
    );
    dialog::choice2_default(
        &body,
        i18n::t(lang, I18nKey::DialogOk),
        i18n::t(lang, I18nKey::DialogCancel),
        "",
    ) == Some(0)
}

pub fn run(
    base_dir: PathBuf,
    loaded: ConfigLoadResult,
    telegram: TelegramHandle,
    result_rx: mpsc::Receiver<UserEvent>,
) {
    let config = loaded.config;
    let lang = config.lang();

    let _app = app::App::default().with_scheme(app::Scheme::Base);
    if !disclaimer::show_startup_disclaimer(lang) {
        return;
    }

    let shared = Rc::new(SharedState {
        base_dir: base_dir.clone(),
        telegram,
        result_rx: RefCell::new(result_rx),
        busy: Cell::new(false),
        lang: Cell::new(lang),
        disclaimer_version: Cell::new(config.disclaimer_version),
    });

    let field_x = field_x();
    let field_w = field_w();
    let connect_w = connect_field_w();

    let mut wind = Window::new(80, 80, WIN_W, WIN_H, i18n::t(lang, I18nKey::WindowTitle));
    style_window(&mut wind);

    let tabs_top = M;
    let tab_h = 430;
    let tab_y = tabs_top + 28;
    const CON_ROW: i32 = 32;
    let mut tabs = Tabs::new(M - 2, tabs_top, WIN_W - 2 * M + 4, tab_h + 32, None);
    style_tabs(&mut tabs);

    // --- Connection tab ---
    let mut tab_connect = Group::new(M - 2, tab_y, WIN_W - 2 * M + 4, tab_h, i18n::t(lang, I18nKey::TabConnect));
    style_tab_page(&mut tab_connect);

    let mut y = tab_y + 10;

    let mut lbl_api_id = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::ApiId));
    style_form_label(&mut lbl_api_id);
    let mut api_id = Input::new(field_x, y - 1, connect_w, IN_H, None);
    style_text_input(&mut api_id);
    api_id.set_value(&config.api_id);
    let mut hint_api_id = Button::new(
        hint_x(field_x, connect_w),
        y - 1,
        HINT_BTN_W,
        IN_H,
        i18n::t(lang, I18nKey::HintApiPortal),
    );
    style_field_hint_link(&mut hint_api_id);
    hint_api_id.set_tooltip(i18n::t(lang, I18nKey::TooltipApiPortal));
    y += CON_ROW;

    let mut lbl_api_hash = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::ApiHash));
    style_form_label(&mut lbl_api_hash);
    let mut api_hash = SecretInput::new(field_x, y - 1, connect_w, IN_H, None);
    style_secret_input(&mut api_hash);
    api_hash.set_value(&config.api_hash);
    let mut hint_api_hash = Button::new(
        hint_x(field_x, connect_w),
        y - 1,
        HINT_BTN_W,
        IN_H,
        i18n::t(lang, I18nKey::HintApiPortal),
    );
    style_field_hint_link(&mut hint_api_hash);
    hint_api_hash.set_tooltip(i18n::t(lang, I18nKey::TooltipApiPortal));
    y += CON_ROW;

    let mut lbl_phone = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::Phone));
    style_form_label(&mut lbl_phone);
    let mut phone = Input::new(field_x, y - 1, connect_w, IN_H, None);
    style_text_input(&mut phone);
    phone.set_value(&config.phone);
    let mut hint_phone = Button::new(hint_x(field_x, connect_w), y - 1, HINT_BTN_W, IN_H, "?");
    style_field_hint_link(&mut hint_phone);
    hint_phone.set_tooltip(i18n::t(lang, I18nKey::TooltipPhone));
    y += CON_ROW + 6;

    let mut btn_connect = Button::new(field_x, y, field_w, 32, i18n::t(lang, I18nKey::BtnConnect));
    style_primary_button(&mut btn_connect);
    y += 38;

    let mut connect_steps = TextDisplay::new(field_x, y, field_w, 58, None);
    style_hint_display(&mut connect_steps);
    let mut steps_buf = TextBuffer::default();
    steps_buf.set_text(i18n::t(lang, I18nKey::ConnectSteps));
    connect_steps.set_buffer(Some(steps_buf.clone()));
    y += 64;

    let mut lbl_code = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::Code));
    style_form_label(&mut lbl_code);
    let mut code = Input::new(field_x, y - 1, 220, IN_H, None);
    style_text_input(&mut code);
    let mut btn_code = Button::new(field_x + 228, y - 2, 148, 32, i18n::t(lang, I18nKey::BtnSendCode));
    style_secondary_button(&mut btn_code);
    y += CON_ROW;

    let mut lbl_2fa = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::TwoFa));
    style_form_label(&mut lbl_2fa);
    let mut password = SecretInput::new(field_x, y - 1, 220, IN_H, None);
    style_secret_input(&mut password);
    let mut btn_password = Button::new(field_x + 228, y - 2, 148, 32, i18n::t(lang, I18nKey::BtnSendPassword));
    style_secondary_button(&mut btn_password);

    tab_connect.end();

    // --- Lookup tab ---
    let mut tab_lookup = Group::new(M - 2, tab_y, WIN_W - 2 * M + 4, tab_h, i18n::t(lang, I18nKey::TabLookup));
    style_tab_page(&mut tab_lookup);
    tab_lookup.hide();

    y = tab_y + 18;
    let mut lbl_mode = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::Mode));
    style_form_label(&mut lbl_mode);
    let mut lookup_mode = Choice::new(field_x, y - 1, 260, IN_H, None);
    style_choice(&mut lookup_mode);
    refill_lookup_modes(&mut lookup_mode, lang);
    y += ROW;

    let mut lbl_query = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::Query));
    style_form_label(&mut lbl_query);
    let mut lookup_input = Input::new(field_x, y - 1, field_w, IN_H, None);
    style_text_input(&mut lookup_input);
    y += ROW + 10;

    let mut phone_confirm =
        CheckButton::new(field_x, y, field_w, 26, i18n::t(lang, I18nKey::PhoneConfirm));
    style_check(&mut phone_confirm);
    y += 32;
    let mut phone_risk = CheckButton::new(field_x, y, field_w, 26, i18n::t(lang, I18nKey::PhoneRisk));
    style_check(&mut phone_risk);
    phone_confirm.deactivate();
    phone_risk.deactivate();
    y += 44;

    let mut btn_lookup = Button::new(field_x, y, 240, 34, i18n::t(lang, I18nKey::BtnLookup));
    style_primary_button(&mut btn_lookup);

    tab_lookup.end();

    // --- Settings tab ---
    let mut tab_settings = Group::new(M - 2, tab_y, WIN_W - 2 * M + 4, tab_h, i18n::t(lang, I18nKey::TabSettings));
    style_tab_page(&mut tab_settings);
    tab_settings.hide();

    y = tab_y + 18;
    let mut lbl_language = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::Language));
    style_form_label(&mut lbl_language);
    let mut language = Choice::new(field_x, y - 1, 240, IN_H, None);
    language.add_choice(&format!(
        "{}|{}",
        i18n::t(Lang::En, I18nKey::LangEn),
        i18n::t(Lang::Ru, I18nKey::LangRu)
    ));
    style_choice(&mut language);
    language.set_value(match config.lang() {
        Lang::En => 0,
        Lang::Ru => 1,
    });
    y += ROW;

    let mut lbl_output = Frame::new(M + 2, y, LBL_W, 24, i18n::t(lang, I18nKey::OutputDir));
    style_form_label(&mut lbl_output);
    let mut output_dir = Input::new(field_x, y - 1, field_w - 124, IN_H, None);
    style_text_input(&mut output_dir);
    output_dir.set_value(&config.output_dir);
    let mut btn_save = Button::new(field_x + field_w - 116, y - 2, 116, 32, i18n::t(lang, I18nKey::BtnSave));
    style_secondary_button(&mut btn_save);
    y += ROW + 8;

    let workdir_str = base_dir.display().to_string();
    let workdir_line_init = workdir_line(lang, &workdir_str);
    let mut lbl_workdir = Frame::new(M + 2, y, WIN_W - 2 * M - 4, 22, workdir_line_init.as_str());
    lbl_workdir.set_align(Align::Left | Align::Inside);
    style_subtitle(&mut lbl_workdir);
    y += 34;

    let mut lbl_security = Frame::new(M + 2, y, WIN_W - 2 * M - 4, 22, i18n::t(lang, I18nKey::SecuritySection));
    style_section_title(&mut lbl_security);
    y += 28;
    let mut sec_config = Frame::new(M + 10, y, WIN_W - 2 * M - 12, 18, i18n::t(lang, I18nKey::SecurityBulletConfig));
    sec_config.set_align(Align::Left | Align::Inside);
    style_subtitle(&mut sec_config);
    y += 22;
    let mut sec_session = Frame::new(M + 10, y, WIN_W - 2 * M - 12, 18, i18n::t(lang, I18nKey::SecurityBulletSession));
    sec_session.set_align(Align::Left | Align::Inside);
    style_subtitle(&mut sec_session);
    y += 22;
    let mut sec_output = Frame::new(M + 10, y, WIN_W - 2 * M - 12, 18, i18n::t(lang, I18nKey::SecurityBulletOutput));
    sec_output.set_align(Align::Left | Align::Inside);
    style_subtitle(&mut sec_output);
    y += 22;
    let mut sec_unix = Frame::new(M + 10, y, WIN_W - 2 * M - 12, 18, i18n::t(lang, I18nKey::SecurityBulletUnix));
    sec_unix.set_align(Align::Left | Align::Inside);
    style_subtitle(&mut sec_unix);
    y += 22;
    let mut sec_windows = Frame::new(M + 10, y, WIN_W - 2 * M - 12, 18, i18n::t(lang, I18nKey::SecurityBulletWindows));
    sec_windows.set_align(Align::Left | Align::Inside);
    style_subtitle(&mut sec_windows);

    tab_settings.end();
    tabs.end();

    let log_top = tabs_top + tab_h + 44;
    let mut lbl_journal = Frame::new(M, log_top, WIN_W - 2 * M, 22, i18n::t(lang, I18nKey::Journal));
    style_section_title(&mut lbl_journal);
    let mut log_buffer = TextBuffer::default();
    let mut welcome = format!("{}\n", i18n::t(lang, I18nKey::WelcomeLog));
    if let Some(err) = loaded.vault_error {
        welcome.push_str(&format!("{err}\n"));
    }
    log_buffer.set_text(&welcome);
    let log_h = WIN_H - log_top - 34;
    let mut log_view = TextDisplay::new(M, log_top + 26, WIN_W - 2 * M, log_h, None);
    log_view.set_buffer(Some(log_buffer.clone()));
    style_log_display(&mut log_view);

    let log = Rc::new(RefCell::new(log_buffer));

    let busy_ui = Rc::new(RefCell::new(BusyOverlay::new()));
    let op_targets = Rc::new(RefCell::new(OperationTargets {
        wind: wind.clone(),
        tabs: tabs.clone(),
        btn_connect: btn_connect.clone(),
        btn_code: btn_code.clone(),
        btn_password: btn_password.clone(),
        btn_lookup: btn_lookup.clone(),
        btn_save: btn_save.clone(),
        lookup_mode: lookup_mode.clone(),
    }));

    hint_api_id.set_callback(|_| browser::open_url(URL_MY_TELEGRAM_APPS));
    hint_api_hash.set_callback(|_| browser::open_url(URL_MY_TELEGRAM_APPS));
    hint_phone.set_callback({
        let state = shared.clone();
        move |_| {
            let lang = state.lang.get();
            dialog::message_default(i18n::t(lang, I18nKey::TooltipPhone));
        }
    });

    let labels = Rc::new(RefCell::new(LocaleLabels {
        wind,
        tab_connect,
        tab_lookup,
        tab_settings,
        lbl_api_id,
        lbl_api_hash,
        hint_api_id,
        hint_api_hash,
        hint_phone,
        lbl_phone,
        btn_connect: btn_connect.clone(),
        lbl_code,
        btn_code: btn_code.clone(),
        lbl_2fa,
        btn_password: btn_password.clone(),
        connect_steps,
        steps_buf,
        lbl_mode,
        lbl_query,
        phone_confirm: phone_confirm.clone(),
        phone_risk: phone_risk.clone(),
        btn_lookup: btn_lookup.clone(),
        lbl_output,
        btn_save: btn_save.clone(),
        lbl_language,
        lbl_workdir,
        lbl_security,
        sec_config,
        sec_session,
        sec_output,
        sec_unix,
        sec_windows,
        lbl_journal,
    }));

    let widgets = Rc::new(Widgets {
        api_id,
        api_hash,
        phone,
        code,
        password,
        lookup_mode: lookup_mode.clone(),
        lookup_input,
        phone_confirm,
        phone_risk,
        output_dir,
        language: language.clone(),
        log: log.clone(),
    });

    fn send_cmd(
        state: &SharedState,
        log: &Rc<RefCell<TextBuffer>>,
        busy_ui: &Rc<RefCell<BusyOverlay>>,
        targets: &Rc<RefCell<OperationTargets>>,
        cmd: WorkerCommand,
    ) {
        let lang = state.lang.get();
        if state.busy.get() {
            log_msg(log, i18n::t(lang, I18nKey::Busy));
            return;
        }
        state.busy.set(true);
        if state.telegram.tx.try_send(cmd).is_err() {
            log_msg(log, i18n::t(lang, I18nKey::WorkerSendFailed));
            state.busy.set(false);
            return;
        }
        busy_ui.borrow_mut().show(lang, &mut targets.borrow_mut());
        let _ = app::flush();
    }

    lookup_mode.set_callback({
        let w = widgets.clone();
        move |c| {
            let on_phone = c.value() == 2;
            let mut pc = w.phone_confirm.clone();
            let mut pr = w.phone_risk.clone();
            if on_phone {
                pc.activate();
                pr.activate();
            } else {
                pc.deactivate();
                pr.deactivate();
                pc.set_value(false);
                pr.set_value(false);
            }
        }
    });

    let ui_ready = Rc::new(Cell::new(false));

    {
        let state = shared.clone();
        let w = widgets.clone();
        let labels2 = labels.clone();
        let workdir = workdir_str.clone();
        let ui_ready2 = ui_ready.clone();
        let busy_ui_lang = busy_ui.clone();
        language.set_callback(move |choice| {
            if !ui_ready2.get() {
                return;
            }
            let choice2 = choice.clone();
            let state2 = state.clone();
            let w2 = w.clone();
            let labels3 = labels2.clone();
            let workdir2 = workdir.clone();
            let busy_ui_timeout = busy_ui_lang.clone();
            app::add_timeout3(0.0, move |_| {
                let new_lang = lang_from_choice(&choice2);
                state2.lang.set(new_lang);
                let cfg = config_from_widgets(&w2, state2.disclaimer_version.get());
                if let Err(e) = cfg.save(&state2.base_dir) {
                    log_msg(&w2.log, &format!("{e:#}"));
                }
                apply_locale(&mut labels3.borrow_mut(), new_lang, &workdir2);
                let mut mode = w2.lookup_mode.clone();
                refill_lookup_modes(&mut mode, new_lang);
                busy_ui_timeout.borrow_mut().refresh_label(new_lang);
            });
        });
    }

    {
        let state = shared.clone();
        let w = widgets.clone();
        let busy = busy_ui.clone();
        let targets = op_targets.clone();
        btn_connect.set_callback(move |_| {
            let lang = lang_from_choice(&w.language);
            state.lang.set(lang);
            let cfg = config_from_widgets(&w, state.disclaimer_version.get());
            let api_id = match cfg.parsed_api_id() {
                Ok(v) => v,
                Err(e) => {
                    log_msg(&w.log, &format!("{e}"));
                    return;
                }
            };
            if cfg.api_hash.trim().is_empty() || cfg.phone.trim().is_empty() {
                log_msg(&w.log, i18n::t(lang, I18nKey::FillHashPhone));
                return;
            }
            if let Err(e) = cfg.save(&state.base_dir) {
                log_msg(&w.log, &format!("{e:#}"));
            }
            send_cmd(
                &state,
                &w.log,
                &busy,
                &targets,
                WorkerCommand::Connect {
                    api_id,
                    api_hash: cfg.api_hash.trim().to_string(),
                    phone: cfg.phone.trim().to_string(),
                },
            );
        });
    }

    {
        let state = shared.clone();
        let w = widgets.clone();
        let busy = busy_ui.clone();
        let targets = op_targets.clone();
        btn_code.set_callback(move |_| {
            send_cmd(
                &state,
                &w.log,
                &busy,
                &targets,
                WorkerCommand::SubmitCode(w.code.value()),
            );
        });
    }

    {
        let state = shared.clone();
        let w = widgets.clone();
        let busy = busy_ui.clone();
        let targets = op_targets.clone();
        btn_password.set_callback(move |_| {
            send_cmd(
                &state,
                &w.log,
                &busy,
                &targets,
                WorkerCommand::SubmitPassword(w.password.value()),
            );
        });
    }

    {
        let state = shared.clone();
        let w = widgets.clone();
        let busy = busy_ui.clone();
        let targets = op_targets.clone();
        btn_lookup.set_callback(move |_| {
            let lang = state.lang.get();
            let input = w.lookup_input.value();
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                log_msg(&w.log, i18n::t(lang, I18nKey::EmptyLookup));
                return;
            }
            match w.lookup_mode.value() {
                0 => send_cmd(
                    &state,
                    &w.log,
                    &busy,
                    &targets,
                    WorkerCommand::LookupUsername(trimmed),
                ),
                1 => match trimmed.parse::<i64>() {
                    Ok(id) => send_cmd(
                        &state,
                        &w.log,
                        &busy,
                        &targets,
                        WorkerCommand::LookupId(id),
                    ),
                    Err(_) => log_msg(&w.log, i18n::t(lang, I18nKey::IdMustBeNumber)),
                },
                2 => {
                    if !w.phone_confirm.value() || !w.phone_risk.value() {
                        log_msg(&w.log, i18n::t(lang, I18nKey::PhoneCheckboxes));
                        return;
                    }
                    if !ask_phone_lookup(lang) {
                        return;
                    }
                    send_cmd(
                        &state,
                        &w.log,
                        &busy,
                        &targets,
                        WorkerCommand::LookupPhone(trimmed),
                    );
                }
                3 => send_cmd(
                    &state,
                    &w.log,
                    &busy,
                    &targets,
                    WorkerCommand::LookupUrl(trimmed),
                ),
                _ => {}
            }
        });
    }

    {
        let state = shared.clone();
        let w = widgets.clone();
        let labels2 = labels.clone();
        let workdir = workdir_str.clone();
        btn_save.set_callback(move |_| {
            let lang = persist_widgets_config(&state, &w, &w.log);
            log_msg(&w.log, i18n::t(lang, I18nKey::SettingsSaved));
            apply_locale(&mut labels2.borrow_mut(), lang, &workdir);
            let mut mode = w.lookup_mode.clone();
            refill_lookup_modes(&mut mode, lang);
        });
    }

    labels.borrow_mut().wind.end();
    ui_ready.set(true);
    labels.borrow_mut().wind.show();

    fn schedule_poll(
        state: Rc<SharedState>,
        log: Rc<RefCell<TextBuffer>>,
        busy_ui: Rc<RefCell<BusyOverlay>>,
        targets: Rc<RefCell<OperationTargets>>,
    ) {
        let state2 = state.clone();
        let log2 = log.clone();
        let busy2 = busy_ui.clone();
        let targets2 = targets.clone();
        app::add_timeout3(0.2, move |_| {
            let lang = state2.lang.get();
            while let Ok(ev) = state2.result_rx.borrow_mut().try_recv() {
                log_msg(&log2, &i18n::format_event(lang, &ev));
                state2.busy.set(false);
            }
            if state2.busy.get() {
                busy2.borrow_mut().tick();
            } else {
                busy2.borrow_mut().hide(&mut targets2.borrow_mut());
            }
            schedule_poll(state2.clone(), log2.clone(), busy2.clone(), targets2.clone());
        });
    }

    schedule_poll(shared, log, busy_ui, op_targets);
    let _ = app::run();
}
