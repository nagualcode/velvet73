//! Decimation from the oversampled circuit rate to the host rate:
//! cascaded half-band FIRs (Kaiser-windowed sinc, linear phase).

fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let q = x * x / 4.0;
    for k in 1..60 {
        term *= q / (k * k) as f64;
        sum += term;
        if term < 1e-14 * sum {
            break;
        }
    }
    sum
}

/// Half-band low-pass taps (odd length `n`), cutoff at a quarter of the
/// input rate.
fn halfband(n: usize, beta: f64) -> Vec<f32> {
    let m = (n - 1) as f64 / 2.0;
    let i0b = bessel_i0(beta);
    let mut h: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64 - m;
            let sinc = if t == 0.0 { 0.5 } else { (std::f64::consts::PI * t * 0.5).sin() / (std::f64::consts::PI * t) };
            let r = t / m;
            let w = bessel_i0(beta * (1.0 - r * r).max(0.0).sqrt()) / i0b;
            sinc * w
        })
        .collect();
    let s: f64 = h.iter().sum();
    for v in &mut h {
        *v /= s;
    }
    h.into_iter().map(|v| v as f32).collect()
}

/// Decimate by two.
pub struct Half {
    taps: Vec<(usize, f32)>,
    center: f32,
    mid: usize,
    buf: Vec<f32>,
    len: usize,
    pos: usize,
}

impl Half {
    pub fn new(n: usize, beta: f64) -> Self {
        let h = halfband(n, beta);
        let mid = (n - 1) / 2;
        let taps = h.iter().enumerate().filter(|(i, v)| *i != mid && v.abs() > 1e-9).map(|(i, v)| (i, *v)).collect();
        Self { taps, center: h[mid], mid, buf: vec![0.0; 2 * n], len: n, pos: 0 }
    }

    /// Group delay in output samples.
    pub fn delay_out(&self) -> f64 {
        self.mid as f64 / 2.0
    }

    #[inline]
    fn push(&mut self, x: f32) {
        self.pos = if self.pos == 0 { self.len - 1 } else { self.pos - 1 };
        self.buf[self.pos] = x;
        self.buf[self.pos + self.len] = x;
    }

    /// Two input samples in, one out.
    #[inline]
    pub fn process(&mut self, a: f32, b: f32) -> f32 {
        self.push(a);
        self.push(b);
        let w = &self.buf[self.pos..self.pos + self.len];
        let mut acc = self.center * w[self.mid];
        for &(i, h) in &self.taps {
            acc += h * w[i];
        }
        acc
    }

    pub fn reset(&mut self) {
        self.buf.fill(0.0);
    }
}

/// Oversampling factor for a host rate: the tine/pickup/amp chain runs at
/// 88.2–96 kHz (the pickup and the tubes make harmonics well above 20 kHz).
pub fn factor_for(host: f64) -> usize {
    if host < 60_000.0 { 2 } else { 1 }
}

/// Mono decimator by 1 or 2.
pub struct Decimator {
    factor: usize,
    half: Half,
}

impl Decimator {
    pub fn new(factor: usize) -> Self {
        Self { factor, half: Half::new(71, 8.5) }
    }

    /// Latency in host samples.
    pub fn latency(&self) -> f64 {
        if self.factor == 2 { self.half.delay_out() } else { 0.0 }
    }

    /// `x` holds `factor` oversampled samples; returns one host sample.
    #[inline]
    pub fn process(&mut self, x: &[f32]) -> f32 {
        if self.factor == 2 { self.half.process(x[0], x[1]) } else { x[0] }
    }

    pub fn reset(&mut self) {
        self.half.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gain_at(f_rel_out: f64) -> f64 {
        let mut d = Decimator::new(2);
        let mut acc: f64 = 0.0;
        for n in 0..8000 {
            let mut x = [0.0f32; 2];
            for (k, v) in x.iter_mut().enumerate() {
                let t = (n * 2 + k) as f64 / 2.0;
                *v = (std::f64::consts::TAU * f_rel_out * t).sin() as f32;
            }
            let y = d.process(&x);
            if n >= 4000 {
                acc += f64::from(y).powi(2);
            }
        }
        (acc / 4000.0 * 2.0).sqrt()
    }

    #[test]
    fn passband_flat_stopband_deep() {
        assert!((gain_at(0.1) - 1.0).abs() < 0.01);
        assert!((gain_at(20.0 / 48.0) - 1.0).abs() < 0.03, "{}", gain_at(20.0 / 48.0));
        assert!(gain_at(0.75) < 1e-3, "alias: {}", gain_at(0.75));
    }
}
