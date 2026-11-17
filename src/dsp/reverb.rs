//! The Twin's spring tank (Accutronics type 4, two springs, long decay):
//! driver 12AT7 through a 500 pF coupling (thin, bright send) and a
//! transformer that saturates when hit hard, two dispersive spring lines
//! (cascades of stretched all-passes in a feedback delay: the "drip" is
//! high frequencies arriving before low ones, again at every echo), and
//! the recovery stage. After Välimäki, Parker & Abel (2010), parameters
//! chosen by ear-free physics: transit time, decay, and the ~4 kHz
//! dispersion corner [DESIGN].

use super::{HighPass, LowPass};

const N_AP: usize = 40;

/// One spring: `N_AP` stretched all-passes (a + z^-K)/(1 + a z^-K) sharing
/// one stride K (so one ring position serves them all), inside a lossy
/// feedback delay.
struct Spring {
    a: f32,
    k: usize,
    ap: Vec<f32>, // N_AP rows of K samples
    pos: usize,
    delay: Vec<f32>,
    dpos: usize,
    fb: f32,
    damp: LowPass,
    last: f32,
}

impl Spring {
    fn new(fs: f64, transit_s: f64, t60: f64, a: f32) -> Self {
        let k = (fs / (2.0 * 4300.0)).round().max(1.0) as usize;
        let len = ((transit_s * fs) as usize).max(1);
        let ap_delay = N_AP as f64 * k as f64 * (1.0 - f64::from(a)) / (1.0 + f64::from(a));
        let loop_s = (len as f64 + ap_delay) / fs;
        Self {
            a,
            k,
            ap: vec![0.0; N_AP * k],
            pos: 0,
            delay: vec![0.0; len],
            dpos: 0,
            fb: 10f64.powf(-3.0 * loop_s / t60) as f32,
            damp: LowPass::new(4200.0, fs),
            last: 0.0,
        }
    }
    #[inline]
    fn run(&mut self, x: f32) -> f32 {
        let mut s = x + self.fb * self.damp.run(self.last);
        let a = self.a;
        let k = self.k;
        let pos = self.pos;
        for i in 0..N_AP {
            let cell = &mut self.ap[i * k + pos];
            let wk = *cell;
            let w = s - a * wk;
            *cell = w;
            s = a * w + wk;
        }
        self.pos = if pos + 1 == k { 0 } else { pos + 1 };
        let out = self.delay[self.dpos];
        self.delay[self.dpos] = s;
        self.dpos = if self.dpos + 1 == self.delay.len() { 0 } else { self.dpos + 1 };
        self.last = out;
        out
    }
    fn clear(&mut self) {
        self.ap.fill(0.0);
        self.delay.fill(0.0);
        self.damp.clear();
        self.last = 0.0;
    }
}

pub struct Tank {
    send_hp: HighPass,
    xfmr_hp: HighPass,
    springs: [Spring; 2],
    ret_hp: HighPass,
    ret_lp: LowPass,
    pub energy: f32,
}

impl Tank {
    pub fn new(fs: f64) -> Self {
        Self {
            // 500 pF into the driver's 1 MΩ grid leak.
            send_hp: HighPass::new(318.0, fs),
            xfmr_hp: HighPass::new(90.0, fs),
            springs: [Spring::new(fs, 0.0335, 2.8, 0.62), Spring::new(fs, 0.0411, 2.5, 0.58)],
            ret_hp: HighPass::new(120.0, fs),
            ret_lp: LowPass::new(5200.0, fs),
            energy: 0.0,
        }
    }

    /// `x`: the vibrato channel's second-stage plate signal (volts).
    /// Returns the recovered reverb, scaled to the same order of level.
    #[inline]
    pub fn run(&mut self, x: f32) -> f32 {
        // Driver: 12AT7 paralleled into the tank transformer; the iron and
        // the tube flatten loud sends (the famous splash).
        let s = self.send_hp.run(x) * (1.0 / 30.0);
        let s = s / (1.0 + s.abs()).sqrt();
        let s = self.xfmr_hp.run(s);
        let y = 0.55 * self.springs[0].run(s) + 0.45 * self.springs[1].run(-s);
        let y = self.ret_lp.run(self.ret_hp.run(y));
        self.energy = 0.999 * self.energy + 0.001 * y.abs();
        y * 30.0
    }

    pub fn clear(&mut self) {
        self.send_hp.clear();
        self.xfmr_hp.clear();
        for s in &mut self.springs {
            s.clear();
        }
        self.ret_hp.clear();
        self.ret_lp.clear();
        self.energy = 0.0;
    }
}
