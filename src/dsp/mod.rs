//! The circuit, stage by stage (see `docs/CIRCUIT.md`):
//!
//! key → hammer (neoprene, Hunt–Crossley contact) → tine + tone bar
//! (modal) → cone-tipped pickup (flux table) → harp (series/parallel,
//! R + L) → Stage nameboard (Bass Boost, Volume) → cable → Twin Reverb
//! Vibrato channel (7025 stages, TMB stack, spring reverb, optical
//! tremolo, 12AT7 phase inverter, 4×6L6GC with feedback and sag) →
//! 2×12 open-back cabinet and a close microphone; or a Leslie 147 (combo
//! preamp, 40 W amp, rotating horn and drum, two microphones).

pub mod amp;
pub mod cab;
pub mod engine;
pub mod keys;
pub mod leslie;
pub mod mna;
pub mod pickup;
pub mod pickup_table;
pub mod resample;
pub mod reverb;
pub mod stage;
pub mod triode;
pub mod voice;

/// Keys of the 88-key harp (A0..C8); the 73-key Stage uses 8..=80 (E1..E7).
pub const KEYS: usize = 88;
pub const MIDI_LOW: u8 = 21;

#[inline]
pub fn key_hz(key88: usize) -> f64 {
    440.0 * 2f64.powf((key88 as f64 - 49.0) / 12.0)
}

/// Small deterministic PRNG (xorshift32) with a cheap Gaussian.
#[derive(Clone)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        let mut r = Self(seed | 1);
        for _ in 0..6 {
            r.next_u32();
        }
        r
    }
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// Uniform in [0, 1).
    #[inline]
    pub fn uni(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }
    /// Approximately N(0, 1): sum of four uniforms, rescaled.
    #[inline]
    pub fn gauss(&mut self) -> f32 {
        let s = self.uni() + self.uni() + self.uni() + self.uni();
        (s - 2.0) * 1.732_050_8
    }
}

/// One-pole smoother that lands exactly on its target.
#[derive(Clone, Copy)]
pub struct Smooth {
    pub v: f32,
    pub k: f32,
}

impl Smooth {
    pub fn new(v: f32, k: f32) -> Self {
        Self { v, k }
    }
    /// Returns true while moving.
    #[inline]
    pub fn step(&mut self, target: f32) -> bool {
        let d = target - self.v;
        if d.abs() < 1e-5 {
            self.v = target;
            false
        } else {
            self.v += d * self.k;
            true
        }
    }
}

/// First-order high-pass (bilinear).
#[derive(Clone, Copy, Default)]
pub struct HighPass {
    b: f32,
    a: f32,
    x1: f32,
    y1: f32,
}

impl HighPass {
    pub fn new(fc: f64, fs: f64) -> Self {
        let mut h = Self::default();
        h.set(fc, fs);
        h
    }
    pub fn set(&mut self, fc: f64, fs: f64) {
        let k = (std::f64::consts::PI * fc / fs).tan();
        self.b = (1.0 / (1.0 + k)) as f32;
        self.a = ((1.0 - k) / (1.0 + k)) as f32;
    }
    #[inline]
    pub fn run(&mut self, x: f32) -> f32 {
        let y = self.b * (x - self.x1) + self.a * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
    pub fn clear(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }
}

/// First-order low-pass (bilinear).
#[derive(Clone, Copy, Default)]
pub struct LowPass {
    b: f32,
    a: f32,
    x1: f32,
    y1: f32,
}

impl LowPass {
    pub fn new(fc: f64, fs: f64) -> Self {
        let mut h = Self::default();
        h.set(fc, fs);
        h
    }
    pub fn set(&mut self, fc: f64, fs: f64) {
        let k = (std::f64::consts::PI * fc.min(0.49 * fs) / fs).tan();
        self.b = (k / (1.0 + k)) as f32;
        self.a = ((1.0 - k) / (1.0 + k)) as f32;
    }
    #[inline]
    pub fn run(&mut self, x: f32) -> f32 {
        let y = self.b * (x + self.x1) + self.a * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
    pub fn clear(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }
}

/// RBJ biquad (transposed direct form II).
#[derive(Clone, Copy, Default)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

#[derive(Clone, Copy)]
pub enum Bq {
    Lp,
    Hp,
    Peak(f64),
}

impl Biquad {
    pub fn new(kind: Bq, f: f64, q: f64, fs: f64) -> Self {
        let mut b = Self::default();
        b.set(kind, f, q, fs);
        b
    }
    pub fn set(&mut self, kind: Bq, f: f64, q: f64, fs: f64) {
        let w = 2.0 * std::f64::consts::PI * f.min(0.45 * fs) / fs;
        let (s, c) = w.sin_cos();
        let al = s / (2.0 * q);
        let (b0, b1, b2, a0, a1, a2) = match kind {
            Bq::Lp => ((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al),
            Bq::Hp => ((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al),
            Bq::Peak(db) => {
                let a = 10f64.powf(db / 40.0);
                (1.0 + al * a, -2.0 * c, 1.0 - al * a, 1.0 + al / a, -2.0 * c, 1.0 - al / a)
            }
        };
        self.b0 = (b0 / a0) as f32;
        self.b1 = (b1 / a0) as f32;
        self.b2 = (b2 / a0) as f32;
        self.a1 = (a1 / a0) as f32;
        self.a2 = (a2 / a0) as f32;
    }
    #[inline]
    pub fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
    pub fn clear(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}
