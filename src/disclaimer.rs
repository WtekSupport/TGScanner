use crate::i18n::{self, Lang, I18nKey};
use crate::ui_style::{
    style_check_on_surface, style_disclaimer_agree_text, style_disclaimer_display,
    style_disclaimer_panel, style_divider, style_danger_button, style_primary_button,
    style_subtitle, style_title, style_window, C_BORDER, M, WIN_W,
};
use fltk::enums::{Align, FrameType};
use fltk::{app, button::Button, frame::Frame, prelude::*};
use fltk::{
    button::CheckButton,
    text::{TextBuffer, TextDisplay},
    window::Window,
};
use std::cell::Cell;
use std::rc::Rc;

const DW: i32 = 720;
const DH: i32 = 640;
const PAD: i32 = 20;
const FOOTER_H: i32 = 158;
const BTN_H: i32 = 36;
const BTN_GAP: i32 = 12;
const AGREE_ROW_H: i32 = 76;
const CHK: i32 = 18;
const HEADER_H: i32 = 88;

/// Modal disclaimer in the same dark theme as the main app. Returns `true` if the user accepts.
pub fn show_startup_disclaimer(lang: Lang) -> bool {
    let _ = app::set_scrollbar_size(11);

    let accepted = Rc::new(Cell::new(false));
    let mut win = Window::new(120, 60, DW, DH, i18n::t(lang, I18nKey::WindowTitle));
    style_window(&mut win);

    let inner_w = DW - 2 * M;
    let inner_x = M;

    let mut title = Frame::new(inner_x, PAD, inner_w, 24, i18n::t(lang, I18nKey::DisclaimerTitle));
    style_title(&mut title);
    title.set_label_size(14);
    title.set_align(Align::Left | Align::Inside);

    let mut subtitle = Frame::new(
        inner_x,
        PAD + 26,
        inner_w,
        32,
        i18n::t(lang, I18nKey::DisclaimerSubtitle),
    );
    style_subtitle(&mut subtitle);
    subtitle.set_align(Align::Left | Align::Top | Align::Inside | Align::Wrap);

    let mut rule = Frame::new(inner_x, HEADER_H - 2, inner_w, 2, "");
    style_divider(&mut rule);

    let footer_y = DH - PAD - FOOTER_H;
    let text_top = HEADER_H + 8;
    let text_h = footer_y - text_top - 10;
    let text_x = inner_x;
    let text_w = inner_w;

    let mut text_border = Frame::new(text_x, text_top, text_w, text_h, None);
    text_border.set_frame(FrameType::FlatBox);
    text_border.set_color(C_BORDER);

    let mut display = TextDisplay::new(text_x + 1, text_top + 1, text_w - 2, text_h - 2, None);
    style_disclaimer_display(&mut display);
    let mut buf = TextBuffer::default();
    buf.set_text(&format!("\n{}\n", i18n::disclaimer_document(lang)));
    display.set_buffer(Some(buf));

    let mut footer = Frame::new(inner_x, footer_y, inner_w, FOOTER_H, None);
    style_disclaimer_panel(&mut footer);

    let fx = inner_x + PAD;
    let fw = inner_w - 2 * PAD;
    let agree_y = footer_y + PAD;
    let chk_y = agree_y + (AGREE_ROW_H - CHK) / 2;

    let mut agree = CheckButton::new(fx, chk_y, CHK, CHK, "");
    style_check_on_surface(&mut agree);

    let mut agree_text = Frame::new(
        fx + CHK + 12,
        agree_y,
        fw - CHK - 12,
        AGREE_ROW_H,
        i18n::t(lang, I18nKey::DisclaimerAcceptCheckbox),
    );
    style_disclaimer_agree_text(&mut agree_text);

    let btn_y = footer_y + FOOTER_H - PAD - BTN_H;
    let btn_w = (fw - BTN_GAP) / 2;

    let mut btn_decline = Button::new(
        fx,
        btn_y,
        btn_w,
        BTN_H,
        i18n::t(lang, I18nKey::DisclaimerBtnDecline),
    );
    style_danger_button(&mut btn_decline);

    let mut btn_accept = Button::new(
        fx + btn_w + BTN_GAP,
        btn_y,
        btn_w,
        BTN_H,
        i18n::t(lang, I18nKey::DisclaimerBtnAccept),
    );
    style_primary_button(&mut btn_accept);
    btn_accept.deactivate();

    {
        let mut accept_btn = btn_accept.clone();
        let mut agree_cb = agree.clone();
        agree.set_callback(move |c| {
            if c.value() {
                accept_btn.activate();
            } else {
                accept_btn.deactivate();
            }
        });
        agree_text.set_callback(move |_| {
            agree_cb.set_value(!agree_cb.value());
            agree_cb.do_callback();
        });
    }

    {
        let accepted2 = accepted.clone();
        let mut win2 = win.clone();
        btn_accept.set_callback(move |_| {
            accepted2.set(true);
            win2.hide();
        });
    }

    {
        let mut win2 = win.clone();
        btn_decline.set_callback(move |_| {
            win2.hide();
        });
    }

    win.set_callback({
        let mut win2 = win.clone();
        move |_| win2.hide()
    });

    win.end();
    win.resizable(&mut display);
    win.size_range(DW, DH, DW, DH);
    win.set_pos(80 + (WIN_W - DW) / 2, 56);
    win.show();

    while win.shown() {
        app::wait();
    }

    accepted.get()
}
