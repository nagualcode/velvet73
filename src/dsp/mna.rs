//! Small linear circuits solved by modified nodal analysis with
//! trapezoidal companion models (exactly the bilinear transform of the
//! circuit). The admittance matrix is inverted once whenever a part value
//! changes (a knob); each sample is then a matrix-vector product.

pub const MAXN: usize = 8;

#[derive(Clone, Copy)]
enum Kind {
    R,
    C,
    L,
}

#[derive(Clone, Copy)]
struct Elem {
    kind: Kind,
    a: usize, // node index + 1 (0 = ground)
    b: usize,
    value: f64,
    g: f64,
    hist: f64, // J (capacitor) or H (inductor)
}

/// A linear network driven by one voltage source through a series
/// resistor into node `inp` (Norton equivalent).
pub struct Net {
    n: usize,
    elems: [Elem; 16],
    ne: usize,
    inp: usize,
    g_in: f64,
    out: usize,
    inv: [[f64; MAXN]; MAXN],
    v: [f64; MAXN],
    t: f64,
    /// Indices of the reactive elements (the only ones with history).
    react: [usize; 16],
    nr: usize,
}

impl Net {
    /// `n` internal nodes (1-based in the element list), source resistor
    /// `r_src` into node `inp`, output at node `out`.
    pub fn new(n: usize, inp: usize, r_src: f64, out: usize, fs: f64) -> Self {
        let e = Elem { kind: Kind::R, a: 0, b: 0, value: 1.0, g: 0.0, hist: 0.0 };
        Self { n, elems: [e; 16], ne: 0, inp, g_in: 1.0 / r_src, out, inv: [[0.0; MAXN]; MAXN], v: [0.0; MAXN], t: 1.0 / fs, react: [0; 16], nr: 0 }
    }

    fn push(&mut self, kind: Kind, a: usize, b: usize, value: f64) -> usize {
        let i = self.ne;
        self.elems[i] = Elem { kind, a, b, value, g: 0.0, hist: 0.0 };
        self.ne += 1;
        if !matches!(kind, Kind::R) {
            self.react[self.nr] = i;
            self.nr += 1;
        }
        i
    }
    pub fn r(&mut self, a: usize, b: usize, ohms: f64) -> usize {
        self.push(Kind::R, a, b, ohms)
    }
    pub fn c(&mut self, a: usize, b: usize, farads: f64) -> usize {
        self.push(Kind::C, a, b, farads)
    }
    pub fn l(&mut self, a: usize, b: usize, henries: f64) -> usize {
        self.push(Kind::L, a, b, henries)
    }

    /// Change a part value (call `build` afterwards).
    pub fn set(&mut self, idx: usize, value: f64) {
        self.elems[idx].value = value;
    }
    pub fn set_source(&mut self, r_src: f64) {
        self.g_in = 1.0 / r_src;
    }

    /// Recompute conductances and the inverse admittance matrix. History
    /// currents are kept (a turned knob does not click).
    pub fn build(&mut self) {
        let n = self.n;
        let mut m = [[0.0f64; MAXN]; MAXN];
        m[self.inp - 1][self.inp - 1] += self.g_in;
        for e in self.elems[..self.ne].iter_mut() {
            e.g = match e.kind {
                Kind::R => 1.0 / e.value.max(1e-3),
                Kind::C => 2.0 * e.value / self.t,
                Kind::L => self.t / (2.0 * e.value),
            };
            let (a, b, g) = (e.a, e.b, e.g);
            if a > 0 {
                m[a - 1][a - 1] += g;
            }
            if b > 0 {
                m[b - 1][b - 1] += g;
            }
            if a > 0 && b > 0 {
                m[a - 1][b - 1] -= g;
                m[b - 1][a - 1] -= g;
            }
        }
        // Gauss–Jordan inverse with partial pivoting.
        let mut inv = [[0.0f64; MAXN]; MAXN];
        for i in 0..n {
            inv[i][i] = 1.0;
        }
        for col in 0..n {
            let mut p = col;
            for r in col + 1..n {
                if m[r][col].abs() > m[p][col].abs() {
                    p = r;
                }
            }
            m.swap(col, p);
            inv.swap(col, p);
            let d = m[col][col];
            let d = if d.abs() < 1e-300 { 1e-300 } else { d };
            for k in 0..n {
                m[col][k] /= d;
                inv[col][k] /= d;
            }
            for r in 0..n {
                if r != col {
                    let f = m[r][col];
                    if f != 0.0 {
                        for k in 0..n {
                            m[r][k] -= f * m[col][k];
                            inv[r][k] -= f * inv[col][k];
                        }
                    }
                }
            }
        }
        self.inv = inv;
    }

    pub fn clear(&mut self) {
        self.v = [0.0; MAXN];
        for e in self.elems[..self.ne].iter_mut() {
            e.hist = 0.0;
        }
    }

    /// One sample: source voltage in, output node voltage out.
    #[inline]
    pub fn run(&mut self, vin: f64) -> f64 {
        let n = self.n;
        let mut rhs = [0.0f64; MAXN];
        rhs[self.inp - 1] += vin * self.g_in;
        for &ri in &self.react[..self.nr] {
            let e = &self.elems[ri];
            let j = match e.kind {
                Kind::R => continue,
                Kind::C => e.hist,  // current source into a
                Kind::L => -e.hist, // H flows a → b
            };
            if e.a > 0 {
                rhs[e.a - 1] += j;
            }
            if e.b > 0 {
                rhs[e.b - 1] -= j;
            }
        }
        let mut v = [0.0f64; MAXN];
        for i in 0..n {
            let row = &self.inv[i];
            let mut s = 0.0;
            for k in 0..n {
                s += row[k] * rhs[k];
            }
            v[i] = s;
        }
        for &ri in &self.react[..self.nr] {
            let e = &mut self.elems[ri];
            let va = if e.a > 0 { v[e.a - 1] } else { 0.0 };
            let vb = if e.b > 0 { v[e.b - 1] } else { 0.0 };
            let vd = va - vb;
            match e.kind {
                Kind::R => {}
                Kind::C => {
                    let i = e.g * vd - e.hist;
                    e.hist = e.g * vd + i;
                }
                Kind::L => {
                    let i = e.g * vd + e.hist;
                    e.hist = i + e.g * vd;
                }
            }
        }
        self.v = v;
        v[self.out - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RC low-pass: −3 dB at 1/(2πRC).
    #[test]
    fn rc_lowpass_corner() {
        let fs = 96_000.0;
        let (r, c) = (10_000.0, 10e-9);
        let fc = 1.0 / (2.0 * std::f64::consts::PI * r * c);
        let mut net = Net::new(1, 1, r, 1, fs);
        net.c(1, 0, c);
        net.build();
        let mut acc = 0.0;
        let n = 96_000;
        for k in 0..n {
            let x = (2.0 * std::f64::consts::PI * fc * k as f64 / fs).sin();
            let y = net.run(x);
            if k > n / 2 {
                acc += y * y;
            }
        }
        let g = (acc / (n / 2) as f64 * 2.0).sqrt();
        assert!((20.0 * g.log10() + 3.01).abs() < 0.1, "{}", 20.0 * g.log10());
    }

    /// Series RL into a resistor: −3 dB at (R1+R2)/(2πL).
    #[test]
    fn rl_lowpass_corner() {
        let fs = 96_000.0;
        let (r1, l, r2) = (1_000.0, 0.5, 9_000.0);
        let fc = (r1 + r2) / (2.0 * std::f64::consts::PI * l);
        let mut net = Net::new(2, 1, r1, 2, fs);
        net.l(1, 2, l);
        net.r(2, 0, r2);
        net.build();
        let mut acc = 0.0;
        let n = 96_000;
        for k in 0..n {
            let x = (2.0 * std::f64::consts::PI * fc * k as f64 / fs).sin();
            let y = net.run(x);
            if k > n / 2 {
                acc += y * y;
            }
        }
        let g = (acc / (n / 2) as f64 * 2.0).sqrt() / 0.9;
        assert!((20.0 * g.log10() + 3.01).abs() < 0.15, "{}", 20.0 * g.log10());
    }
}
