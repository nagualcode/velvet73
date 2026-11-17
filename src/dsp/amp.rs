//! Fender Twin Reverb (AA270 schematic), Vibrato channel — the one with
//! reverb and tremolo, where a Rhodes was plugged.
//!
//! V2A 7025 (100k / 1.5k bypassed, 68k‖68k input) → TMB tone stack
//! (250 pF, 0.1 µF, 0.047 µF, 250k/250k/10k, 100k slope) and the 1M audio
//! Volume with its 120 pF bright cap, solved as one nodal network with the
//! stage's plate resistance as source → V2B 7025 → spring reverb send and
//! dry path → mix → V4B 7025 → optical tremolo (neon + LDR shunt) →
//! 12AT7 long-tail phase inverter → 4×6L6GC push-pull, fixed bias −52 V,
//! global feedback from the 4 Ω tap (820 Ω / 100 Ω), supply sag through
//! the silicon rectifier and ~35 µF, output transformer.

use super::mna::Net;
use super::reverb::Tank;
use super::stage::audio;
use super::triode::{Stage, T12AX7};
use super::{HighPass, LowPass, Rng};

#[derive(Clone, Copy, PartialEq)]
pub struct AmpKnobs {
    /// Fender numbers 1..10 mapped to 0..1 rotation.
    pub volume: f32,
    pub treble: f32,
    pub middle: f32,
    pub bass: f32,
    pub bright: bool,
    pub reverb: f32,
    pub speed: f32,
    pub intensity: f32,
}

/// The tone stack + volume + bright network.
struct ToneStack {
    net: Net,
    e: [usize; 7],
}

impl ToneStack {
    fn new(fs: f64) -> Self {
        // 1 P (plate side), 2 T, 3 O (treble wiper), 4 B, 5 M, 6 X, 7 W (volume wiper).
        let mut net = Net::new(7, 1, 38e3, 7, fs);
        net.c(1, 2, 250e-12);
        let e0 = net.r(2, 3, 125e3);
        let e1 = net.r(3, 4, 125e3);
        let e2 = net.r(4, 5, 125e3);
        let e3 = net.r(5, 0, 5e3);
        net.r(1, 6, 100e3);
        net.c(6, 4, 0.1e-6);
        net.c(6, 5, 0.047e-6);
        let e4 = net.r(3, 7, 500e3);
        let e5 = net.c(3, 7, 120e-12);
        let e6 = net.r(7, 0, 500e3);
        Self { net, e: [e0, e1, e2, e3, e4, e5, e6] }
    }
    fn set(&mut self, k: &AmpKnobs) {
        let t = audio(f64::from(k.treble));
        let l = audio(f64::from(k.bass));
        let m = audio(f64::from(k.middle));
        let v = audio(f64::from(k.volume));
        let e = self.e;
        self.net.set(e[0], (250e3 * (1.0 - t)).max(1.0));
        self.net.set(e[1], (250e3 * t).max(1.0));
        self.net.set(e[2], (250e3 * l).max(1.0));
        self.net.set(e[3], (10e3 * m).max(1.0));
        self.net.set(e[4], (1e6 * (1.0 - v)).max(1.0));
        self.net.set(e[5], if k.bright { 120e-12 } else { 1e-18 });
        self.net.set(e[6], (1e6 * v).max(1.0));
        self.net.build();
    }
}

/// Fender "Vibrato": a phase-shift oscillator lighting a neon lamp that
/// faces an LDR shunting the signal. The neon only strikes above a
/// threshold and the CdS cell follows light quickly, darkness slowly:
/// a choppy, asymmetric throb, not a sine.
struct Tremolo {
    phase: f64,
    ldr: f32,
    fs: f32,
}

impl Tremolo {
    fn new(fs: f64) -> Self {
        Self { phase: 0.0, ldr: 0.0, fs: fs as f32 }
    }
    #[inline]
    fn gain(&mut self, speed: f32, intensity: f32) -> f32 {
        let hz = 1.2 + 8.3 * speed * speed;
        self.phase += f64::from(hz / self.fs);
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let s = (std::f64::consts::TAU * self.phase).sin() as f32;
        // The oscillator itself is not a clean sine: soft-clipped.
        let s = 1.4 * s / (1.0 + 0.4 * s.abs());
        let light = ((s - 0.25) / 0.75).clamp(0.0, 1.0);
        let k = if light > self.ldr { 1.0 / (0.006 * self.fs) } else { 1.0 / (0.07 * self.fs) };
        self.ldr += (light - self.ldr) * k;
        if intensity <= 0.0 {
            return 1.0;
        }
        // LDR from 5 MΩ dark to ~4 kΩ lit (log), in series with the
        // Intensity pot, shunting a ~100 kΩ source.
        let r_ldr = 5e6f32 * (4e3f32 / 5e6).powf(self.ldr);
        let r_int = 50e3 * (1.0 - audio(f64::from(intensity)) as f32);
        let z = r_ldr + r_int;
        z / (100e3 + z)
    }
}

/// 12AT7 phase inverter and the 6L6GC quartet with feedback and sag.
/// The open-loop curve (PI drive → push-pull plate currents → speaker
/// volts) is solved once into a table; per sample the feedback loop is a
/// two-step Newton solve on it, and the supply sag scales the ceiling.
struct Power {
    v_tab: Vec<f32>,
    i_tab: Vec<f32>,
    inv_step: f32,
    u: f32,
    sag: f32,
    i_avg: f32,
    k_avg: f32,
    ph: (f32, f32),
    rot: (f32, f32),
}

const BIAS: f32 = -52.0;
const VCO: f32 = -68.0;
const BETA_NFB: f32 = 0.109;
const A_PI: f32 = 22.0;
const U_MAX: f32 = 12.0;
const TAB: usize = 4097;

/// Plate current of one side (two tubes), A.
fn side(vg: f32) -> f32 {
    // Grid conduction: the PI cannot push the grid far above 0 V.
    let vg = if vg > 0.0 { 4.0 * (vg / 4.0).tanh() } else { vg };
    let x = (vg - VCO) / 4.0;
    let sp = if x > 20.0 { x } else { x.exp().ln_1p() };
    2.0 * 6.24e-4 * (4.0 * sp).powf(1.5)
}

/// tanh, rational approximation (smooth, monotone, |err| < 2.5 %).
#[inline]
fn tanh_fast(x: f32) -> f32 {
    let x = x.clamp(-3.0, 3.0);
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

impl Power {
    fn new(fs: f64) -> Self {
        let mut v_tab = Vec::with_capacity(TAB);
        let mut i_tab = Vec::with_capacity(TAB);
        for k in 0..TAB {
            let u = -U_MAX + 2.0 * U_MAX * k as f32 / (TAB - 1) as f32;
            let d = A_PI * u;
            let d = 85.0 * (d / 85.0).tanh() + 0.02 * d.abs().min(85.0); // LTP slight imbalance
            let ia = side(BIAS + d);
            let ib = side(BIAS - d);
            v_tab.push(44.7 * (ia - ib));
            i_tab.push(ia + ib);
        }
        Self {
            v_tab,
            i_tab,
            inv_step: (TAB - 1) as f32 / (2.0 * U_MAX),
            u: 0.0,
            sag: 1.0,
            i_avg: 0.16,
            k_avg: (1.0 / (0.04 * fs)) as f32,
            ph: (1.0, 0.0),
            rot: ((std::f64::consts::TAU * 120.0 / fs).cos() as f32, (std::f64::consts::TAU * 120.0 / fs).sin() as f32),
        }
    }

    /// Open-loop speaker volts, slope, and supply current at PI input `u`.
    #[inline]
    fn open(&self, u: f32) -> (f32, f32, f32) {
        let x = ((u + U_MAX) * self.inv_step).clamp(0.0, (TAB - 2) as f32);
        let i = x as usize;
        let t = x - i as f32;
        let dv = self.v_tab[i + 1] - self.v_tab[i];
        let v = self.v_tab[i] + t * dv;
        let cur = self.i_tab[i] + t * (self.i_tab[i + 1] - self.i_tab[i]);
        // Plate-voltage ceiling, lowered by the sagging supply.
        let vmax = 33.0 * self.sag;
        let a = v / vmax;
        let sat = vmax * tanh_fast(a);
        let a2 = a.clamp(-3.0, 3.0).powi(2);
        let dsat = (1.0 - a2 / 9.0).max(0.02) * dv * self.inv_step;
        (sat, dsat, cur)
    }

    #[inline]
    fn run(&mut self, x: f32, ripple: f32) -> f32 {
        // Solve u = x − β·V(u) (feedback into the PI tail), warm-started.
        let mut u = self.u;
        for _ in 0..2 {
            let (v, dv, _) = self.open(u);
            let f = u - x + BETA_NFB * v;
            u -= f / (1.0 + BETA_NFB * dv).max(0.05);
        }
        self.u = u;
        let (v, _, i) = self.open(u);
        // Supply: B+ sags with the average current (rectifier, transformer,
        // ~35 µF), and carries 120 Hz ripple proportional to the load.
        self.i_avg += (i - self.i_avg) * self.k_avg;
        let rip = if ripple > 0.0 {
            // 120 Hz phasor (rotation, renormalised once a cycle).
            let (c, sn) = (self.rot.0, self.rot.1);
            let (x0, y0) = self.ph;
            self.ph = (x0 * c - y0 * sn, x0 * sn + y0 * c);
            if self.ph.1.abs() < 1e-3 && self.ph.0 > 0.0 {
                let m = (self.ph.0 * self.ph.0 + self.ph.1 * self.ph.1).sqrt();
                self.ph = (self.ph.0 / m, self.ph.1 / m);
            }
            self.ph.1 * ripple * self.i_avg
        } else {
            0.0
        };
        self.sag = 1.0 - 0.11 * (self.i_avg - 0.16).max(0.0) + rip;
        v
    }

    fn clear(&mut self) {
        self.u = 0.0;
        self.sag = 1.0;
        self.i_avg = 0.16;
    }
}

pub struct Twin {
    in_hp: HighPass,
    v2a: Stage,
    ts: ToneStack,
    v2b: Stage,
    c_hp: HighPass,
    pub tank: Tank,
    v4b: Stage,
    c2_hp: HighPass,
    trem: Tremolo,
    power: Power,
    ot_hp: HighPass,
    ot_lp: LowPass,
    noise: Rng,
    knobs: Option<AmpKnobs>,
}

impl Twin {
    pub fn new(fs: f64) -> Self {
        Self {
            in_hp: HighPass::new(8.0, fs),
            v2a: Stage::new(T12AX7, 320.0, 100e3, 1500.0, 34e3, 12.0),
            ts: ToneStack::new(fs),
            v2b: Stage::new(T12AX7, 320.0, 100e3, 820.0, 47e3, 60.0),
            c_hp: HighPass::new(20.0, fs),
            tank: Tank::new(fs),
            v4b: Stage::new(T12AX7, 320.0, 100e3, 1500.0, 100e3, 60.0),
            c2_hp: HighPass::new(15.0, fs),
            trem: Tremolo::new(fs),
            power: Power::new(fs),
            ot_hp: HighPass::new(28.0, fs),
            ot_lp: LowPass::new(16_000.0, fs),
            noise: Rng::new(0x7713),
            knobs: None,
        }
    }

    pub fn set(&mut self, k: &AmpKnobs) {
        if self.knobs.as_ref() != Some(k) {
            self.ts.set(k);
            self.knobs = Some(*k);
        }
    }

    /// Input: volts at the jack. Output: volts across the 4 Ω speaker tap.
    /// `noise`: 0 = none, 1 = as built (tube hiss, B+ ripple).
    #[inline]
    pub fn run(&mut self, vin: f32, noise: f32) -> f32 {
        let k = self.knobs.expect("knobs");
        // Grid noise of the first 7025 plus the 34 kΩ input resistors:
        // ~2.5 µV rms over the audio band.
        let hiss = if noise > 0.0 { self.noise.gauss() * 1.2e-5 * noise } else { 0.0 };
        let a = self.v2a.run(self.in_hp.run(vin) + hiss);
        let w = self.ts.net.run(f64::from(a)) as f32;
        let b = self.c_hp.run(self.v2b.run(w));
        // Reverb send/return, mixed with the dry signal through the
        // channel's resistive network ahead of the third stage.
        let wet = self.tank.run(b);
        let mix = 0.3 * (b + 0.9 * audio(f64::from(k.reverb)) as f32 * wet);
        let c = self.c2_hp.run(self.v4b.run(mix));
        let g = self.trem.gain(k.speed, k.intensity);
        let pi_in = 0.04 * c * g;
        let v = self.power.run(pi_in, 0.004 * noise);
        self.ot_lp.run(self.ot_hp.run(v))
    }

    pub fn clear(&mut self) {
        self.in_hp.clear();
        self.ts.net.clear();
        self.c_hp.clear();
        self.tank.clear();
        self.c2_hp.clear();
        self.power.clear();
        self.ot_hp.clear();
        self.ot_lp.clear();
    }

    /// Small-signal gain of the first stage (for tests).
    pub fn first_stage_gain(&self) -> f32 {
        self.v2a.gain()
    }
}

#[cfg(test)]
mod bench {
    use super::*;

    /// `cargo test --release --lib amp_bench -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn amp_bench() {
        let fs = 96_000.0;
        let n = 96_000 * 4;
        let k = AmpKnobs { volume: 0.4, treble: 0.5, middle: 0.4, bass: 0.45, bright: false, reverb: 0.22, speed: 0.4, intensity: 0.0 };
        let mut t = Twin::new(fs);
        t.set(&k);
        let x: Vec<f32> = (0..n).map(|i| 0.1 * (i as f32 * 0.01).sin()).collect();
        let s = std::time::Instant::now();
        let mut acc = 0.0;
        for &v in &x {
            acc += t.run(v, 1.0);
        }
        println!("twin: {:.1} ms per s (acc {acc})", s.elapsed().as_secs_f64() * 1000.0 / 4.0);
        let mut tank = Tank::new(fs);
        let s = std::time::Instant::now();
        for &v in &x {
            acc += tank.run(v);
        }
        println!("tank: {:.1} ms per s", s.elapsed().as_secs_f64() * 1000.0 / 4.0);
        let mut ts = ToneStack::new(fs);
        ts.set(&k);
        let s = std::time::Instant::now();
        for &v in &x {
            acc += ts.net.run(f64::from(v)) as f32;
        }
        println!("tonestack: {:.1} ms per s", s.elapsed().as_secs_f64() * 1000.0 / 4.0);
        let mut p = Power::new(fs);
        let s = std::time::Instant::now();
        for &v in &x {
            acc += p.run(v, 0.004);
        }
        println!("power: {:.1} ms per s {acc}", s.elapsed().as_secs_f64() * 1000.0 / 4.0);
    }
}
