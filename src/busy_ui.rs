use crate::i18n::{self, Lang, I18nKey};
use crate::ui_style::{
    style_busy_backdrop, style_busy_label, style_busy_panel, style_busy_progress, WIN_H, WIN_W,
};
use fltk::button::Button;
use fltk::enums::{Align, Cursor, FrameType};
use fltk::frame::Frame;
use fltk::group::Group;
use fltk::menu::Choice;
use fltk::misc::Progress;
use fltk::prelude::*;
use fltk::window::Window;
use std::cell::Cell;

const PANEL_W: i32 = 300;
const PANEL_H: i32 = 86;

pub struct BusyOverlay {
    group: Group,
    label: Frame,
    progress: Progress,
    pulse: Cell<u32>,
}

pub struct OperationTargets {
    pub wind: Window,
    pub tabs: fltk::group::Tabs,
    pub btn_connect: Button,
    pub btn_code: Button,
    pub btn_password: Button,
    pub btn_lookup: Button,
    pub btn_save: Button,
    pub lookup_mode: Choice,
}

impl BusyOverlay {
    pub fn new() -> Self {
        let mut group = Group::new(0, 0, WIN_W, WIN_H, None);
        group.set_frame(FrameType::NoBox);

        let mut backdrop = Frame::new(0, 0, WIN_W, WIN_H, None);
        style_busy_backdrop(&mut backdrop);

        let px = (WIN_W - PANEL_W) / 2;
        let py = (WIN_H - PANEL_H) / 2;
        let mut panel = Frame::new(px, py, PANEL_W, PANEL_H, None);
        style_busy_panel(&mut panel);

        let mut label = Frame::new(px + 20, py + 16, PANEL_W - 40, 22, "");
        style_busy_label(&mut label);
        label.set_align(Align::Center | Align::Inside);

        let mut progress = Progress::new(px + 20, py + 48, PANEL_W - 40, 16, None);
        style_busy_progress(&mut progress);

        group.end();
        group.hide();

        Self {
            group,
            label,
            progress,
            pulse: Cell::new(0),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.group.visible()
    }

    pub fn show(&mut self, lang: Lang, targets: &mut OperationTargets) {
        self.pulse.set(0);
        self.progress.set_value(0.0);
        self.label.set_label(i18n::t(lang, I18nKey::PreloaderWorking));
        set_targets_active(targets, false);
        targets.wind.set_cursor(Cursor::Wait);
        self.group.show();
        self.group.redraw();
        targets.wind.redraw();
    }

    pub fn hide(&mut self, targets: &mut OperationTargets) {
        if !self.is_visible() {
            return;
        }
        self.group.hide();
        self.progress.set_value(0.0);
        set_targets_active(targets, true);
        targets.wind.set_cursor(Cursor::Default);
        targets.wind.redraw();
    }

    pub fn refresh_label(&mut self, lang: Lang) {
        if self.is_visible() {
            self.label.set_label(i18n::t(lang, I18nKey::PreloaderWorking));
        }
    }

    pub fn tick(&mut self) {
        if !self.is_visible() {
            return;
        }
        let step = self.pulse.get().wrapping_add(5);
        self.pulse.set(step);
        self.progress.set_value(f64::from(step % 100));
        self.group.redraw();
    }
}

fn set_targets_active(targets: &mut OperationTargets, active: bool) {
    macro_rules! toggle {
        ($w:expr) => {
            if active {
                $w.activate();
            } else {
                $w.deactivate();
            }
        };
    }
    toggle!(targets.tabs);
    toggle!(targets.btn_connect);
    toggle!(targets.btn_code);
    toggle!(targets.btn_password);
    toggle!(targets.btn_lookup);
    toggle!(targets.btn_save);
    toggle!(targets.lookup_mode);
}
