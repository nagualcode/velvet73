//! The harp as a source and the Stage piano's nameboard.
//!
//! All pickups in series/parallel make one inductive source: ≈1425 Ω DC
//! (73 keys) and ≈0.5 H (estimated from three independent routes, range
//! 0.33–0.85 H, docs/CIRCUIT.md §5). The nameboard: "Bass Boost" is a
//! 50 kΩ reverse-audio rheostat bridged by 47 nF in series with the
//! signal (a bass *cut* that vanishes at full rotation), then a 10 kΩ
//! audio Volume pot to ground [SCH, EP-Forum redraw of 1979–84 pianos;
//! Gabrielli: "low shelving 18 dB bass cut at 300 Hz"]. The 10 kΩ loading
//! the harp's inductance is a ~4 kHz low-pass: much of a Stage's warmth.
//! Then the cable's capacitance and the amplifier's 1 MΩ input.

use super::mna::Net;

#[derive(Clone, Copy, PartialEq)]
pub enum Board {
    /// 10k audio Volume, 50k reverse-audio Bass Boost, 47 nF.
    Original,
    /// The common technicians' mod: 50k Volume, 250k Bass, 10 nF.
    Modded,
    /// Harp straight to the cable (no nameboard).
    Direct,
}

/// Audio-taper pot: fraction of the track below the wiper.
#[inline]
pub fn audio(x: f64) -> f64 {
    ((10f64.powf(2.0 * x.clamp(0.0, 1.0)) - 1.0) / 99.0).clamp(0.0, 1.0)
}

pub struct Nameboard {
    net: Net,
    e_rl: usize,
    e_rr: usize,
    e_cbb: usize,
    e_rt: usize,
    e_rb: usize,
    e_cc: usize,
    pub r_harp: f64,
}

impl Nameboard {
    pub fn new(fs: f64) -> Self {
        // Nodes: 1 after the harp's resistance, 2 after its inductance,
        // 3 after the bass network, 4 the volume wiper / jack.
        let mut net = Net::new(4, 1, 1425.0, 4, fs);
        let e_rl = net.l(1, 2, 0.5);
        let e_rr = net.r(2, 3, 1.0);
        let e_cbb = net.c(2, 3, 47e-9);
        let e_rt = net.r(3, 4, 1.0);
        let e_rb = net.r(4, 0, 10e3);
        let e_cc = net.c(4, 0, 500e-12);
        net.r(4, 0, 1e6); // amplifier input
        net.r(3, 0, 1e9); // keeps node 3 defined in Direct mode
        let mut n = Self { net, e_rl, e_rr, e_cbb, e_rt, e_rb, e_cc, r_harp: 1425.0 };
        n.set(Board::Original, 1.0, 1.0, 0.5, 4.0, false);
        n
    }

    /// `volume`, `bass` 0..1 (rotation), harp inductance (H), cable (m).
    pub fn set(&mut self, board: Board, volume: f64, bass: f64, harp_h: f64, cable_m: f64, keys88: bool) {
        let (r_harp, l) = if keys88 { (1785.0, harp_h * 1.25) } else { (1425.0, harp_h) };
        self.r_harp = r_harp;
        self.net.set_source(r_harp);
        self.net.set(self.e_rl, l);
        self.net.set(self.e_cc, 100e-12 * cable_m.max(0.3) + 60e-12);
        match board {
            Board::Direct => {
                self.net.set(self.e_rr, 1e-3);
                self.net.set(self.e_rt, 1e-3);
                self.net.set(self.e_rb, 1e12);
            }
            Board::Original | Board::Modded => {
                let (rv, rbb, cbb) = if board == Board::Original { (10e3, 50e3, 47e-9) } else { (50e3, 250e3, 10e-9) };
                let rr = rbb * audio(1.0 - bass);
                let a = audio(volume);
                self.net.set(self.e_rr, rr.max(1.0));
                self.net.set(self.e_cbb, cbb);
                self.net.set(self.e_rt, (rv * (1.0 - a)).max(1.0));
                self.net.set(self.e_rb, (rv * a).max(1.0));
            }
        }
        self.net.build();
    }

    #[inline]
    pub fn run(&mut self, emf: f32) -> f32 {
        self.net.run(f64::from(emf)) as f32
    }

    pub fn clear(&mut self) {
        self.net.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(nb: &mut Nameboard, f: f64, fs: f64) -> f64 {
        nb.clear();
        let n = (fs * 0.5) as usize;
        let mut acc = 0.0;
        for k in 0..n {
            let y = nb.run((2.0 * std::f64::consts::PI * f * k as f64 / fs).sin() as f32);
            if k > n / 2 {
                acc += f64::from(y).powi(2);
            }
        }
        10.0 * (acc / (n / 2) as f64 * 2.0).log10()
    }

    /// Bass Boost at minimum: the low shelf of Gabrielli et al. (≈−13 dB
    /// at 50 Hz, little above 1 kHz); at maximum: flat.
    #[test]
    fn bass_boost_is_a_bass_cut() {
        let fs = 96_000.0;
        let mut nb = Nameboard::new(fs);
        nb.set(Board::Original, 1.0, 1.0, 0.5, 4.0, false);
        let (lo_full, mid_full) = (level(&mut nb, 50.0, fs), level(&mut nb, 1000.0, fs));
        nb.set(Board::Original, 1.0, 0.0, 0.5, 4.0, false);
        let (lo_cut, mid_cut) = (level(&mut nb, 50.0, fs), level(&mut nb, 1000.0, fs));
        assert!(lo_full - lo_cut > 10.0, "{lo_full} {lo_cut}");
        assert!((mid_full - mid_cut).abs() < 1.5);
    }

    /// The 10 kΩ volume pot loading ~0.5 H: treble falls off around 4 kHz;
    /// straight from the harp into 1 MΩ it rings near 10 kHz instead.
    #[test]
    fn the_volume_pot_darkens_the_harp() {
        let fs = 96_000.0;
        let mut nb = Nameboard::new(fs);
        nb.set(Board::Original, 1.0, 1.0, 0.5, 5.0, false);
        let d = level(&mut nb, 8000.0, fs) - level(&mut nb, 500.0, fs);
        assert!(d < -6.0, "{d}");
        nb.set(Board::Direct, 1.0, 1.0, 0.5, 5.0, false);
        let d2 = level(&mut nb, 8000.0, fs) - level(&mut nb, 500.0, fs);
        assert!(d2 > 3.0, "{d2}");
    }
}
