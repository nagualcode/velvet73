//! Triode gain stages of the Twin (7025 = low-noise 12AX7, 12AT7).
//!
//! Koren's triode equations; each common-cathode stage (plate resistor,
//! bypassed cathode resistor, grid fed through a source resistance with
//! grid current) is solved on its load line once at start-up and kept as
//! a dense transfer table: grid signal in → plate signal out (inverting,
//! with the tube's own asymmetry: soft cutoff one way, grid-current
//! flattening the other).

#[derive(Clone, Copy)]
pub struct Koren {
    mu: f64,
    ex: f64,
    kg1: f64,
    kp: f64,
    kvb: f64,
}

pub const T12AX7: Koren = Koren { mu: 100.0, ex: 1.4, kg1: 1060.0, kp: 600.0, kvb: 300.0 };
pub const T12AT7: Koren = Koren { mu: 60.0, ex: 1.35, kg1: 460.0, kp: 300.0, kvb: 300.0 };

impl Koren {
    /// Plate current, A.
    pub fn ip(&self, vgk: f64, vpk: f64) -> f64 {
        if vpk <= 0.0 {
            return 0.0;
        }
        let a = self.kp * (1.0 / self.mu + vgk / (self.kvb + vpk * vpk).sqrt());
        let sp = if a > 30.0 { a } else { a.exp().ln_1p() };
        let e1 = vpk / self.kp * sp;
        if e1 <= 0.0 { 0.0 } else { 2.0 * e1.powf(self.ex) / self.kg1 }
    }
}

/// Grid current when the grid goes positive, A.
fn ig(vgk: f64) -> f64 {
    if vgk > 0.0 { vgk.powf(1.5) / 1500.0 } else { 0.0 }
}

pub struct Stage {
    table: Vec<f32>,
    lo: f32,
    inv_step: f32,
}

impl Stage {
    /// `b_plus`, plate resistor `ra`, cathode resistor `rk` (bypassed),
    /// grid source resistance `rg`, table covering ±`range` volts.
    pub fn new(tube: Koren, b_plus: f64, ra: f64, rk: f64, rg: f64, range: f64) -> Self {
        // Quiescent point.
        let plate = |vg: f64, vk: f64| {
            let (mut lo, mut hi) = (vk, b_plus);
            for _ in 0..80 {
                let vp = 0.5 * (lo + hi);
                if vp - b_plus + ra * tube.ip(vg - vk, vp - vk) > 0.0 {
                    hi = vp;
                } else {
                    lo = vp;
                }
            }
            0.5 * (lo + hi)
        };
        let mut vk = 1.5;
        for _ in 0..60 {
            let vp = plate(0.0, vk);
            vk = 0.5 * vk + 0.5 * rk * tube.ip(-vk, vp - vk);
        }
        let vp0 = plate(0.0, vk);
        let n = 4097;
        let mut table = Vec::with_capacity(n);
        for i in 0..n {
            let vin = -range + 2.0 * range * i as f64 / (n - 1) as f64;
            // Grid: vin − vg = rg·ig(vg − vk).
            let vg = if vin - vk <= 0.0 {
                vin
            } else {
                let (mut lo, mut hi) = (vk.min(vin), vin);
                for _ in 0..60 {
                    let m = 0.5 * (lo + hi);
                    if vin - m - rg * ig(m - vk) > 0.0 {
                        lo = m;
                    } else {
                        hi = m;
                    }
                }
                0.5 * (lo + hi)
            };
            table.push((plate(vg, vk) - vp0) as f32);
        }
        Self { table, lo: -range as f32, inv_step: ((n - 1) as f64 / (2.0 * range)) as f32 }
    }

    #[inline]
    pub fn run(&self, vin: f32) -> f32 {
        let x = ((vin - self.lo) * self.inv_step).clamp(0.0, (self.table.len() - 2) as f32);
        let i = x as usize;
        let t = x - i as f32;
        self.table[i] + t * (self.table[i + 1] - self.table[i])
    }

    /// Small-signal gain (for calibration and tests).
    pub fn gain(&self) -> f32 {
        (self.run(0.01) - self.run(-0.01)) / 0.02
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 7025 with 100k/1.5k bypassed: gain ≈ 55–65, plate at ~200 V.
    #[test]
    fn preamp_stage_has_fender_gain() {
        let s = Stage::new(T12AX7, 320.0, 100e3, 1500.0, 34e3, 10.0);
        let g = s.gain();
        assert!((-70.0..-45.0).contains(&g), "gain {g}");
        // Asymmetric: positive grid swing flattens sooner than negative.
        assert!(s.run(4.0).abs() < s.run(-4.0).abs() * 1.5);
    }
}
