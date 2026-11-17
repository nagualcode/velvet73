//! The Twin's 2×12 open-back cabinet and a dynamic microphone close to
//! the grille. Parametric, from published responses of 12" guitar
//! speakers of the period (open-back bass loss, cone resonance near
//! 100 Hz, presence hump at 2–4 kHz, steep fall above ~5 kHz) [DESIGN].
//! MIC moves from the dust cap (brighter) to the cone edge (darker).

use super::{Biquad, Bq};

pub struct Cab {
    bq: [Biquad; 8],
    fs: f64,
    mic: f32,
}

impl Cab {
    pub fn new(fs: f64) -> Self {
        let mut c = Self { bq: [Biquad::default(); 8], fs, mic: -1.0 };
        c.set_mic(0.35);
        c
    }

    pub fn set_mic(&mut self, mic: f32) {
        if (mic - self.mic).abs() < 1e-4 {
            return;
        }
        self.mic = mic;
        let fs = self.fs;
        let m = f64::from(mic);
        let lp = 6800.0 * (1.0 - 0.5 * m);
        self.bq[0].set(Bq::Hp, 72.0, 0.85, fs);
        self.bq[1].set(Bq::Peak(3.0), 105.0, 1.3, fs);
        self.bq[2].set(Bq::Peak(-2.5), 420.0, 1.0, fs);
        self.bq[3].set(Bq::Peak(3.5 - 2.5 * m), 2400.0, 1.3, fs);
        self.bq[4].set(Bq::Peak(2.5 - 3.0 * m), 4200.0, 2.2, fs);
        self.bq[5].set(Bq::Lp, lp, 0.8, fs);
        self.bq[6].set(Bq::Lp, lp * 1.25, 0.6, fs);
        // Close-mic proximity: a little extra low end.
        self.bq[7].set(Bq::Peak(1.5 + 1.5 * m), 180.0, 0.8, fs);
    }

    #[inline]
    pub fn run(&mut self, x: f32) -> f32 {
        let mut y = x;
        for b in self.bq.iter_mut() {
            y = b.run(y);
        }
        y
    }

    pub fn clear(&mut self) {
        for b in self.bq.iter_mut() {
            b.clear();
        }
    }
}
