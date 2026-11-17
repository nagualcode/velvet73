//! Editor: black tolex and brushed aluminium, the materials of the piano
//! and the amp, drawn with a modern DAW's economy.
//!
//! Top: the name, the presets and the pedal. Across the
//! middle: the harp itself, one tine per key, glowing with how far its tip
//! is swinging (the picture of what the pickups see). Below: three
//! panels laid out like the hardware's own controls: the piano's
//! regulation (amber, the colour of the tines), the Stage nameboard
//! (teal) and the Twin's Vibrato channel (blue), each a brushed plate
//! with a black band, skirted knobs with spun caps and jewel lamps.
//! Colour is only an accent.
//! No virtual keyboard: play it from yours.

use crate::dsp::KEYS;
use crate::params::VelvetParams;
use crate::params::VelvetParamsParamId as P;
use crate::params::load_f32;
use crate::presets::{self, Preset};
use egui::{Align2, Color32, CornerRadius, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, pos2, vec2};
use std::sync::atomic::Ordering;
use truce::prelude::*;
use truce_core::editor::{PluginContext, PluginContextReadF32};

pub const WINDOW_W: u32 = 1180;
pub const WINDOW_H: u32 = 580;
pub const MIN_W: u32 = 826;
pub const MIN_H: u32 = 406;
pub const MAX_W: u32 = 2360;
pub const MAX_H: u32 = 1160;

type Ctx = PluginContext<VelvetParams>;

// ---------------------------------------------------------------------------
// Palette: graphite, thin lines, one accent per section.
// ---------------------------------------------------------------------------

const BG: Color32 = Color32::from_rgb(28, 28, 29);
const PANEL: Color32 = Color32::from_rgb(38, 38, 40);
const HEAD: Color32 = Color32::from_rgb(46, 46, 49);
const WELL: Color32 = Color32::from_rgb(20, 20, 21);
const FIELD: Color32 = Color32::from_rgb(24, 24, 25);
const LINE: Color32 = Color32::from_rgb(62, 62, 66);
const TRACK: Color32 = Color32::from_rgb(70, 70, 75);
const TEXT: Color32 = Color32::from_rgb(222, 222, 224);
const TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 146);
const AMBER: Color32 = Color32::from_rgb(232, 168, 86);
const TEAL: Color32 = Color32::from_rgb(92, 184, 164);
const BLUE: Color32 = Color32::from_rgb(132, 152, 238);
const RED: Color32 = Color32::from_rgb(226, 98, 90);
// Tolex, aluminium and the ink printed on it.
const TOLEX: Color32 = Color32::from_rgb(19, 19, 20);
const BAND: Color32 = Color32::from_rgb(22, 22, 24);
const METAL_HI: Color32 = Color32::from_rgb(214, 216, 218);
const METAL_MID: Color32 = Color32::from_rgb(188, 190, 193);
const METAL_LO: Color32 = Color32::from_rgb(158, 161, 165);
const INK: Color32 = Color32::from_rgb(34, 35, 38);
const INK_DIM: Color32 = Color32::from_rgb(82, 84, 90);
const SKIRT: Color32 = Color32::from_rgb(24, 24, 26);

pub fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.panel_fill = TOLEX;
    v.window_fill = TOLEX;
    v
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

/// Brighten (d > 0) or darken a colour by `d` levels.
fn shade(c: Color32, d: f32) -> Color32 {
    let l = |x: u8| (f32::from(x) + d).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(l(c.r()), l(c.g()), l(c.b()))
}

/// The accent as printed ink: deep enough to read on aluminium.
fn ink_of(accent: Color32) -> Color32 {
    mix(accent, Color32::BLACK, 0.38)
}

fn hash(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B1) ^ 0x5bd1_e995;
    x ^= x >> 15;
    x = x.wrapping_mul(0x85EB_CA77);
    x ^= x >> 13;
    (x & 0xFFFF) as f32 / 65535.0
}

fn quad(m: &mut Mesh, r: Rect, tl: Color32, tr: Color32, bl: Color32, br: Color32) {
    let i = m.vertices.len() as u32;
    m.colored_vertex(r.left_top(), tl);
    m.colored_vertex(r.right_top(), tr);
    m.colored_vertex(r.left_bottom(), bl);
    m.colored_vertex(r.right_bottom(), br);
    m.add_triangle(i, i + 1, i + 2);
    m.add_triangle(i + 1, i + 3, i + 2);
}

/// Brushed aluminium: a soft top-lit gradient, then streaks along the
/// grain (one every two screen pixels, each varying along its length),
/// a bevel and a dark edge.
fn plate(pt: &egui::Painter, r: Rect, seed: u32) {
    let mut m = Mesh::default();
    let y1 = r.top() + r.height() * 0.42;
    quad(&mut m, Rect::from_min_max(r.left_top(), pos2(r.right(), y1)), METAL_HI, METAL_HI, METAL_MID, METAL_MID);
    quad(&mut m, Rect::from_min_max(pos2(r.left(), y1), r.right_bottom()), METAL_MID, METAL_MID, METAL_LO, METAL_LO);
    let rows = (r.height() / 2.0) as u32;
    let segs = 5u32;
    let sw = r.width() / segs as f32;
    for row in 0..rows {
        let y = r.top() + row as f32 * 2.0;
        let light = hash(seed ^ row.wrapping_mul(7919)) > 0.5;
        let a = |k: u32| -> Color32 {
            let v = hash(seed.wrapping_add(row * 31 + k * 977));
            let al = (v * v * 26.0) as u8;
            if light { Color32::from_rgba_unmultiplied(255, 255, 255, al) } else { Color32::from_rgba_unmultiplied(0, 0, 0, al / 2) }
        };
        for k in 0..segs {
            let rr = Rect::from_min_max(pos2(r.left() + k as f32 * sw, y), pos2(r.left() + (k + 1) as f32 * sw, y + 1.0));
            let (c0, c1) = (a(k), a(k + 1));
            quad(&mut m, rr, c0, c1, c0, c1);
        }
    }
    pt.add(Shape::mesh(m));
    pt.line_segment([r.left_top(), r.right_top()], Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 150)));
    pt.line_segment([r.left_bottom() - vec2(0.0, 0.5), r.right_bottom() - vec2(0.0, 0.5)], Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 110)));
    pt.rect_stroke(r, CornerRadius::ZERO, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 160)), StrokeKind::Outside);
}

/// Spun aluminium (a lathe-turned cap): a conic sheen, two highlights
/// opposite each other, and fine rings.
fn spun(pt: &egui::Painter, c: Pos2, r: f32) {
    let mut m = Mesh::default();
    m.colored_vertex(c, METAL_MID);
    let n = 48u32;
    for k in 0..=n {
        let th = std::f32::consts::TAU * k as f32 / n as f32;
        let l = 30.0 * (2.0 * (th + 0.7)).cos() + 8.0 * (6.0 * th).cos() - 6.0;
        m.colored_vertex(c + vec2(th.cos(), th.sin()) * r, shade(METAL_MID, l));
        if k > 0 {
            m.add_triangle(0, k, k + 1);
        }
    }
    pt.add(Shape::mesh(m));
    for k in 1..4 {
        pt.circle_stroke(c, r * k as f32 / 4.0, Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 22)));
    }
    pt.circle_stroke(c, r, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 120)));
}

fn screw(pt: &egui::Painter, f: &Frame, c: Pos2, turn: f32) {
    let r = f.len(4.2);
    pt.circle_filled(c + vec2(0.0, 0.8), r, Color32::from_rgba_unmultiplied(0, 0, 0, 90));
    spun(pt, c, r);
    let d = vec2(turn.cos(), turn.sin()) * r * 0.8;
    let e = vec2(-d.y, d.x);
    let s = Stroke::new(f.len(1.1), Color32::from_rgba_unmultiplied(20, 20, 22, 200));
    pt.line_segment([c - d, c + d], s);
    pt.line_segment([c - e, c + e], s);
}

/// A jewel pilot lamp: dark glass off, lit glass with a halo on.
fn jewel(pt: &egui::Painter, f: &Frame, c: Pos2, on: bool, color: Color32) {
    let r = f.len(5.0);
    if on {
        for k in 0..4 {
            let rr = r * (1.4 + 0.45 * k as f32);
            let a = (46.0 / (1.0 + k as f32)) as u8;
            pt.circle_filled(c, rr, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a));
        }
    }
    pt.circle_filled(c, r + f.len(1.4), Color32::from_rgb(70, 72, 76));
    pt.circle_filled(c, r, if on { color } else { mix(color, Color32::BLACK, 0.78) });
    pt.circle_filled(c - vec2(r * 0.35, r * 0.35), r * 0.32, Color32::from_rgba_unmultiplied(255, 255, 255, if on { 170 } else { 40 }));
}

/// An engraved rule across a plate.
fn groove(pt: &egui::Painter, a: Pos2, b: Pos2) {
    pt.line_segment([a, b], Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 70)));
    pt.line_segment([a + vec2(0.0, 1.0), b + vec2(0.0, 1.0)], Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 120)));
}

// ---------------------------------------------------------------------------
// Layout frame: authored at 1180×640 and scaled.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Frame {
    origin: Pos2,
    s: f32,
}

impl Frame {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x * self.s, y * self.s)
    }
    fn r(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.p(x, y), vec2(w * self.s, h * self.s))
    }
    fn len(&self, v: f32) -> f32 {
        v * self.s
    }
    fn font(&self, size: f32) -> FontId {
        FontId::proportional(size * self.s)
    }
    fn mono(&self, size: f32) -> FontId {
        FontId::monospace(size * self.s)
    }
}

// ---------------------------------------------------------------------------
// Parameter helpers
// ---------------------------------------------------------------------------

fn info_of(ctx: &Ctx, id: P) -> Option<ParamInfo> {
    let id: u32 = id.into();
    ctx.params().param_infos().into_iter().find(|i| i.id == id)
}

fn default_norm(ctx: &Ctx, id: P) -> f64 {
    info_of(ctx, id).map_or(0.0, |i| i.range.normalize(i.default_plain))
}

fn enum_index(ctx: &Ctx, id: P, count: usize) -> usize {
    (ctx.get_param(id) * (count - 1) as f32).round() as usize
}

fn set_enum(ctx: &Ctx, id: P, idx: usize, count: usize) {
    ctx.automate(id, idx as f64 / (count - 1) as f64);
}

/// Vertical drag edits `id` (Shift = fine, double-click = default).
fn drag_param(ui: &egui::Ui, ctx: &Ctx, f: &Frame, resp: &egui::Response, id: P, px_full: f32) {
    let raw_id = resp.id.with("raw");
    if resp.drag_started() {
        ctx.begin_edit(id);
        ui.ctx().data_mut(|d| d.insert_temp(raw_id, ctx.get_param(id)));
    }
    if resp.dragged() {
        let fine = ui.input(|i| i.modifiers.shift);
        let d = -resp.drag_delta().y / (px_full * if fine { 6.0 } else { 1.0 }) / f.s.max(0.3);
        let raw: f32 = ui.ctx().data(|dd| dd.get_temp(raw_id)).unwrap_or_else(|| ctx.get_param(id));
        let raw = (raw + d).clamp(0.0, 1.0);
        ui.ctx().data_mut(|dd| dd.insert_temp(raw_id, raw));
        ctx.set_param(id, f64::from(raw));
    }
    if resp.drag_stopped() {
        ctx.end_edit(id);
    }
    if resp.double_clicked() {
        ctx.automate(id, default_norm(ctx, id));
    }
}

// ---------------------------------------------------------------------------
// Widgets
// ---------------------------------------------------------------------------

/// Skirted amp knob on the plate: black skirt, spun-aluminium cap, white
/// pointer; around it the printed scale and the value arc in the section's
/// ink; label above, value below. Label and value are part of the hit area.
fn knob(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame, id: P, label: &str, x: f32, y: f32, r: f32, accent: Color32) {
    let center = f.p(x, y);
    let rad = f.len(r);
    let hit = Rect::from_min_max(center - vec2(rad * 1.45, rad + f.len(26.0)), center + vec2(rad * 1.45, rad + f.len(28.0)));
    let resp = ui.interact(hit, ui.id().with(("knob", u32::from(id))), Sense::click_and_drag());
    drag_param(ui, ctx, f, &resp, id, 200.0);
    let v = ctx.get_param(id);
    let hot = resp.hovered() || resp.dragged();
    let pt = ui.painter();
    let start = std::f32::consts::FRAC_PI_4 * 3.0;
    let sweep = std::f32::consts::FRAC_PI_2 * 3.0;
    let arc = |from: f32, to: f32, rr: f32| -> Vec<Pos2> {
        let n = 48;
        (0..=n)
            .map(|k| {
                let a = from + (to - from) * k as f32 / n as f32;
                center + vec2(a.cos(), a.sin()) * rr
            })
            .collect()
    };
    let ink = ink_of(accent);
    // Printed scale: eleven ticks, 0 to 10.
    for k in 0..=10 {
        let t = start + sweep * k as f32 / 10.0;
        let d = vec2(t.cos(), t.sin());
        let long = k % 5 == 0;
        pt.line_segment(
            [center + d * (rad + f.len(3.5)), center + d * (rad + f.len(if long { 9.0 } else { 7.0 }))],
            Stroke::new(f.len(if long { 1.4 } else { 1.0 }), INK_DIM),
        );
    }
    let a = start + sweep * v;
    if v > 0.003 {
        pt.add(Shape::line(arc(start, a, rad + f.len(3.6)), Stroke::new(f.len(2.6), if hot { mix(ink, accent, 0.5) } else { ink })));
    }
    // Shadow, skirt, cap.
    pt.circle_filled(center + vec2(0.0, f.len(2.0)), rad * 1.02, Color32::from_rgba_unmultiplied(0, 0, 0, 70));
    pt.circle_filled(center, rad, SKIRT);
    pt.circle_stroke(center, rad - f.len(0.8), Stroke::new(f.len(1.0), Color32::from_rgb(58, 58, 62)));
    let cap = rad * 0.64;
    spun(pt, center, cap);
    if hot {
        pt.circle_stroke(center, cap, Stroke::new(f.len(1.2), mix(accent, Color32::WHITE, 0.3)));
    }
    let dir = vec2(a.cos(), a.sin());
    pt.line_segment([center + dir * cap * 1.04, center + dir * rad * 0.94], Stroke::new(f.len(2.4), Color32::from_rgb(236, 236, 232)));
    pt.line_segment([center + dir * cap * 0.25, center + dir * cap * 0.9], Stroke::new(f.len(1.2), Color32::from_rgba_unmultiplied(20, 20, 22, 150)));
    pt.text(center - vec2(0.0, rad + f.len(13.0)), Align2::CENTER_BOTTOM, label.to_uppercase(), f.font(9.5), INK);
    pt.text(center + vec2(0.0, rad + f.len(19.0)), Align2::CENTER_CENTER, ctx.format_param(id), f.mono(9.5), if hot { ink } else { INK_DIM });
}

/// Segmented selector for an enum parameter: click a segment, or drag
/// up/down one notch per step.
fn segmented(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame, id: P, labels: &[&str], x: f32, y: f32, w: f32, h: f32, accent: Color32) {
    let n = labels.len();
    let cur = enum_index(ctx, id, n);
    let all = f.r(x, y, w, h);
    let resp_all = ui.interact(all, ui.id().with(("seg-drag", u32::from(id))), Sense::drag());
    let acc_id = resp_all.id.with("acc");
    if resp_all.dragged() {
        let mut acc: f32 = ui.ctx().data(|d| d.get_temp(acc_id)).unwrap_or(0.0);
        acc += resp_all.drag_delta().x - resp_all.drag_delta().y;
        let step = f.len(26.0);
        if acc.abs() > step {
            let next = (cur as i32 + acc.signum() as i32).clamp(0, n as i32 - 1) as usize;
            set_enum(ctx, id, next, n);
            acc = 0.0;
        }
        ui.ctx().data_mut(|d| d.insert_temp(acc_id, acc));
    }
    if resp_all.drag_stopped() {
        ui.ctx().data_mut(|d| d.insert_temp(acc_id, 0.0f32));
    }
    let sw = w / n as f32;
    for (k, label) in labels.iter().enumerate() {
        let r = f.r(x + k as f32 * sw, y, sw, h);
        let resp = ui.interact(r.shrink(f.len(1.0)), ui.id().with(("seg", u32::from(id), k)), Sense::click());
        if resp.clicked() {
            set_enum(ctx, id, k, n);
        }
        let on = k == cur;
        let pt = ui.painter();
        pt.rect_filled(r, CornerRadius::ZERO, if on { mix(PANEL, accent, 0.28) } else if resp.hovered() { HEAD } else { FIELD });
        pt.text(r.center(), Align2::CENTER_CENTER, label.to_uppercase(), f.font(9.5), if on { mix(accent, TEXT, 0.35) } else { TEXT_DIM });
    }
    let pt = ui.painter();
    pt.rect_stroke(all, CornerRadius::same((2.0 * f.s) as u8), Stroke::new(f.len(1.0), LINE), StrokeKind::Inside);
    for k in 1..n {
        let xx = all.left() + all.width() * k as f32 / n as f32;
        pt.line_segment([pos2(xx, all.top()), pos2(xx, all.bottom())], Stroke::new(f.len(1.0), LINE));
    }
}

/// On/off button with a status lamp.
fn toggle(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame, id: P, label: &str, x: f32, y: f32, w: f32, accent: Color32) {
    let r = f.r(x, y, w, 24.0);
    let resp = ui.interact(r, ui.id().with(("toggle", u32::from(id))), Sense::click());
    let on = ctx.get_param(id) > 0.5;
    if resp.clicked() {
        ctx.automate(id, if on { 0.0 } else { 1.0 });
    }
    let pt = ui.painter();
    pt.rect_filled(r, CornerRadius::same((2.0 * f.s) as u8), if resp.hovered() { HEAD } else { FIELD });
    pt.rect_stroke(r, CornerRadius::same((2.0 * f.s) as u8), Stroke::new(f.len(1.0), if on { mix(accent, LINE, 0.4) } else { LINE }), StrokeKind::Inside);
    jewel(pt, f, pos2(r.left() + f.len(13.0), r.center().y), on, accent);
    pt.text(pos2(r.left() + f.len(25.0), r.center().y), Align2::LEFT_CENTER, label.to_uppercase(), f.font(9.5), if on { TEXT } else { TEXT_DIM });
}

/// A brushed plate with a black band across the top (the silverface way),
/// four screws, and the section's accent as a thin line under the band.
fn panel(pt: &egui::Painter, f: &Frame, x: f32, y: f32, w: f32, h: f32, title: &str, sub: &str, accent: Color32) {
    let seed = (x as u32).wrapping_mul(2654435761);
    plate(pt, f.r(x, y, w, h), seed);
    pt.rect_filled(f.r(x, y, w, 28.0), CornerRadius::ZERO, BAND);
    pt.rect_filled(f.r(x, y + 28.0, w, 2.0), CornerRadius::ZERO, accent);
    pt.text(f.p(x + 26.0, y + 14.0), Align2::LEFT_CENTER, title.to_uppercase(), f.font(10.5), TEXT);
    pt.text(f.p(x + w - 26.0, y + 14.0), Align2::RIGHT_CENTER, sub, f.font(9.5), TEXT_DIM);
    for (k, (sx, sy)) in [(11.0, 14.0), (w - 11.0, 14.0), (11.0, h - 11.0), (w - 11.0, h - 11.0)].into_iter().enumerate() {
        screw(pt, f, f.p(x + sx, y + sy), 0.3 + 1.7 * hash(seed + k as u32));
    }
}

// ---------------------------------------------------------------------------
// Presets
// ---------------------------------------------------------------------------

/// Preset key (parameter field name) → parameter.
pub(crate) fn key_id(key: &str) -> Option<P> {
    Some(match key {
        "voicing" => P::Voicing,
        "gap" => P::Gap,
        "hammer" => P::Hammer,
        "touch" => P::Touch,
        "condition" => P::Condition,
        "a4" => P::A4,
        "stretch" => P::Stretch,
        "keys" => P::Keys,
        "stage_volume" => P::StageVolume,
        "bass_boost" => P::BassBoost,
        "harp" => P::Harp,
        "cable" => P::Cable,
        "board" => P::Board,
        "amp" => P::Amp,
        "amp_volume" => P::AmpVolume,
        "treble" => P::Treble,
        "middle" => P::Middle,
        "bass" => P::Bass,
        "bright" => P::Bright,
        "reverb" => P::Reverb,
        "speed" => P::Speed,
        "intensity" => P::Intensity,
        "mic" => P::Mic,
        "noise" => P::Noise,
        "output" => P::Output,
        _ => return None,
    })
}

fn preset_norm(infos: &[ParamInfo], key: &str, plain: f64) -> Option<(P, f64)> {
    let id = key_id(key)?;
    let raw: u32 = id.into();
    let info = infos.iter().find(|i| i.id == raw)?;
    Some((id, info.range.normalize(plain)))
}

fn apply_preset(ctx: &Ctx, infos: &[ParamInfo], p: &Preset) {
    for (k, v) in &p.values {
        if let Some((id, n)) = preset_norm(infos, k, *v) {
            ctx.automate(id, n);
        }
    }
    // A new instrument on the stand: no ghost notes from the last one.
    ctx.params().ui.preset_epoch.fetch_add(1, Ordering::Relaxed);
}

fn current_preset(ctx: &Ctx, infos: &[ParamInfo]) -> Option<usize> {
    presets::library().iter().position(|p| {
        p.values.iter().all(|(k, v)| preset_norm(infos, k, *v).is_none_or(|(id, n)| (f64::from(ctx.get_param(id)) - n).abs() < 2e-3))
    })
}

fn browser_id() -> egui::Id {
    egui::Id::new("velvet-preset-browser")
}

// ---------------------------------------------------------------------------
// The editor
// ---------------------------------------------------------------------------

pub fn velvet_ui(ui: &mut egui::Ui, ctx: &Ctx) {
    let full = ui.max_rect();
    let s = (full.width() / WINDOW_W as f32).min(full.height() / WINDOW_H as f32);
    let size = vec2(WINDOW_W as f32 * s, WINDOW_H as f32 * s);
    let f = Frame { origin: full.center() - size * 0.5, s };
    tolex(ui.painter(), full);
    let infos = ctx.params().param_infos();
    header(ui, ctx, &f, &infos);
    harp(ui, ctx, &f);
    piano_panel(ui, ctx, &f);
    stage_panel(ui, ctx, &f);
    twin_panel(ui, ctx, &f);
    browser(ui, ctx, &f, &infos);
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
}

/// Black tolex: near-black with a faint pebble grain.
fn tolex(pt: &egui::Painter, r: Rect) {
    pt.rect_filled(r, CornerRadius::ZERO, TOLEX);
    let mut m = Mesh::default();
    let cell = 3.0;
    let (nx, ny) = ((r.width() / cell) as u32, (r.height() / cell) as u32);
    for j in 0..ny {
        for i in 0..nx {
            let h = hash(i.wrapping_mul(73_856_093) ^ j.wrapping_mul(19_349_663));
            if h > 0.72 {
                let a = ((h - 0.72) * 40.0) as u8;
                let p = r.left_top() + vec2(i as f32 * cell, j as f32 * cell);
                let c = Color32::from_rgba_unmultiplied(255, 255, 255, a);
                quad(&mut m, Rect::from_min_size(p, vec2(1.5, 1.5)), c, c, c, c);
            }
        }
    }
    pt.add(Shape::mesh(m));
}

fn header(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame, infos: &[ParamInfo]) {
    let sh = &ctx.params().ui;
    {
        let pt = ui.painter();
        pt.line_segment([f.p(0.0, 52.0), f.p(WINDOW_W as f32, 52.0)], Stroke::new(f.len(1.0), Color32::from_rgb(40, 40, 43)));
        pt.text(f.p(20.0, 26.0), Align2::LEFT_CENTER, "VELVET", FontId::proportional(20.0 * f.s), TEXT);
        pt.text(f.p(102.0, 26.0), Align2::LEFT_CENTER, "73", FontId::proportional(20.0 * f.s), AMBER);
        pt.text(f.p(138.0, 20.0), Align2::LEFT_CENTER, "Mark I Stage 73 · Twin Reverb", f.font(9.5), TEXT_DIM);
        pt.text(f.p(138.0, 33.0), Align2::LEFT_CENTER, "Jardim Sonoro", f.font(9.0), mix(TEXT_DIM, BG, 0.3));
    }
    // Presets.
    let lib = presets::library();
    let cur = current_preset(ctx, infos);
    let mid = f.r(420.0, 12.0, 300.0, 28.0);
    for (key, label, x, d) in [("pprev", "◀", 392.0, lib.len().max(1) - 1), ("pnext", "▶", 724.0, 1)] {
        let r = f.r(x, 12.0, 26.0, 28.0);
        let resp = ui.interact(r, ui.id().with(key), Sense::click());
        if resp.clicked() && !lib.is_empty() {
            let i = cur.map_or(0, |c| (c + d) % lib.len());
            apply_preset(ctx, infos, &lib[i]);
        }
        let pt = ui.painter();
        pt.rect_filled(r, CornerRadius::same((2.0 * f.s) as u8), if resp.hovered() { HEAD } else { FIELD });
        pt.rect_stroke(r, CornerRadius::same((2.0 * f.s) as u8), Stroke::new(f.len(1.0), LINE), StrokeKind::Inside);
        pt.text(r.center(), Align2::CENTER_CENTER, label, f.font(9.0), if resp.hovered() { TEXT } else { TEXT_DIM });
    }
    let resp = ui.interact(mid, ui.id().with("preset"), Sense::click());
    if resp.clicked() {
        let open: bool = ui.ctx().data(|d| d.get_temp(browser_id())).unwrap_or(false);
        ui.ctx().data_mut(|d| d.insert_temp(browser_id(), !open));
    }
    {
        let pt = ui.painter();
        pt.rect_filled(mid, CornerRadius::same((2.0 * f.s) as u8), if resp.hovered() { HEAD } else { FIELD });
        pt.rect_stroke(mid, CornerRadius::same((2.0 * f.s) as u8), Stroke::new(f.len(1.0), if resp.hovered() { AMBER } else { LINE }), StrokeKind::Inside);
        let (cat, name) = cur.map_or(("", "—"), |i| (lib[i].category.as_str(), lib[i].name.as_str()));
        pt.text(pos2(mid.left() + f.len(12.0), mid.center().y), Align2::LEFT_CENTER, cat.to_uppercase(), f.font(9.0), AMBER);
        pt.text(pos2(mid.right() - f.len(28.0), mid.center().y), Align2::RIGHT_CENTER, name, f.font(12.0), TEXT);
        let c = pos2(mid.right() - f.len(14.0), mid.center().y);
        pt.add(Shape::convex_polygon(vec![c + vec2(-f.len(4.0), -f.len(2.0)), c + vec2(f.len(4.0), -f.len(2.0)), c + vec2(0.0, f.len(3.0))], TEXT_DIM, Stroke::NONE));
    }
    // Sustain pedal lamp.
    let sus = sh.sustain.load(Ordering::Relaxed) != 0;
    {
        let pt = ui.painter();
        jewel(pt, f, f.p(990.0, 26.0), sus, AMBER);
        pt.text(f.p(1003.0, 26.0), Align2::LEFT_CENTER, "PEDAL", f.font(9.5), if sus { TEXT } else { TEXT_DIM });
    }
    // Hush: everything to silence.
    let r = f.r(1074.0, 12.0, 86.0, 28.0);
    let resp = ui.interact(r, ui.id().with("hush"), Sense::click());
    if resp.clicked() {
        sh.panic.fetch_add(1, Ordering::Relaxed);
    }
    let pt = ui.painter();
    pt.rect_filled(r, CornerRadius::same((2.0 * f.s) as u8), if resp.hovered() { mix(PANEL, RED, 0.45) } else { FIELD });
    pt.rect_stroke(r, CornerRadius::same((2.0 * f.s) as u8), Stroke::new(f.len(1.0), mix(RED, LINE, 0.4)), StrokeKind::Inside);
    pt.text(r.center(), Align2::CENTER_CENTER, "HUSH", f.font(10.0), if resp.hovered() { TEXT } else { RED });
}

/// The harp: one tine per key, its glow the tip excursion.
fn harp(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame) {
    let sh = &ctx.params().ui;
    let k88 = ctx.get_param(P::Keys) > 0.5;
    let (lo, hi) = if k88 { (1usize, 88usize) } else { (8, 80) };
    let n = hi - lo + 1;
    let area = f.r(16.0, 64.0, 1148.0, 118.0);
    let pt = ui.painter();
    pt.rect_filled(area, CornerRadius::same((3.0 * f.s) as u8), WELL);
    pt.rect_stroke(area, CornerRadius::same((3.0 * f.s) as u8), Stroke::new(f.len(1.0), LINE), StrokeKind::Inside);
    pt.text(area.left_top() + vec2(f.len(10.0), f.len(9.0)), Align2::LEFT_TOP, "HARP", f.font(9.0), TEXT_DIM);
    // Tone-bar rail.
    let left = area.left() + f.len(26.0);
    let width = area.width() - f.len(52.0);
    let base_y = area.bottom() - f.len(20.0);
    pt.line_segment([pos2(left - f.len(8.0), base_y), pos2(left + width + f.len(8.0), base_y)], Stroke::new(f.len(2.0), TRACK));
    for i in 0..n {
        let key = lo + i;
        let x = left + width * (i as f32 + 0.5) / n as f32;
        // Tine length from the bass to the treble, as on the real harp.
        let t = (key as f32 - 1.0) / (KEYS as f32 - 1.0);
        let len = f.len(78.0 - 56.0 * t.powf(0.8));
        let lvl = load_f32(&sh.level[key - 1]);
        let glow = (lvl / 3.0).clamp(0.0, 1.0).powf(0.6);
        let col = if glow > 0.0 { mix(mix(TRACK, AMBER, 0.6), mix(AMBER, TEXT, 0.5), glow) } else { mix(TRACK, WELL, 0.15) };
        let sway = (lvl * 0.9).min(6.0) * f.s;
        if glow > 0.02 {
            pt.line_segment([pos2(x, base_y), pos2(x, base_y - len - sway)], Stroke::new(f.len(5.0), mix(WELL, AMBER, 0.25 + 0.35 * glow)));
        }
        pt.line_segment([pos2(x, base_y), pos2(x, base_y - len)], Stroke::new(f.len(1.6), col));
        // Pickup.
        let pick = Rect::from_center_size(pos2(x, base_y - len - f.len(6.0)), vec2(f.len(4.0), f.len(3.0)));
        pt.rect_filled(pick, CornerRadius::ZERO, if glow > 0.02 { col } else { LINE });
        // Octave marks (C).
        if (key + 8) % 12 == 0 {
            pt.text(pos2(x, base_y + f.len(9.0)), Align2::CENTER_CENTER, format!("C{}", (key + 8) / 12), f.font(8.5), TEXT_DIM);
        }
    }
}

fn piano_panel(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame) {
    let (x, y, w, h) = (16.0, 194.0, 452.0, 372.0);
    panel(ui.painter(), f, x, y, w, h, "Piano", "tines · hammers · pickups", AMBER);
    let a = AMBER;
    knob(ui, ctx, f, P::Voicing, "Voicing", x + 80.0, y + 104.0, 34.0, a);
    knob(ui, ctx, f, P::Gap, "Pickup", x + 226.0, y + 104.0, 34.0, a);
    knob(ui, ctx, f, P::Hammer, "Hammer", x + 372.0, y + 104.0, 34.0, a);
    knob(ui, ctx, f, P::Touch, "Touch", x + 80.0, y + 236.0, 26.0, a);
    knob(ui, ctx, f, P::Condition, "Condition", x + 226.0, y + 236.0, 26.0, a);
    knob(ui, ctx, f, P::A4, "Tuning", x + 372.0, y + 236.0, 26.0, a);
    let pt = ui.painter();
    groove(pt, f.p(x + 14.0, y + 304.0), f.p(x + w - 14.0, y + 304.0));
    pt.text(f.p(x + 16.0, y + 336.0), Align2::LEFT_CENTER, "KEYS", f.font(9.5), INK);
    segmented(ui, ctx, f, P::Keys, &["73", "88"], x + 56.0, y + 324.0, 120.0, 24.0, a);
    toggle(ui, ctx, f, P::Stretch, "Stretch tuning", x + 196.0, y + 324.0, 130.0, a);
    ui.painter().text(f.p(x + w - 16.0, y + 336.0), Align2::RIGHT_CENTER, "CC64 pedal", f.font(9.0), INK_DIM);
}

fn stage_panel(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame) {
    let (x, y, w, h) = (480.0, 194.0, 236.0, 372.0);
    panel(ui.painter(), f, x, y, w, h, "Stage", "nameboard", TEAL);
    let a = TEAL;
    knob(ui, ctx, f, P::StageVolume, "Volume", x + 64.0, y + 104.0, 34.0, a);
    knob(ui, ctx, f, P::BassBoost, "Bass Boost", x + 172.0, y + 104.0, 34.0, a);
    knob(ui, ctx, f, P::Harp, "Harp", x + 64.0, y + 236.0, 26.0, a);
    knob(ui, ctx, f, P::Cable, "Cable", x + 172.0, y + 236.0, 26.0, a);
    groove(ui.painter(), f.p(x + 14.0, y + 304.0), f.p(x + w - 14.0, y + 304.0));
    segmented(ui, ctx, f, P::Board, &["Original", "Modded", "Direct"], x + 14.0, y + 324.0, 208.0, 24.0, a);
}

fn twin_panel(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame) {
    let (x, y, w, h) = (728.0, 194.0, 436.0, 372.0);
    panel(ui.painter(), f, x, y, w, h, "Twin Reverb", "vibrato channel", BLUE);
    let a = BLUE;
    let r = 24.0;
    let step = 81.0;
    let row1 = y + 104.0;
    knob(ui, ctx, f, P::AmpVolume, "Volume", x + 56.0, row1, r, a);
    knob(ui, ctx, f, P::Treble, "Treble", x + 56.0 + step, row1, r, a);
    knob(ui, ctx, f, P::Middle, "Middle", x + 56.0 + 2.0 * step, row1, r, a);
    knob(ui, ctx, f, P::Bass, "Bass", x + 56.0 + 3.0 * step, row1, r, a);
    knob(ui, ctx, f, P::Reverb, "Reverb", x + 56.0 + 4.0 * step, row1, r, a);
    let row2 = y + 236.0;
    knob(ui, ctx, f, P::Speed, "Speed", x + 56.0, row2, r, a);
    knob(ui, ctx, f, P::Intensity, "Intensity", x + 56.0 + step, row2, r, a);
    knob(ui, ctx, f, P::Mic, "Mic", x + 56.0 + 2.0 * step, row2, r, a);
    knob(ui, ctx, f, P::Noise, "Noise", x + 56.0 + 3.0 * step, row2, r, a);
    knob(ui, ctx, f, P::Output, "Output", x + 56.0 + 4.0 * step, row2, r, a);
    groove(ui.painter(), f.p(x + 14.0, y + 304.0), f.p(x + w - 14.0, y + 304.0));
    segmented(ui, ctx, f, P::Amp, &["Twin", "Line", "DI"], x + 16.0, y + 324.0, 210.0, 24.0, a);
    toggle(ui, ctx, f, P::Bright, "Bright", x + 300.0, y + 324.0, 118.0, a);
}

/// The preset list, drawn over everything while open.
fn browser(ui: &mut egui::Ui, ctx: &Ctx, f: &Frame, infos: &[ParamInfo]) {
    let open: bool = ui.ctx().data(|d| d.get_temp(browser_id())).unwrap_or(false);
    if !open {
        return;
    }
    let lib = presets::library();
    let cur = current_preset(ctx, infos);
    let area = f.r(300.0, 56.0, 580.0, 260.0);
    let veil = ui.interact(f.r(0.0, 0.0, WINDOW_W as f32, WINDOW_H as f32), ui.id().with("veil"), Sense::click());
    let mut close = veil.clicked();
    {
        let pt = ui.painter();
        pt.rect_filled(area, CornerRadius::same((3.0 * f.s) as u8), HEAD);
        pt.rect_stroke(area, CornerRadius::same((3.0 * f.s) as u8), Stroke::new(f.len(1.0), mix(AMBER, LINE, 0.5)), StrokeKind::Inside);
    }
    let mut cats: Vec<&str> = Vec::new();
    for p in lib {
        if !cats.contains(&p.category.as_str()) {
            cats.push(&p.category);
        }
    }
    let colw = 560.0 / cats.len().max(1) as f32;
    let tints = [AMBER, BLUE, TEAL, RED, TEXT_DIM];
    let mut tip = "";
    for (c, cat) in cats.iter().enumerate() {
        let x = 312.0 + c as f32 * colw;
        ui.painter().text(f.p(x + 6.0, 76.0), Align2::LEFT_CENTER, cat.to_uppercase(), f.font(9.5), tints[c % tints.len()]);
        for (row, (i, p)) in lib.iter().enumerate().filter(|(_, p)| p.category == *cat).enumerate() {
            let r = f.r(x, 90.0 + row as f32 * 26.0, colw - 8.0, 24.0);
            let resp = ui.interact(r, ui.id().with(("preset-item", i)), Sense::click());
            let on = cur == Some(i);
            let pt = ui.painter();
            if resp.hovered() || on {
                pt.rect_filled(r, CornerRadius::same((2.0 * f.s) as u8), if on { mix(PANEL, tints[c % tints.len()], 0.25) } else { PANEL });
            }
            pt.text(pos2(r.left() + f.len(8.0), r.center().y), Align2::LEFT_CENTER, &p.name, f.font(11.5), TEXT);
            if resp.hovered() {
                tip = &p.comment;
            }
            if resp.clicked() {
                apply_preset(ctx, infos, p);
                close = true;
            }
        }
    }
    ui.painter().text(f.p(314.0, 302.0), Align2::LEFT_CENTER, tip, f.font(9.5), TEXT_DIM);
    if close {
        ui.ctx().data_mut(|d| d.insert_temp(browser_id(), false));
    }
}

