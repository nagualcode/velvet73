//! The cone-tipped pickup.
//!
//! The flux the coil sees is set by where the steel tine tip sits in the
//! pole piece's field. The pole is a body of revolution, so only the tip's
//! radial distance ρ from the pole axis matters (vertical *and* horizontal
//! motion of the tine), at the tine's fixed axial gap. The field table
//! comes from a surface-charge model of the pole drawn in US 4,040,321,
//! validated against the patent's magnetometer curves and against Shear's
//! and Gabrielli's spectra (docs/PICKUP_ANALYSIS.md). The coil voltage is
//! −N dΦ/dt: computed as the difference of Φ between samples.

use super::pickup_table::{FIELD, GAP_N, GAP_STEP_MM, GAP0_MM, RHO_N, RHO_STEP_MM};

pub const SLICE: usize = RHO_N;

/// Φ(ρ) for one gap, interpolated between the table rows.
#[derive(Clone)]
pub struct Slice {
    pub phi: [f32; SLICE],
}

impl Slice {
    pub fn at_gap(gap_mm: f32) -> Self {
        let g = ((gap_mm - GAP0_MM) / GAP_STEP_MM).clamp(0.0, (GAP_N - 1) as f32 - 1e-4);
        let i = g as usize;
        let t = g - i as f32;
        let j = (i + 1).min(GAP_N - 1);
        let mut phi = [0.0; SLICE];
        for k in 0..SLICE {
            phi[k] = FIELD[i][k] * (1.0 - t) + FIELD[j][k] * t;
        }
        Self { phi }
    }

    /// Φ at radial distance ρ (mm), cubic Hermite (Catmull–Rom), so its
    /// time derivative has no steps.
    #[inline]
    pub fn eval(&self, rho_mm: f32) -> f32 {
        let x = (rho_mm / RHO_STEP_MM).clamp(0.0, (SLICE - 3) as f32);
        let i = x as usize;
        let t = x - i as f32;
        // ρ is a distance: mirror across the axis for the left neighbour.
        let p0 = if i == 0 { self.phi[1] } else { self.phi[i - 1] };
        let p1 = self.phi[i];
        let p2 = self.phi[i + 1];
        let p3 = self.phi[i + 2];
        let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
        let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
        let c = -0.5 * p0 + 0.5 * p2;
        ((a * t + b) * t + c) * t + p1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harmonics(s: &Slice, y0: f32, amp: f32) -> Vec<f32> {
        let n = 4096;
        let mut v = vec![0.0f32; n];
        let mut prev = s.eval(y0.abs());
        for k in 0..n {
            let y = y0 + amp * (std::f32::consts::TAU * k as f32 / n as f32).sin();
            let p = s.eval(y.abs());
            v[k] = p - prev;
            prev = p;
        }
        (1..=4)
            .map(|h| {
                let (mut re, mut im) = (0.0f64, 0.0f64);
                for (k, x) in v.iter().enumerate() {
                    let ph = std::f64::consts::TAU * h as f64 * k as f64 / n as f64;
                    re += f64::from(*x) * ph.cos();
                    im += f64::from(*x) * ph.sin();
                }
                (re * re + im * im).sqrt() as f32
            })
            .collect()
    }

    /// Manual fig. 4-8: on the axis only even harmonics (the octave),
    /// far off the axis mostly the fundamental.
    #[test]
    fn voicing_follows_the_service_manual() {
        let s = Slice::at_gap(3.2);
        let on = harmonics(&s, 0.0, 3.0);
        assert!(on[1] > 30.0 * on[0], "on axis: {on:?}");
        let off = harmonics(&s, 5.0, 3.0);
        assert!(off[0] > 2.0 * off[1], "5 mm: {off:?}");
    }

    #[test]
    fn closer_is_louder() {
        let far = harmonics(&Slice::at_gap(3.2), 1.5, 1.0)[0];
        let near = harmonics(&Slice::at_gap(0.6), 1.5, 1.0)[0];
        assert!(near > 4.0 * far);
    }
}
