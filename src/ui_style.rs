use fltk::button::{Button, CheckButton};
use fltk::enums::{Align, Color, Font, FrameType};
use fltk::frame::Frame;
use fltk::group::{Group, Tabs};
use fltk::input::{Input, SecretInput};
use fltk::menu::Choice;
use fltk::misc::Progress;
use fltk::prelude::*;
use fltk::text::{TextDisplay, WrapMode};

pub const WIN_W: i32 = 880;
pub const WIN_H: i32 = 720;
pub const M: i32 = 24;
pub const LBL_W: i32 = 112;
pub const GAP: i32 = 10;
pub const ROW: i32 = 36;
pub const IN_H: i32 = 28;
pub const HINT_BTN_W: i32 = 132;

pub fn connect_field_w() -> i32 {
    field_w() - HINT_BTN_W - GAP
}

// Telegram dark (my.telegram.org / Desktop night palette)
pub const C_BG: Color = Color::from_rgb(23, 33, 43); // #17212b
pub const C_SURFACE: Color = Color::from_rgb(36, 47, 61); // #242f3d
pub const C_INPUT: Color = Color::from_rgb(28, 39, 51); // #1c2733
pub const C_BORDER: Color = Color::from_rgb(43, 55, 69); // #2b3745
pub const C_TEXT: Color = Color::from_rgb(255, 255, 255);
pub const C_MUTED: Color = Color::from_rgb(112, 132, 153); // #708499
pub const C_HINT: Color = Color::from_rgb(36, 47, 61);
pub const C_HINT_TEXT: Color = Color::from_rgb(112, 132, 153);
pub const C_LOG_BG: Color = Color::from_rgb(15, 23, 33); // #0f1721
pub const C_LOG_TEXT: Color = Color::from_rgb(245, 245, 245);
pub const C_ACCENT: Color = Color::from_rgb(51, 144, 236); // #3390ec
pub const C_ACCENT_LABEL: Color = Color::from_rgb(255, 255, 255);
pub const C_DANGER: Color = Color::from_rgb(229, 57, 53); // #e53935
pub const C_TAB_BAR: Color = Color::from_rgb(28, 39, 51);

pub fn field_x() -> i32 {
    M + LBL_W + GAP
}

pub fn field_w() -> i32 {
    WIN_W - field_x() - M
}

pub fn style_window(wind: &mut fltk::window::Window) {
    wind.set_color(C_BG);
}

pub fn style_tabs(tabs: &mut Tabs) {
    tabs.set_color(C_TAB_BAR);
    tabs.set_selection_color(C_BG);
    tabs.set_label_color(C_MUTED);
}

pub fn style_tab_page(group: &mut Group) {
    group.set_color(C_BG);
}

pub fn style_title(f: &mut Frame) {
    f.set_color(C_BG);
    f.set_label_font(Font::HelveticaBold);
    f.set_label_size(13);
    f.set_label_color(C_TEXT);
}

pub fn style_subtitle(f: &mut Frame) {
    f.set_color(C_BG);
    f.set_label_size(12);
    f.set_label_color(C_MUTED);
}

pub fn style_form_label(f: &mut Frame) {
    f.set_color(C_BG);
    f.set_align(Align::Right | Align::Inside);
    f.set_label_size(12);
    f.set_label_color(C_MUTED);
}

pub fn style_section_title(f: &mut Frame) {
    f.set_color(C_BG);
    f.set_label_font(Font::HelveticaBold);
    f.set_label_size(12);
    f.set_label_color(C_TEXT);
    f.set_align(Align::Left | Align::Inside);
}

pub fn style_divider(f: &mut Frame) {
    f.set_frame(FrameType::FlatBox);
    f.set_color(C_BORDER);
}

pub fn style_hint_display(d: &mut TextDisplay) {
    d.set_frame(FrameType::FlatBox);
    d.set_color(C_HINT);
    d.set_text_color(C_HINT_TEXT);
    d.set_text_size(12);
    d.wrap_mode(WrapMode::AtBounds, 6);
}

pub fn style_disclaimer_display(d: &mut TextDisplay) {
    d.set_frame(FrameType::FlatBox);
    d.set_color(C_LOG_BG);
    d.set_text_color(C_TEXT);
    d.set_text_size(12);
    d.set_text_font(Font::Helvetica);
    d.wrap_mode(WrapMode::AtBounds, 0);
}

pub fn style_disclaimer_panel(f: &mut Frame) {
    f.set_frame(FrameType::FlatBox);
    f.set_color(C_SURFACE);
}

pub fn style_disclaimer_agree_text(f: &mut Frame) {
    f.set_frame(FrameType::FlatBox);
    f.set_color(C_SURFACE);
    f.set_label_color(C_TEXT);
    f.set_label_size(11);
    f.set_label_font(Font::Helvetica);
    f.set_align(Align::Left | Align::Top | Align::Inside | Align::Wrap);
}

pub fn style_check_on_surface(c: &mut CheckButton) {
    c.set_color(C_SURFACE);
    c.set_label_color(C_TEXT);
    c.set_label_size(11);
}

pub fn style_log_display(d: &mut TextDisplay) {
    d.set_frame(FrameType::FlatBox);
    d.set_color(C_LOG_BG);
    d.set_text_color(C_LOG_TEXT);
    d.set_text_font(Font::Courier);
    d.set_text_size(12);
    d.wrap_mode(WrapMode::AtBounds, 4);
}

pub fn style_text_input(i: &mut Input) {
    i.set_frame(FrameType::FlatBox);
    i.set_color(C_INPUT);
    i.set_text_color(C_TEXT);
    i.set_text_size(12);
    i.set_selection_color(C_ACCENT);
}

pub fn style_secret_input(i: &mut SecretInput) {
    i.set_frame(FrameType::FlatBox);
    i.set_color(C_INPUT);
    i.set_text_color(C_TEXT);
    i.set_text_size(12);
    i.set_selection_color(C_ACCENT);
}

pub fn style_choice(c: &mut Choice) {
    c.set_frame(FrameType::FlatBox);
    c.set_color(C_INPUT);
    c.set_text_color(C_TEXT);
    c.set_label_color(C_TEXT);
    c.set_text_size(12);
}

pub fn style_check(c: &mut CheckButton) {
    c.set_color(C_BG);
    c.set_label_color(C_TEXT);
    c.set_label_size(12);
}

pub fn style_primary_button(b: &mut Button) {
    b.set_frame(FrameType::FlatBox);
    b.set_color(C_ACCENT);
    b.set_label_color(C_ACCENT_LABEL);
    b.set_label_size(12);
}

pub fn style_secondary_button(b: &mut Button) {
    b.set_frame(FrameType::FlatBox);
    b.set_color(C_SURFACE);
    b.set_label_color(C_TEXT);
    b.set_label_size(12);
}

pub fn style_danger_button(b: &mut Button) {
    b.set_frame(FrameType::FlatBox);
    b.set_color(C_SURFACE);
    b.set_label_color(C_DANGER);
    b.set_label_size(12);
}

pub fn style_busy_backdrop(f: &mut Frame) {
    f.set_frame(FrameType::FlatBox);
    f.set_color(Color::from_rgb(15, 23, 33));
}

pub fn style_busy_panel(f: &mut Frame) {
    f.set_frame(FrameType::DownBox);
    f.set_color(C_SURFACE);
}

pub fn style_busy_label(f: &mut Frame) {
    f.set_frame(FrameType::NoBox);
    f.set_color(C_SURFACE);
    f.set_label_color(C_TEXT);
    f.set_label_size(13);
}

pub fn style_busy_progress(p: &mut Progress) {
    p.set_frame(FrameType::FlatBox);
    p.set_color(C_INPUT);
    p.set_selection_color(C_ACCENT);
    p.set_label_color(C_MUTED);
    p.set_minimum(0.0);
    p.set_maximum(100.0);
    p.set_value(0.0);
}

pub fn style_field_hint_link(b: &mut Button) {
    b.set_frame(FrameType::FlatBox);
    b.set_color(C_BG);
    b.set_selection_color(C_HINT);
    b.set_label_color(C_ACCENT);
    b.set_label_size(11);
    b.set_align(Align::Center | Align::Inside);
}
