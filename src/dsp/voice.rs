//! One tone generator: hammer, tine + tone bar (modal), damper, pickup.
//!
//! Modes are exact damped rotations of z = q − i·v/ω (tip-normalised
//! coordinates, metres): free ringing costs four multiplies per mode. The
//! hammer is a mass on a nonlinear, lossy neoprene spring (Hunt–Crossley,
//! F = K c^β (1 + λ ċ)) pushing on the tine at the striking line; the
//! contact is integrated with four sub-steps per sample while it lasts
//! (1–3 ms). The pickup sees the tip's radial distance from the pole axis.

use super::keys::{FUND_A, FUND_B, HORIZ, INHARM, KeyModel, M, SUB};
use super::pickup::Slice;

const SUBSTEPS: usize = 4;
/// Pickup emf scale (volts per unit table flux per second).
pub const EMF: f32 = 5.2e-4;

pub struct Voice {
    pub km: KeyModel,
    slice: Slice,
    re: [f32; M],
    im: [f32; M],
    cr: [f32; M],
    ci: [f32; M],
    cr4: [f32; M],
    ci4: [f32; M],
    /// φ at the strike point, ω, 1/(M ω) per mode.
    phi: [f32; M],
    w: [f32; M],
    inv_mw: [f32; M],
    /// Damper: target engagement, current (0..1), per-sample gain at full.
    damp_target: f32,
    damp: f32,
    damp_full: f32,
    // Hammer.
    xh: f32,
    vh: f32,
    contact: bool,
    contact_left: u32,
    prev_phi: f32,
    pub active: bool,
    /// Modes being simulated: all `M` after a strike, the four long ones
    /// once the inharmonic modes have died away (or for a tine only
    /// nudged by the sustain-pedal halo).
    nm: usize,
    pub held: bool,
    /// Tip excursion, mm (for the editor), and the sleep check.
    pub level_mm: f32,
    quiet: u32,
    dt: f32,
    fs: f32,
}

impl Voice {
    pub fn new(km: KeyModel, fs: f64) -> Self {
        let mut v = Self {
            km,
            slice: Slice::at_gap(km.gap_mm),
            re: [0.0; M],
            im: [0.0; M],
            cr: [0.0; M],
            ci: [0.0; M],
            cr4: [0.0; M],
            ci4: [0.0; M],
            phi: [0.0; M],
            w: [0.0; M],
            inv_mw: [0.0; M],
            damp_target: 1.0,
            damp: 1.0,
            damp_full: 1.0,
            xh: 0.0,
            vh: 0.0,
            contact: false,
            contact_left: 0,
            prev_phi: 0.0,
            active: false,
            nm: INHARM,
            held: false,
            level_mm: 0.0,
            quiet: 0,
            dt: (1.0 / fs) as f32,
            fs: fs as f32,
        };
        v.retune(km, fs);
        v.prev_phi = v.slice.eval(km.offset_mm);
        v
    }

    /// New regulation (knobs moved). Keeps the vibration.
    pub fn retune(&mut self, km: KeyModel, fs: f64) {
        let gap_changed = (km.gap_mm - self.km.gap_mm).abs() > 1e-4;
        self.km = km;
        if gap_changed || self.slice.phi[0] == 0.0 {
            self.slice = Slice::at_gap(km.gap_mm);
        }
        let t = 1.0 / fs;
        for i in 0..M {
            let m = km.modes[i];
            if !m.on {
                self.cr[i] = 0.0;
                self.ci[i] = 0.0;
                self.cr4[i] = 0.0;
                self.ci4[i] = 0.0;
                self.phi[i] = 0.0;
                self.inv_mw[i] = 0.0;
                self.re[i] = 0.0;
                self.im[i] = 0.0;
                continue;
            }
            let w = 2.0 * std::f64::consts::PI * m.hz;
            let r = 10f64.powf(-m.decay * t / 20.0);
            let r4 = 10f64.powf(-m.decay * t / (20.0 * SUBSTEPS as f64));
            self.cr[i] = (r * (w * t).cos()) as f32;
            self.ci[i] = (r * (w * t).sin()) as f32;
            self.cr4[i] = (r4 * (w * t / SUBSTEPS as f64).cos()) as f32;
            self.ci4[i] = (r4 * (w * t / SUBSTEPS as f64).sin()) as f32;
            self.phi[i] = m.at_strike as f32;
            self.w[i] = w as f32;
            self.inv_mw[i] = (1.0 / (m.mass * w)) as f32;
        }
        self.damp_full = 10f64.powf(-km.damper * t / 20.0) as f32;
    }

    /// After a regulation change: the pickup's reference follows the tip
    /// where it is now (no step in the flux, no click).
    pub fn resync_pickup(&mut self) {
        let y = self.km.offset_mm + 1000.0 * self.vertical();
        let x = 1000.0 * self.re[HORIZ];
        self.prev_phi = self.slice.eval((y * y + x * x).sqrt());
    }

    /// The hammer arrives at `v0` m/s.
    pub fn strike(&mut self, v0: f32) {
        self.xh = self.strike_disp();
        self.vh = v0;
        self.contact = true;
        self.contact_left = (0.02 * self.fs) as u32; // give up after 20 ms
        self.active = true;
        self.nm = M;
        self.held = true;
        self.quiet = 0;
    }

    /// A small push through the harp rail (sustain-pedal halo).
    pub fn kick(&mut self, impulse: f32) {
        for i in [FUND_A, FUND_B, SUB] {
            self.im[i] -= impulse * self.phi[i] * self.inv_mw[i];
        }
        self.active = true;
        self.quiet = 0;
    }

    pub fn set_damper(&mut self, engaged: bool) {
        self.damp_target = if engaged { 1.0 } else { 0.0 };
        if !engaged {
            self.damp = 0.0;
        }
    }

    pub fn damped(&self) -> bool {
        self.damp_target > 0.5
    }

    pub fn silence(&mut self) {
        self.re = [0.0; M];
        self.im = [0.0; M];
        self.contact = false;
        self.active = false;
        self.nm = INHARM;
        self.level_mm = 0.0;
        self.prev_phi = self.slice.eval(self.km.offset_mm);
    }

    #[inline]
    fn strike_disp(&self) -> f32 {
        let mut s = 0.0;
        for i in 0..M {
            if i != HORIZ {
                s += self.phi[i] * self.re[i];
            }
        }
        s
    }

    /// Tip displacement in the hammer's plane, m.
    #[inline]
    fn vertical(&self) -> f32 {
        let mut s = self.re[FUND_A] + self.re[FUND_B] + self.re[SUB];
        for i in INHARM..self.nm {
            s += self.re[i];
        }
        s
    }

    #[inline]
    fn strike_vel(&self) -> f32 {
        let mut s = 0.0;
        for i in 0..M {
            if i != HORIZ {
                s -= self.phi[i] * self.w[i] * self.im[i];
            }
        }
        s
    }

    /// Hammer contact for one sample (sub-stepped).
    fn contact_step(&mut self) {
        let h = self.dt / SUBSTEPS as f32;
        let k = self.km.k_contact as f32;
        let lam = self.km.lambda as f32;
        let beta = self.km.beta as f32;
        let inv_mh = 1.0 / self.km.hammer_mass as f32;
        let hs = self.km.horiz_share as f32;
        for _ in 0..SUBSTEPS {
            let c = self.xh - self.strike_disp();
            let mut f = 0.0;
            if c > 0.0 {
                let cv = self.vh - self.strike_vel();
                f = (k * c.powf(beta) * (1.0 + lam * cv)).max(0.0);
            }
            self.vh -= f * inv_mh * h;
            self.xh += self.vh * h;
            if f > 0.0 {
                let j = f * h;
                for i in 0..M {
                    let share = if i == HORIZ { hs } else { 1.0 };
                    self.im[i] -= share * j * self.phi[i] * self.inv_mw[i];
                }
            }
            for i in 0..M {
                let (a, b) = (self.re[i], self.im[i]);
                self.re[i] = self.cr4[i] * a - self.ci4[i] * b;
                self.im[i] = self.ci4[i] * a + self.cr4[i] * b;
            }
            if c <= 0.0 && self.vh < self.strike_vel() {
                self.contact = false;
                break;
            }
        }
        self.contact_left = self.contact_left.saturating_sub(1);
        if self.contact_left == 0 {
            self.contact = false;
        }
    }

    /// Adds this tone generator's pickup voltage (before the harp's
    /// group division) into `out`.
    pub fn process(&mut self, out: &mut [f32]) {
        if !self.active {
            return;
        }
        let gain = self.km.harp_gain * EMF * self.fs;
        let off = self.km.offset_mm;
        let mut peak = 0.0f32;
        for o in out.iter_mut() {
            if self.contact {
                self.contact_step();
            } else {
                rotate(&mut self.re, &mut self.im, &self.cr, &self.ci, self.nm);
            }
            // Damper felt closing in (≈10 ms) and pulling energy out.
            if self.damp_target > self.damp {
                self.damp = (self.damp + 100.0 * self.dt).min(1.0);
            }
            if self.damp > 0.0 {
                let g = 1.0 - self.damp * (1.0 - self.damp_full);
                for i in 0..self.nm {
                    self.re[i] *= g;
                    self.im[i] *= g;
                }
            }
            let dy = 1000.0 * self.vertical();
            let y = off + dy;
            let x = 1000.0 * self.re[HORIZ];
            let rho = (y * y + x * x).sqrt();
            let p = self.slice.eval(rho);
            *o += gain * (p - self.prev_phi);
            self.prev_phi = p;
            peak = peak.max(dy.abs());
        }
        self.level_mm = peak;
        // Sleep check; inharmonic modes retire when inaudible.
        if !self.contact {
            if self.nm > INHARM {
                let mut hf = 0.0;
                for i in INHARM..M {
                    hf += self.re[i].abs() + self.im[i].abs();
                }
                if hf < 2e-8 {
                    for i in INHARM..M {
                        self.re[i] = 0.0;
                        self.im[i] = 0.0;
                    }
                    self.nm = INHARM;
                }
            }
            let mut amp = 0.0;
            for i in 0..self.nm {
                amp += self.re[i].abs() + self.im[i].abs();
            }
            if amp < 1e-6 {
                self.quiet += out.len() as u32;
                if self.quiet as f32 > 0.05 * self.fs {
                    self.silence();
                }
            } else {
                self.quiet = 0;
            }
        }
        // The pickup's reference stays exact when modes retire.
        if self.nm == INHARM && self.active {
            self.resync_pickup();
        }
    }

    pub fn tip_amplitude_mm(&self) -> f32 {
        let mut a = 0.0;
        for i in 0..M {
            if i != HORIZ {
                a += (self.re[i] * self.re[i] + self.im[i] * self.im[i]).sqrt();
            }
        }
        1000.0 * a
    }
}

/// Free ringing of the first `n` modes: z ← r·e^{iωT}·z.
#[inline]
fn rotate(re: &mut [f32; M], im: &mut [f32; M], cr: &[f32; M], ci: &[f32; M], n: usize) {
    if n == M {
        for i in 0..M {
            let (a, b) = (re[i], im[i]);
            re[i] = cr[i] * a - ci[i] * b;
            im[i] = ci[i] * a + cr[i] * b;
        }
    } else {
        for i in 0..INHARM {
            let (a, b) = (re[i], im[i]);
            re[i] = cr[i] * a - ci[i] * b;
            im[i] = ci[i] * a + cr[i] * b;
        }
    }
}

#[cfg(test)]
mod probe {
    use super::*;
    use crate::dsp::keys::{Regulation, Unit};

    /// `cargo test --release --lib strike_probe -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn strike_probe() {
        let fs = 96_000.0;
        let u = Unit::new(1975);
        let reg = Regulation { voicing: 0.45, gap_mm: 2.0, hammer: 1.0, condition: 0.0, a4: 440.0, stretch: false };
        for key in [21usize, 35, 52, 70] {
            for v0 in [0.5f32, 1.5, 3.0] {
                let mut v = Voice::new(u.model(key, &reg, fs), fs);
                v.strike(v0);
                let mut out = vec![0.0f32; 1];
                let mut contact = 0;
                while v.contact {
                    v.process(&mut out);
                    contact += 1;
                }
                let amps: Vec<String> = (0..M).map(|i| format!("{:.2}", 1000.0 * (v.re[i] * v.re[i] + v.im[i] * v.im[i]).sqrt())).collect();
                println!("key {key:2} f0 {:7.1} v0 {v0:.1}: contact {:5.2} ms, hammer out {:5.2} m/s, modes mm [{}]", v.km.f0, contact as f64 / 96.0, v.vh, amps.join(" "));
            }
        }
    }
}
