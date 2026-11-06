//! Fill `GroupMapWindow` from worker `GroupMapReady` data.

use crate::group_map_layout::{edges_to_path_commands, layout_bipartite};
use crate::i18n::{self, Lang, I18nKey};
use crate::slint_app::{GroupMapNodeSlot, GroupMapWindow};
use crate::user_event::GroupMapReady;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

pub fn apply_strings(win: &GroupMapWindow, lang: Lang) {
    win.set_window_title(i18n::t(lang, I18nKey::GroupMapWindowTitle).into());
    win.set_subtitle_text(i18n::t(lang, I18nKey::GroupMapSubtitle).into());
    win.set_legend_self(i18n::t(lang, I18nKey::GroupMapLegendSelf).into());
    win.set_legend_user(i18n::t(lang, I18nKey::GroupMapLegendUser).into());
    win.set_legend_group(i18n::t(lang, I18nKey::GroupMapLegendGroup).into());
    win.set_map_hint_text(i18n::t(lang, I18nKey::GroupMapInteractionHint).into());
    win.set_close_label(i18n::t(lang, I18nKey::CardClose).into());
}

fn truncate_label(s: &str, max_chars: usize) -> SharedString {
    let iter = s.chars();
    let count = iter.clone().count();
    if count <= max_chars {
        return SharedString::from(s);
    }
    let prefix: String = iter.take(max_chars.saturating_sub(1)).collect();
    SharedString::from(format!("{prefix}…"))
}

pub fn populate(win: &GroupMapWindow, lang: Lang, data: &GroupMapReady) {
    let stats = i18n::format_group_map_stats(
        lang,
        data.anchor_label.as_str(),
        data.depth,
        data.dialogs_scanned,
        data.groups_processed,
        data.users_found,
        data.edges.len(),
        &data.errors,
    );
    win.set_stats_text(stats.into());

    let (positions, edge_lines, canvas_w, canvas_h) = layout_bipartite(&data.nodes, &data.edges);
    let cmds = edges_to_path_commands(&edge_lines);
    win.set_edge_path_commands(cmds.into());

    win.set_graph_w(canvas_w);
    win.set_graph_h(canvas_h);

    // Shorter labels for people/crowded graphs reduce overlap; groups keep a bit more room.
    let max_label = if data.nodes.len() > 800 {
        18usize
    } else if data.nodes.len() > 400 {
        24usize
    } else if data.nodes.len() > 200 {
        32usize
    } else {
        48usize
    };

    let slots: Vec<GroupMapNodeSlot> = data
        .nodes
        .iter()
        .filter_map(|n| {
            positions.iter().find(|p| p.id == n.id).map(|p| GroupMapNodeSlot {
                kind: n.kind,
                label: truncate_label(
                    &n.label,
                    if n.kind == 2 { max_label + 8 } else { max_label },
                ),
                rel_x: p.x / canvas_w,
                rel_y: p.y / canvas_h,
            })
        })
        .collect();
    win.set_map_nodes(ModelRc::new(VecModel::from(slots)));

    let wk = win.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(80), move || {
        if let Some(ui) = wk.upgrade() {
            ui.invoke_fit_map_view();
        }
    });
}
