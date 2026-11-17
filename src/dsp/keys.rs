//! What each of the 88 tone generators is made of.
//!
//! Every number here has a source (tags as in docs/CIRCUIT.md):
//! - tine lengths: the Service Manual's tine cutting chart, measured
//!   [MAN], cross-checked against Gabrielli et al. 2020 Fig. 3 [ART];
//! - tine Ø 1.5 mm steel [ART Shear, US 4,040,321];
//! - tuning-spring mass: whatever makes the measured length sound the key
//!   (Rayleigh, tip mass) [INFER];
//! - mode ratios 0.48 / 7.1 / 20.4 / 39.7 / 62.9 / 93.1 and their spread,
//!   and the 88-key bottom keys without tone bar [ART Gabrielli];
//! - fundamental decay from Shear's measured Q of a 1974 Mark I [ART];
//! - inharmonic decay ≈ 3.8·f^0.41 dB/s, fitted to Gabrielli Table II [INFER];
//! - hammer tip zones (durometer 30/50/70/90/wrapped) and striking line
//!   57 → 3 mm [MAN ch. 4]; escapement is implicit in the strike;
//! - stretch tuning chart [MAN fig. 5-4].
//!
//! The unit's own spread (each tine voiced and regulated by hand) is a
//! fixed seed: the same piano every time you open it. CONDITION scales
//! how far it has drifted since.

use super::{KEYS, Rng, key_hz};

/// Hunt–Crossley base stiffness (durometer-30 zone), N/m^β.
pub const K_BASE: f64 = 1.6e10;
/// Strike participation of the inharmonic modes relative to the
/// fundamental's (the uniform-cantilever shapes put mode 3 on a node at
/// the striking line, which the real tine does not show), and the sub-fundamental's mass ratio: the real tine
/// is tapered and loaded by its spring (measured ratios 7.1/20.4/39.7, not
/// 6.27/17.5/34.4), which moves its nodes. Calibrated so a pp F3 attack
/// matches Gabrielli et al. 2020 Fig. 5 at the pickup output [DESIGN←ART].
pub const PART: [f64; 5] = [0.0134, 2.55, 8.4, 8.4, 8.4];
pub const SUB_MASS: f64 = 3.0;

/// Modes per tone generator.
pub const M: usize = 9;
pub const FUND_A: usize = 0;
pub const FUND_B: usize = 1;
pub const SUB: usize = 2;
pub const HORIZ: usize = 3;
/// Inharmonic tine modes start here (the first four ring longest).
pub const INHARM: usize = 4;

/// Tine length from the cutting chart, mm (key 1..88).
const CHART_MM: [f32; KEYS] = [
    157.0, 157.0, 157.0, 157.0, 157.0, 157.0, 157.0, 157.0, 154.0, 150.1, 145.3, 142.3, 137.2, 132.9, 130.1, 127.0, 122.7, 118.9, 114.3, 111.5, 109.2,
    105.7, 103.9, 101.6, 99.3, 96.3, 93.7, 90.4, 87.9, 85.6, 82.3, 80.3, 78.2, 77.0, 74.9, 71.9, 70.6, 68.3, 67.0, 65.3, 64.2, 62.2, 59.9, 58.1, 56.4, 53.8,
    53.3, 53.3, 50.5, 49.8, 48.2, 46.5, 45.4, 46.5, 44.2, 42.6, 42.6, 41.4, 39.9, 38.8, 37.6, 36.3, 35.3, 34.8, 34.0, 33.2, 32.0, 31.0, 30.5, 29.7, 28.7,
    27.7, 27.7, 26.4, 25.6, 25.1, 24.6, 24.1, 23.1, 22.6, 22.3, 21.6, 21.0, 20.5, 19.8, 19.3, 18.8, 18.0,
];

/// Cantilever eigenvalues βL for modes 1..6.
const BETA_L: [f64; 6] = [1.875_104, 4.694_091, 7.854_757, 10.995_541, 14.137_168, 17.278_76];

/// Tip-normalised cantilever mode shape at relative position x (0 = clamp).
fn shape(n: usize, x: f64) -> f64 {
    let b = BETA_L[n];
    let sig = (b.cosh() + b.cos()) / (b.sinh() + b.sin());
    let f = |x: f64| (b * x).cosh() - (b * x).cos() - sig * ((b * x).sinh() - (b * x).sin());
    f(x) / f(1.0)
}

/// Stretch chart of the Service Manual (fig. 5-4), cents, by key.
fn stretch_cents(key: usize) -> f64 {
    const P: [(f64, f64); 13] =
        [(1.0, -20.0), (13.0, -7.0), (25.0, -4.0), (37.0, -2.5), (46.0, -1.0), (49.0, 0.0), (55.0, 1.0), (61.0, 3.0), (67.0, 6.0), (73.0, 10.0), (79.0, 17.0), (85.0, 25.0), (88.0, 30.0)];
    let k = key as f64;
    for w in P.windows(2) {
        if k <= w[1].0 {
            let t = (k - w[0].0) / (w[1].0 - w[0].0);
            return w[0].1 + t * (w[1].1 - w[0].1);
        }
    }
    30.0
}

/// Fundamental decay, dB/s, from Shear's Q (1974 Mark I): Q → 8.686·πf/Q.
fn fund_decay(f0: f64) -> f64 {
    const S: [(f64, f64); 5] = [(77.8, 949.0), (155.6, 731.0), (311.1, 1520.0), (622.3, 2175.0), (1244.5, 1761.0)];
    let r: [(f64, f64); 5] = std::array::from_fn(|i| (S[i].0.ln(), (8.686 * std::f64::consts::PI * S[i].0 / S[i].1).ln()));
    let x = f0.ln();
    let y = if x <= r[0].0 {
        r[0].1 - 0.2 * (r[0].0 - x)
    } else if x >= r[4].0 {
        r[4].1 + 1.3 * (x - r[4].0)
    } else {
        let mut y = r[0].1;
        for w in r.windows(2) {
            if x <= w[1].0 {
                y = w[0].1 + (x - w[0].0) / (w[1].0 - w[0].0) * (w[1].1 - w[0].1);
                break;
            }
        }
        y
    };
    y.exp()
}

#[derive(Clone, Copy, Default)]
pub struct Mode {
    pub hz: f64,
    /// Amplitude decay, dB/s (tine free, damper lifted).
    pub decay: f64,
    /// Tip-normalised shape at the strike point (participation).
    pub at_strike: f64,
    /// Modal mass for tip-normalised coordinates, kg.
    pub mass: f64,
    pub on: bool,
}

/// The playing-time controls that shape the tone generators.
#[derive(Clone, Copy, PartialEq)]
pub struct Regulation {
    /// 0 = on the pole axis (bell), ~0.45 = factory, 1 = far below (deep).
    pub voicing: f32,
    /// Tip-to-pole gap, mm.
    pub gap_mm: f32,
    /// Hammer tip hardness scale (1 = factory neoprene).
    pub hammer: f32,
    /// 0 = freshly regulated, 1 = forty years on the road.
    pub condition: f32,
    pub a4: f32,
    pub stretch: bool,
}

#[derive(Clone, Copy, Default)]
pub struct KeyModel {
    pub f0: f64,
    pub modes: [Mode; M],
    pub hammer_mass: f64,
    /// Hunt–Crossley stiffness (N/m^β) and loss (s/m).
    pub k_contact: f64,
    pub lambda: f64,
    pub beta: f64,
    /// Share of the hammer blow that goes into the horizontal polarisation.
    pub horiz_share: f64,
    /// Damper felt, dB/s.
    pub damper: f64,
    /// Vertical rest offset of the tip from the pole axis, mm.
    pub offset_mm: f32,
    pub gap_mm: f32,
    /// Series/parallel harp: polarity of the group, divided by group size.
    pub harp_gain: f32,
    /// Distance along the harp for the sustain-pedal halo, 0..1.
    pub harp_pos: f32,
}

/// The fixed spread of this particular piano (seeded).
pub struct Unit {
    g: [[f32; 16]; KEYS],
    len_mm: [f64; KEYS],
}

impl Unit {
    pub fn new(serial: u32) -> Self {
        let mut r = Rng::new(serial.wrapping_mul(2_654_435_761) ^ 0x7313_0000);
        let g = std::array::from_fn(|_| std::array::from_fn(|_| r.gauss()));
        // Smooth the chart's pixel noise (monotone, 3-point mean).
        let mut len = [0.0f64; KEYS];
        for k in 0..KEYS {
            let a = CHART_MM[k.saturating_sub(1)];
            let b = CHART_MM[k];
            let c = CHART_MM[(k + 1).min(KEYS - 1)];
            len[k] = f64::from(a + b + c) / 3.0;
        }
        for k in 1..KEYS {
            len[k] = len[k].min(len[k - 1]);
        }
        Self { g, len_mm: len }
    }

    pub fn tine_mm(&self, key: usize) -> f64 {
        self.len_mm[key - 1]
    }

    /// Build key `key` (1..=88).
    pub fn model(&self, key: usize, reg: &Regulation, fs: f64) -> KeyModel {
        let g = &self.g[key - 1];
        let cond = f64::from(reg.condition);
        let no_bar = key <= 7; // 88-key bottom keys: no tone bar [ART Gabrielli]
        // --- pitch -------------------------------------------------------
        let mut cents = if reg.stretch { stretch_cents(key) } else { 0.0 };
        cents += f64::from(g[0]) * (0.6 + 6.0 * cond); // hand tuning, then drift
        let f0 = key_hz(key) * f64::from(reg.a4) / 440.0 * 2f64.powf(cents / 1200.0);
        // --- tine --------------------------------------------------------
        let len = self.len_mm[key - 1] / 1000.0;
        let r = 0.75e-3;
        let m_tine = 7850.0 * std::f64::consts::PI * r * r * len;
        let c_eb = (200e9f64 / 7850.0).sqrt() * r / 2.0;
        let f_bare = BETA_L[0] * BETA_L[0] / (2.0 * std::f64::consts::PI) * c_eb / (len * len);
        let m_spring = (0.2427 * m_tine * ((f_bare / f0).powi(2) - 1.0)).max(0.04 * m_tine);
        let m1 = 0.25 * m_tine + 0.8 * m_spring;
        let mh = 0.25 * m_tine + 0.25 * m_spring;
        // --- strike -----------------------------------------------------
        // Striking line: 57 mm from the tone generator in the extreme bass,
        // 3 mm in the extreme treble; hammer tip ~5 mm wide.
        let t = (key as f64 - 1.0) / 87.0;
        // The manual's figure is to the tip's leading edge; the blow lands
        // ~3 mm further, at the centre of the tip [INFER].
        let strike_mm = 57.0 * (1.0 - t) + 3.0 * t + 3.0 + f64::from(g[1]) * 1.5 * (0.3 + cond);
        let xs = (strike_mm / 1000.0 / len).clamp(0.04, 0.6);
        let half_w = (2.5e-3 / len).min(0.08);
        let part = |n: usize| (0..5).map(|i| shape(n, (xs - half_w + half_w * 0.5 * i as f64).max(0.0))).sum::<f64>() / 5.0;
        // --- modes --------------------------------------------------------
        let base = if no_bar { 2.5 } else { fund_decay(f0) };
        let dead = if f64::from(g[2]) > 1.6 { 1.0 + 1.5 * cond } else { 1.0 }; // a tired Torrington tine
        let base = base * (1.0 + 0.25 * cond * f64::from(g[3]).abs()) * dead;
        let mut modes = [Mode::default(); M];
        let p1 = part(0);
        // The fork: one fundamental that drains fast into the tone bar and
        // one that it gives back slowly, a fraction of a hertz apart.
        let beat = if no_bar { 0.0 } else { (0.03 + 0.25 * cond) * (0.4 + f64::from(g[4]).abs()) };
        modes[FUND_A] = Mode { hz: f0, decay: base * 1.8, at_strike: p1, mass: m1 / 0.62, on: true };
        modes[FUND_B] = Mode { hz: f0 + beat, decay: base * 0.55, at_strike: p1, mass: m1 / 0.38, on: !no_bar };
        // Sub-fundamental: the tone bar's own mode [ART Gabrielli].
        let sub_ratio = (0.48 + 0.19 * f64::from(g[5])).clamp(0.3, 0.86);
        let sub_decay = (9.1 * (f0 / 43.65).powf(1.96)).clamp(6.0, 400.0);
        modes[SUB] = Mode { hz: f0 * sub_ratio, decay: sub_decay, at_strike: p1, mass: m1 * SUB_MASS * (1.0 + 0.8 * f64::from(g[6]).abs()), on: !no_bar };
        let (ratios, sds): ([f64; 5], [f64; 5]) =
            if no_bar { ([9.6, 29.1, 52.4, 80.0, 115.0], [0.2, 0.8, 1.5, 2.5, 3.0]) } else { ([7.1, 20.4, 39.7, 62.9, 93.1], [0.3, 0.4, 0.9, 1.9, 2.7]) };
        for i in 0..5 {
            let hz = f0 * (ratios[i] + sds[i] * f64::from(g[7 + i]));
            let low_only = i >= 3 && key > 30;
            let on = hz < 12_000.0 && hz < 0.42 * fs && !low_only;
            let decay = 3.77 * hz.powf(0.41) * (0.8 + 0.3 * f64::from(g[12]).abs());
            modes[INHARM + i] = Mode { hz, decay, at_strike: p1 * PART[i], mass: mh, on };
        }
        // Horizontal polarisation of the fundamental [ART Pfeifle].
        let dh = (0.0006 + 0.0025 * cond) * f64::from(g[13]);
        modes[HORIZ] = Mode { hz: f0 * (1.0 + dh), decay: base * 0.7, at_strike: p1, mass: m1, on: true };
        // --- hammer -------------------------------------------------------
        let zone = match key {
            1..=30 => 1.0,
            31..=40 => 1.8,
            41..=50 => 3.2,
            51..=64 => 5.6,
            _ => 9.0,
        };
        let hammer_mass = 0.0068 - 0.0026 * t;
        let k_contact = K_BASE * zone * f64::from(reg.hammer) * (1.0 + 0.08 * f64::from(g[14]) * (0.3 + cond));
        // --- damper --------------------------------------------------------
        let damper = (70.0 + 220.0 * t) * (1.0 - 0.45 * cond * (0.5 + 0.5 * f64::from(g[15]).abs()).min(1.0));
        // --- pickup --------------------------------------------------------
        let v = f64::from(reg.voicing);
        let offset = 0.25 + 4.75 * v.powf(1.6) + f64::from(g[8]) * (0.08 + 0.35 * cond);
        let gap = f64::from(reg.gap_mm) * (1.0 + f64::from(g[9]) * (0.03 + 0.12 * cond));
        // Harp wiring: 73-key harps use a group of 4 at the bottom (keys
        // 8..11) then groups of 3; the 88's extra keys come in 3s and a 4.
        let (group, size) = if key >= 12 {
            ((key - 12) / 3 + 1, 3)
        } else if key >= 8 {
            (0, 4)
        } else if key >= 4 {
            (100, 4)
        } else {
            (101, 3)
        };
        let harp_gain = if group % 2 == 0 { 1.0 } else { -1.0 } / size as f32;
        KeyModel {
            f0,
            modes,
            hammer_mass,
            k_contact,
            lambda: 0.45,
            beta: 2.3,
            horiz_share: 0.03 + 0.10 * (0.3 + cond) * f64::from(g[10]).abs(),
            damper,
            offset_mm: offset.clamp(0.0, 8.0) as f32,
            gap_mm: gap.clamp(0.4, 4.0) as f32,
            harp_gain,
            harp_pos: t as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cutting chart and a 1.5 mm steel beam put the bare tine near its
    /// note through the middle of the keyboard (docs/PICKUP_ANALYSIS.md §1),
    /// so the tuning springs come out light there and heavy in the bass.
    #[test]
    fn springs_are_heavy_in_the_bass_light_in_the_middle() {
        let u = Unit::new(1975);
        let reg = Regulation { voicing: 0.45, gap_mm: 2.0, hammer: 1.0, condition: 0.0, a4: 440.0, stretch: false };
        let k1 = u.model(8, &reg, 96_000.0);
        let k2 = u.model(40, &reg, 96_000.0);
        assert!(k1.modes[0].mass > 2.0 * k2.modes[0].mass);
    }

    #[test]
    fn cantilever_shapes_are_tip_normalised() {
        for n in 0..6 {
            assert!((shape(n, 1.0) - 1.0).abs() < 1e-9);
            assert!(shape(n, 0.0).abs() < 1e-9);
        }
    }
}
