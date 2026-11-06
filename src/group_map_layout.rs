//! 2D layout for the group membership graph: users (and self) on the left, groups on the right.

use crate::user_event::{GroupMapEdge, GroupMapNode};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct NodePos {
    pub id: i64,
    /// Pixel X in fixed viewport (see `VIEW_W`).
    pub x: f32,
    /// Pixel Y in fixed viewport (see `VIEW_H`).
    pub y: f32,
}

/// Default canvas when the graph is small; grows with node count for readable spacing.
pub const VIEW_W: f32 = 640.0;
pub const VIEW_H: f32 = 400.0;

const PADDING: f32 = 48.0;
/// Minimum vertical gap between node centers so labels do not sit on the same pixel row.
/// Vertical gap between node centers; must leave room for labels at typical font sizes.
const MIN_NODE_SPACING: f32 = 26.0;
/// Wider canvas when there are many nodes (more horizontal room for edge crossings).
const MIN_INNER_WIDTH: f32 = 560.0;
/// Upper bound for canvas width/height (logical px). Must stay above `n_max * MIN_NODE_SPACING`
/// or dense graphs get vertically compressed and zoom cannot recover readability.
const MAX_CANVAS_DIMENSION: f32 = 262_144.0;

/// SVG-like path for `Path` in Slint: `M x1 y1 L x2 y2` segments.
pub fn edges_to_path_commands(edges: &[(f32, f32, f32, f32)]) -> String {
    let mut s = String::new();
    for &(x1, y1, x2, y2) in edges {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&format!("M {x1:.1} {y1:.1} L {x2:.1} {y2:.1}"));
    }
    s
}

/// Left column: self + users; right column: group chats. Edge endpoints use node centers.
/// Returns node positions, edge segments, and canvas size `(w, h)` (logical px) used for layout.
pub fn layout_bipartite(
    nodes: &[GroupMapNode],
    edges: &[GroupMapEdge],
) -> (Vec<NodePos>, Vec<(f32, f32, f32, f32)>, f32, f32) {
    let mut users: Vec<i64> = Vec::new();
    let mut chats: Vec<i64> = Vec::new();
    for n in nodes {
        if n.kind == 2 {
            chats.push(n.id);
        } else {
            users.push(n.id);
        }
    }
    users.sort_unstable();
    chats.sort_unstable();

    let nu = users.len().max(1);
    let nc = chats.len().max(1);
    let n_max = nu.max(nc).max(1);

    // Height: enough rows that centers are at least MIN_NODE_SPACING apart.
    let min_inner_h = (n_max as f32) * MIN_NODE_SPACING;
    let inner_h = (VIEW_H - 2.0 * PADDING).max(min_inner_h);
    let h = (inner_h + 2.0 * PADDING).min(MAX_CANVAS_DIMENSION);

    // Width: default aspect; widen when there are many nodes or very long labels.
    let max_label_chars = nodes
        .iter()
        .map(|n| n.label.chars().count())
        .max()
        .unwrap_or(0) as f32;
    let label_bonus = (max_label_chars / 20.0).min(40.0) * 8.0;
    let min_inner_w = MIN_INNER_WIDTH + label_bonus + (n_max as f32).ln() * 40.0;
    let inner_w = (VIEW_W - 2.0 * PADDING).max(min_inner_w);
    let w = (inner_w + 2.0 * PADDING).min(MAX_CANVAS_DIMENSION);

    let inner_w_eff = w - 2.0 * PADDING;
    let inner_h_eff = h - 2.0 * PADDING;

    let x_left = PADDING + inner_w_eff * 0.22;
    let x_right = PADDING + inner_w_eff * 0.78;

    let mut pos: HashMap<i64, (f32, f32)> = HashMap::new();

    for (i, &id) in users.iter().enumerate() {
        let y = PADDING + (i as f32 + 0.5) * inner_h_eff / nu as f32;
        pos.insert(id, (x_left, y));
    }
    for (j, &id) in chats.iter().enumerate() {
        let y = PADDING + (j as f32 + 0.5) * inner_h_eff / nc as f32;
        pos.insert(id, (x_right, y));
    }

    let node_positions: Vec<NodePos> = nodes
        .iter()
        .filter_map(|n| pos.get(&n.id).map(|&(x, y)| NodePos { id: n.id, x, y }))
        .collect();

    let mut lines: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(edges.len());
    for e in edges {
        if let (Some(&(x1, y1)), Some(&(x2, y2))) = (pos.get(&e.user_id), pos.get(&e.chat_id)) {
            lines.push((x1, y1, x2, y2));
        }
    }

    (node_positions, lines, w, h)
}
