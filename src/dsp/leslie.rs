//! A Leslie 147 rotary speaker, the way a Rhodes met one: a combo preamp
//! (one 12AX7 stage) into the 147's 40 W amplifier (12AU7 long-tail phase
//! splitter, two 6550s push-pull), an 800 Hz passive crossover, a rotating
//! horn above and a 15" woofer firing down into a rotating drum below,
//! inside a wooden cabinet, picked up by two microphones.
//!
//! The rotors are geometry, not an LFO: each source moves on its circle,
//! and every path to each microphone (direct, and mirrored off the side
//! and back walls) is read from a delay line at its true, moving length,
//! so the Doppler shift, the amplitude swing and the chorus of reflections
//! each arriving with its own shift come out of the same calculation
//! (after Smith, Serafin, Abel & Berners, DAFx 2002). The horn beams its
//! treble: the brightness of each path follows the angle between the horn
//! mouth and that path.
//!
//! Speeds: John Fisher's measurement of a 147 (horn 48/400 rpm, drum
//! 40/342 rpm). Spin-up and spin-down: the horn in about a second, the
//! heavy drum in about five (Leslie 122H manual and owners' timings) [MAN,
//! ART]. Geometry and directivity [DESIGN]; see `docs/CIRCUIT.md` §11.

use super::amp::Power;
use super::triode::{Stage, T12AX7};
use super::{Biquad, Bq, HighPass, LowPass};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rotor {
    /// Chorale.
    Slow,
    /// Tremolo.
    Fast,
    /// Motors off: the rotors coast to rest.
    Stop,
}

#[derive(Clone, Copy, PartialEq)]
pub struct LeslieKnobs {
    pub rotor: Rotor,
    /// Combo preamp drive, 0..1.
    pub drive: f32,
    /// Horn against drum, −1 (drum) .. +1 (horn).
    pub balance: f32,
    /// Angle between the two microphones, degrees (0 = mono).
    pub width: f32,
}

impl Default for LeslieKnobs {
    fn default() -> Self {
        Self { rotor: Rotor::Slow, drive: 0.35, balance: 0.0, width: 120.0 }
    }
}

// ---------------------------------------------------------------------------
// The amplifier (runs at the oversampled rate)
// ---------------------------------------------------------------------------

/// Combo preamp and the 147's power amp. Input: volts at the jack.
/// Output: volts on the speaker line.
pub struct LesliePower {
    in_hp: HighPass,
    pre: Stage,
    c_hp: HighPass,
    power: Power,
    ot_hp: HighPass,
    ot_lp: LowPass,
}

impl LesliePower {
    pub fn new(fs: f64) -> Self {
        Self {
            in_hp: HighPass::new(10.0, fs),
            pre: Stage::new(T12AX7, 300.0, 100e3, 1500.0, 68e3, 60.0),
            c_hp: HighPass::new(25.0, fs),
            power: Power::new(fs),
            ot_hp: HighPass::new(35.0, fs),
            ot_lp: LowPass::new(14_000.0, fs),
        }
    }

    #[inline]
    pub fn run(&mut self, vin: f32, drive: f32, noise: f32) -> f32 {
        // Drive: the preamp's level control (audio taper) into the 147's
        // line input, which needs only a fraction of a volt to clip.
        let d = drive.clamp(0.0, 1.0);
        let g = 0.04 + 2.0 * d * d * d;
        let a = self.c_hp.run(self.pre.run(self.in_hp.run(vin)));
        let v = self.power.run(g * a, 0.004 * noise);
        self.ot_lp.run(self.ot_hp.run(v))
    }

    pub fn clear(&mut self) {
        self.in_hp.clear();
        self.c_hp.clear();
        self.power.clear();
        self.ot_hp.clear();
        self.ot_lp.clear();
    }
}

// ---------------------------------------------------------------------------
// The rotors (host rate)
// ---------------------------------------------------------------------------

const SOUND: f32 = 343.0;
const LINE: usize = 4096;
const MASK: usize = LINE - 1;

/// Horn mouth radius and drum scoop radius (m).
const R_HORN: f32 = 0.15;
const R_DRUM: f32 = 0.12;
/// Microphones from the rotor axis (m).
const MIC_D: f32 = 0.62;
/// Inside half-width and depth to the back wall (m).
const WALL_X: f32 = 0.27;
const WALL_BACK: f32 = 0.25;
/// Common path length removed from every delay (m), so the latency is
/// only what the geometry adds beyond it.
const BASE: f32 = 0.40;

/// rpm → revolutions per second: [slow, fast] (Fisher, 147).
const HORN_RPS: [f32; 2] = [48.0 / 60.0, 400.0 / 60.0];
const DRUM_RPS: [f32; 2] = [40.0 / 60.0, 342.0 / 60.0];
/// Time to settle (s): [spin up, spin down].
const HORN_T: [f32; 2] = [1.1, 1.5];
const DRUM_T: [f32; 2] = [5.5, 5.0];

struct Delay {
    buf: Vec<f32>,
    w: usize,
}

impl Delay {
    fn new() -> Self {
        Self { buf: vec![0.0; LINE], w: 0 }
    }
    #[inline]
    fn push(&mut self, x: f32) {
        self.w = (self.w + 1) & MASK;
        self.buf[self.w] = x;
    }
    /// Read `d` samples back (fractional, cubic Hermite).
    #[inline]
    fn read(&self, d: f32) -> f32 {
        let d = d.clamp(1.0, (LINE - 4) as f32);
        let i = d as usize;
        let t = d - i as f32;
        let at = |k: usize| self.buf[(self.w + LINE - k) & MASK];
        let (y0, y1, y2, y3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);
        ((c3 * t + c2) * t + c1) * t + y1
    }
    fn clear(&mut self) {
        self.buf.fill(0.0);
    }
}

/// A source on a rotor: one of its paths to one microphone.
#[derive(Clone, Copy)]
struct Path {
    /// Mirror the source across x = ±WALL_X (1 / −1) or the back wall (0
    /// with `back`), or not at all.
    mirror_x: f32,
    back: bool,
    gain: f32,
}

const PATHS: [Path; 4] = [
    Path { mirror_x: 0.0, back: false, gain: 1.0 },
    Path { mirror_x: 1.0, back: false, gain: 0.38 },
    Path { mirror_x: -1.0, back: false, gain: 0.38 },
    Path { mirror_x: 0.0, back: true, gain: 0.30 },
];

/// Geometry of one path at rotor angle `th` (0 = facing the front):
/// (length in m, cosine between the source's facing and the path).
#[inline]
fn path(p: &Path, r: f32, s: f32, c: f32, mic: (f32, f32)) -> (f32, f32) {
    // Source on its circle, facing outward; y towards the front.
    let (mut sx, mut sy, mut ux, mut uy) = (r * s, r * c, s, c);
    if p.mirror_x != 0.0 {
        sx = 2.0 * p.mirror_x * WALL_X - sx;
        ux = -ux;
    }
    if p.back {
        sy = -2.0 * WALL_BACK - sy;
        uy = -uy;
    }
    let (dx, dy) = (mic.0 - sx, mic.1 - sy);
    let len = (dx * dx + dy * dy).sqrt();
    // Clamped: rounding can push the cosine a hair past ±1.
    (len, ((ux * dx + uy * dy) / len.max(1e-4)).clamp(-1.0, 1.0))
}

pub struct Leslie {
    fs: f32,
    // Crossover and drivers.
    lo_lp: [Biquad; 2],
    hi_hp: [Biquad; 2],
    horn_lp: Biquad,
    horn_peak: Biquad,
    drum_hp: Biquad,
    drum_box: Biquad,
    horn_line: Delay,
    drum_line: Delay,
    /// Per microphone and path: the one-pole that darkens the horn when it
    /// faces away.
    beam: [[f32; 4]; 2],
    // Rotors.
    pub horn_angle: f32,
    pub drum_angle: f32,
    pub horn_rps: f32,
    pub drum_rps: f32,
    mics: [(f32, f32); 2],
    width: f32,
}

impl Leslie {
    pub fn new(fs: f64) -> Self {
        let bw = std::f64::consts::FRAC_1_SQRT_2;
        let mut l = Self {
            fs: fs as f32,
            lo_lp: [Biquad::new(Bq::Lp, 800.0, bw, fs), Biquad::new(Bq::Lp, 800.0, bw, fs)],
            hi_hp: [Biquad::new(Bq::Hp, 800.0, bw, fs), Biquad::new(Bq::Hp, 800.0, bw, fs)],
            // Compression driver into the horn: falls off above ~6 kHz,
            // with the horn's own lift around 2 kHz.
            horn_lp: Biquad::new(Bq::Lp, 6000.0, 0.8, fs),
            horn_peak: Biquad::new(Bq::Peak(2.5), 2000.0, 1.2, fs),
            drum_hp: Biquad::new(Bq::Hp, 45.0, 0.7, fs),
            // The closed wooden box under the drum.
            drum_box: Biquad::new(Bq::Peak(3.0), 110.0, 1.4, fs),
            horn_line: Delay::new(),
            drum_line: Delay::new(),
            beam: [[0.0; 4]; 2],
            horn_angle: 0.0,
            drum_angle: 1.9,
            horn_rps: HORN_RPS[0],
            drum_rps: DRUM_RPS[0],
            mics: [(0.0, MIC_D); 2],
            width: -1.0,
        };
        l.set_width(120.0);
        l
    }

    fn set_width(&mut self, deg: f32) {
        if (deg - self.width).abs() < 1e-3 {
            return;
        }
        self.width = deg;
        let h = 0.5 * deg.to_radians();
        // Left microphone on the left (−x), right on the right.
        self.mics = [(-MIC_D * h.sin(), MIC_D * h.cos()), (MIC_D * h.sin(), MIC_D * h.cos())];
    }

    /// Advance the motors by `n` samples towards `rotor`.
    fn motors(&mut self, rotor: Rotor, n: usize) {
        let dt = n as f32 / self.fs;
        let step = |v: &mut f32, target: f32, t: [f32; 2]| {
            // Settling time ≈ 3 time constants.
            let tau = if target > *v { t[0] } else { t[1] } / 3.0;
            *v += (target - *v) * (1.0 - (-dt / tau).exp());
        };
        let (h, d) = match rotor {
            Rotor::Slow => (HORN_RPS[0], DRUM_RPS[0]),
            Rotor::Fast => (HORN_RPS[1], DRUM_RPS[1]),
            Rotor::Stop => (0.0, 0.0),
        };
        step(&mut self.horn_rps, h, HORN_T);
        step(&mut self.drum_rps, d, DRUM_T);
    }

    /// Turn the rotors for `n` silent samples (the engine asleep).
    pub fn spin(&mut self, rotor: Rotor, n: usize) {
        self.motors(rotor, n);
        let tau = std::f32::consts::TAU;
        let t = n as f32 / self.fs;
        self.horn_angle = (self.horn_angle + tau * self.horn_rps * t) % tau;
        self.drum_angle = (self.drum_angle + tau * self.drum_rps * t) % tau;
    }

    /// Speaker-line volts in, (left, right) microphone signals out.
    /// `k` is read once per call; call with blocks of up to a few ms.
    pub fn run_block(&mut self, k: &LeslieKnobs, x: &[f32], left: &mut [f32], right: &mut [f32]) {
        self.motors(k.rotor, x.len());
        self.set_width(k.width);
        let tau = std::f32::consts::TAU;
        let dh = tau * self.horn_rps / self.fs;
        let dd = tau * self.drum_rps / self.fs;
        let per_m = self.fs / SOUND;
        let base = BASE * per_m;
        let gh = (1.0 + 0.5 * k.balance.clamp(-1.0, 1.0)).max(0.0);
        let gd = (1.0 - 0.5 * k.balance.clamp(-1.0, 1.0)).max(0.0);
        let w_scale = std::f32::consts::TAU / self.fs;
        for i in 0..x.len() {
            let s = x[i];
            let lo = self.lo_lp[0].run(s);
            let lo = self.lo_lp[1].run(lo);
            let hi = self.hi_hp[0].run(s);
            let hi = self.hi_hp[1].run(hi);
            // 800 Hz crossover, Linkwitz–Riley slopes so the two halves
            // sum flat when the rotors stand still [DESIGN: the 147's
            // passive network is not modelled part by part].
            let hi = self.horn_peak.run(self.horn_lp.run(hi));
            let lo = self.drum_box.run(self.drum_hp.run(lo));
            self.horn_line.push(hi);
            self.drum_line.push(lo);
            self.horn_angle += dh;
            if self.horn_angle > tau {
                self.horn_angle -= tau;
            }
            self.drum_angle += dd;
            if self.drum_angle > tau {
                self.drum_angle -= tau;
            }
            let (hs, hc) = self.horn_angle.sin_cos();
            let (ds, dc) = self.drum_angle.sin_cos();
            let mut out = [0.0f32; 2];
            for (m, o) in out.iter_mut().enumerate() {
                let mic = self.mics[m];
                let mut acc_h = 0.0;
                let mut acc_d = 0.0;
                for (j, p) in PATHS.iter().enumerate() {
                    // Horn: strong beaming, and its treble narrows further.
                    let (len, cs) = path(p, R_HORN, hs, hc, mic);
                    let face = 0.5 * (1.0 + cs);
                    let g = p.gain * (0.40 + 0.60 * face * face.sqrt()) * (MIC_D / len);
                    let y = self.horn_line.read(len * per_m - base);
                    let fc = 1800.0 + 7500.0 * face * face;
                    let a = (-fc * w_scale).exp();
                    let st = &mut self.beam[m][j];
                    *st = y + a * (*st - y);
                    acc_h += g * *st;
                    // Drum: the scoop throws the bass round, much less
                    // directional.
                    let (len, cs) = path(p, R_DRUM, ds, dc, mic);
                    let g = p.gain * (0.65 + 0.35 * 0.5 * (1.0 + cs)) * (MIC_D / len);
                    acc_d += g * self.drum_line.read(len * per_m - base);
                }
                *o = gh * acc_h + gd * acc_d;
            }
            left[i] = out[0];
            right[i] = out[1];
        }
    }

    pub fn clear(&mut self) {
        for b in self.lo_lp.iter_mut().chain(self.hi_hp.iter_mut()) {
            b.clear();
        }
        self.horn_lp.clear();
        self.horn_peak.clear();
        self.drum_hp.clear();
        self.drum_box.clear();
        self.horn_line.clear();
        self.drum_line.clear();
        self.beam = [[0.0; 4]; 2];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The horn's Doppler swing at full speed: a 2 kHz tone heard at one
    /// microphone wanders by about ±(2πr·f_rot/c)·f ≈ ±37 Hz.
    #[test]
    fn fast_horn_shifts_pitch() {
        let fs = 48_000.0;
        let mut l = Leslie::new(fs);
        let k = LeslieKnobs { rotor: Rotor::Fast, drive: 0.3, balance: 1.0, width: 180.0 };
        let n = 512;
        let mut x = vec![0.0f32; n];
        let (mut a, mut b) = (vec![0.0f32; n], vec![0.0f32; n]);
        let mut ph = 0.0f64;
        // Let the motors come up to speed.
        for _ in 0..(4.0 * fs) as usize / n {
            l.run_block(&k, &x, &mut a, &mut b);
        }
        assert!((l.horn_rps - 400.0 / 60.0).abs() < 0.1, "horn at {} rps", l.horn_rps);
        // Zero-crossing rate over short windows.
        let mut freqs = Vec::new();
        for _ in 0..60 {
            for s in x.iter_mut() {
                ph += 2000.0 / fs;
                *s = (std::f64::consts::TAU * ph).sin() as f32;
            }
            l.run_block(&k, &x, &mut a, &mut b);
            // Rising zero crossings, interpolated: periods over the window.
            let t: Vec<f32> = (1..n).filter(|&i| a[i - 1] <= 0.0 && a[i] > 0.0).map(|i| (i - 1) as f32 + a[i - 1] / (a[i - 1] - a[i])).collect();
            if t.len() > 2 {
                freqs.push((t.len() - 1) as f32 * fs as f32 / (t[t.len() - 1] - t[0]));
            }
        }
        let lo = freqs[4..].iter().cloned().fold(f32::MAX, f32::min);
        let hi = freqs[4..].iter().cloned().fold(0.0, f32::max);
        eprintln!("2 kHz through the fast horn: {lo:.0}..{hi:.0} Hz");
        assert!(hi - lo > 40.0, "no Doppler swing: {lo}..{hi} Hz");
        assert!(hi - lo < 400.0, "swing too wide: {lo}..{hi} Hz");
    }

    #[test]
    fn drum_is_slower_to_spin_up_than_the_horn() {
        let fs = 48_000.0;
        let mut l = Leslie::new(fs);
        let k = LeslieKnobs { rotor: Rotor::Fast, ..LeslieKnobs::default() };
        let n = 480;
        let x = vec![0.0f32; n];
        let (mut a, mut b) = (vec![0.0f32; n], vec![0.0f32; n]);
        for _ in 0..100 {
            l.run_block(&k, &x, &mut a, &mut b); // 1 s
        }
        let horn = l.horn_rps / HORN_RPS[1];
        let drum = l.drum_rps / DRUM_RPS[1];
        assert!(horn > 0.9, "horn only at {horn}");
        assert!(drum < 0.6, "drum already at {drum}");
    }
}

#[cfg(test)]
mod probe {
    use super::*;

    /// Amplitude swing (dB) of a steady tone at one microphone over a
    /// full turn, horn-only and drum-only, and L/R correlation.
    #[test]
    #[ignore]
    fn am_depth() {
        let fs = 48_000.0;
        for (hz, bal) in [(2000.0f64, 1.0f32), (150.0, -1.0)] {
            for width in [0.0f32, 120.0, 180.0] {
                let mut l = Leslie::new(fs);
                let k = LeslieKnobs { rotor: Rotor::Fast, drive: 0.3, balance: bal, width };
                let n = 480;
                let mut x = vec![0.0f32; n];
                let (mut a, mut b) = (vec![0.0f32; n], vec![0.0f32; n]);
                let mut ph = 0.0f64;
                let mut env = Vec::new();
                let (mut sl, mut sr, mut slr) = (0.0f64, 0.0f64, 0.0f64);
                for blk in 0..800 {
                    for s in x.iter_mut() {
                        ph += hz / fs;
                        *s = (std::f64::consts::TAU * ph).sin() as f32;
                    }
                    l.run_block(&k, &x, &mut a, &mut b);
                    if blk > 600 {
                        for c in a.chunks(96) {
                            env.push((c.iter().map(|v| v * v).sum::<f32>() / c.len() as f32).sqrt());
                        }
                        for (p, q) in a.iter().zip(&b) {
                            sl += f64::from(p * p);
                            sr += f64::from(q * q);
                            slr += f64::from(p * q);
                        }
                    }
                }
                let hi = env.iter().cloned().fold(0.0, f32::max);
                let lo = env.iter().cloned().fold(f32::MAX, f32::min);
                eprintln!("{hz:>5} Hz {} width {width:>3}°: swing {:.1} dB, L/R corr {:+.2}", if bal > 0.0 { "horn" } else { "drum" }, 20.0 * (hi / lo).log10(), slr / (sl * sr).sqrt());
            }
        }
    }
}
