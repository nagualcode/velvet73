//! The whole instrument: 88 tone generators (73 playable on the Stage
//! 73), the harp, the nameboard, the Twin (or a DI), cabinet and mic.

use super::amp::{AmpKnobs, Twin};
use super::cab::Cab;
use super::keys::{Regulation, Unit};
use super::resample::{Decimator, factor_for};
use super::stage::{Board, Nameboard};
use super::voice::Voice;
use super::{HighPass, KEYS, MIDI_LOW, Smooth};

pub const MAX_CHUNK: usize = 256;

#[derive(Clone, Copy, PartialEq)]
pub enum AmpMode {
    /// Fender Twin Reverb, Vibrato channel, 2×12 and a mic.
    Twin,
    /// Twin line out (no cabinet).
    TwinLine,
    /// Straight into the console through a DI.
    Di,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Controls {
    // Piano.
    pub voicing: f32,
    pub gap_mm: f32,
    pub hammer: f32,
    pub touch: f32,
    pub condition: f32,
    pub a4: f32,
    pub stretch: bool,
    pub keys88: bool,
    // Stage.
    pub stage_volume: f32,
    pub bass_boost: f32,
    pub harp_h: f32,
    pub cable_m: f32,
    pub board: Board,
    // Amp.
    pub amp: AmpMode,
    pub knobs: AmpKnobs,
    pub mic: f32,
    pub noise: f32,
    pub output_db: f32,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            voicing: 0.45,
            gap_mm: 2.0,
            hammer: 1.0,
            touch: 0.5,
            condition: 0.25,
            a4: 440.0,
            stretch: true,
            keys88: false,
            stage_volume: 1.0,
            bass_boost: 0.8,
            harp_h: 0.5,
            cable_m: 5.0,
            board: Board::Original,
            amp: AmpMode::Twin,
            knobs: AmpKnobs { volume: 0.4, treble: 0.5, middle: 0.4, bass: 0.45, bright: false, reverb: 0.22, speed: 0.4, intensity: 0.0 },
            mic: 0.35,
            noise: 1.0,
            output_db: 0.0,
        }
    }
}

pub struct Engine {
    host: f64,
    factor: usize,
    fs: f64,
    unit: Unit,
    voices: Vec<Voice>,
    reg: Option<Regulation>,
    reg_s: [Smooth; 3],
    board: Option<(Board, f32, f32, f32, f32, bool)>,
    board_s: [Smooth; 2],
    nameboard: Nameboard,
    twin: Twin,
    cab: Cab,
    dec: Decimator,
    dc: HighPass,
    harp: Vec<f32>,
    over: Vec<f32>,
    sustain: bool,
    hum_phase: f64,
    asleep: bool,
    quiet: usize,
    pub levels: [f32; KEYS],
    /// Peak volts at the jack and across the speaker since last read (analysis).
    pub probe: [f32; 2],
}

impl Engine {
    pub fn new(host: f64, serial: u32) -> Self {
        let factor = factor_for(host);
        let fs = host * factor as f64;
        let unit = Unit::new(serial);
        let c = Controls::default();
        let reg = regulation(&c);
        let voices = (1..=KEYS).map(|k| Voice::new(unit.model(k, &reg, fs), fs)).collect();
        let k = (1.0 - (-1.0 / (0.02 * host / 64.0)).exp()) as f32;
        let mut e = Self {
            host,
            factor,
            fs,
            unit,
            voices,
            reg: Some(reg),
            reg_s: [Smooth::new(c.voicing, k), Smooth::new(c.gap_mm, k), Smooth::new(c.hammer, k)],
            board: None,
            board_s: [Smooth::new(c.stage_volume, k), Smooth::new(c.bass_boost, k)],
            nameboard: Nameboard::new(fs),
            twin: Twin::new(fs),
            cab: Cab::new(host),
            dec: Decimator::new(factor),
            dc: HighPass::new(10.0, host),
            harp: vec![0.0; MAX_CHUNK * 2],
            over: vec![0.0; MAX_CHUNK * 2],
            sustain: false,
            hum_phase: 0.0,
            asleep: false,
            quiet: 0,
            levels: [0.0; KEYS],
            probe: [0.0; 2],
        };
        e.twin.set(&c.knobs);
        e
    }

    pub fn host_rate(&self) -> f64 {
        self.host
    }
    pub fn latency(&self) -> f64 {
        self.dec.latency()
    }
    pub fn is_asleep(&self) -> bool {
        self.asleep
    }

    fn key_of(note: u8, c: &Controls) -> Option<usize> {
        let k = i32::from(note) - i32::from(MIDI_LOW) + 1;
        let (lo, hi) = if c.keys88 { (1, 88) } else { (8, 80) };
        (lo..=hi).contains(&k).then_some(k as usize)
    }

    pub fn note_on(&mut self, note: u8, vel: f32, c: &Controls) {
        let Some(k) = Self::key_of(note, c) else { return };
        if vel <= 0.0 {
            self.note_off(note, c);
            return;
        }
        // Key velocity → hammer speed (≈0.1–2 m/s). The bass hammers
        // travel a longer escapement and the long tines swing wide; the
        // per-register scale keeps the tip excursion within the ±6 mm a
        // real harp allows [DESIGN, docs/CIRCUIT.md §1].
        let p = 3.0 - 2.0 * c.touch;
        let v = &mut self.voices[k - 1];
        let reg = ((v.km.f0 / 196.0).powf(0.8) as f32).clamp(0.45, 1.3);
        let v0 = (0.1 + 1.5 * vel.clamp(0.0, 1.0).powf(p)) * reg;
        v.set_damper(false);
        v.strike(v0);
        let pos = v.km.harp_pos;
        let push = v.km.hammer_mass as f32 * v0;
        // Sustain pedal down: the blow travels along the harp rail and
        // nudges every free tine (the "halo" of a pedalled chord).
        if self.sustain {
            for (i, o) in self.voices.iter_mut().enumerate() {
                if i != k - 1 && !o.damped() {
                    let d = (o.km.harp_pos - pos).abs();
                    o.kick(0.0025 * push * (-d / 0.25).exp());
                }
            }
        }
        self.asleep = false;
        self.quiet = 0;
    }

    pub fn note_off(&mut self, note: u8, c: &Controls) {
        let Some(k) = Self::key_of(note, c) else { return };
        let v = &mut self.voices[k - 1];
        v.held = false;
        if !self.sustain {
            v.set_damper(true);
        }
    }

    pub fn sustain(&mut self, on: bool) {
        self.sustain = on;
        for v in &mut self.voices {
            if on {
                v.set_damper(false);
            } else if !v.held {
                v.set_damper(true);
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        self.sustain = false;
        for v in &mut self.voices {
            v.held = false;
            v.set_damper(true);
        }
    }

    /// Everything to silence at once (panic, preset change).
    pub fn panic(&mut self) {
        self.sustain = false;
        for v in &mut self.voices {
            v.held = false;
            v.silence();
            v.set_damper(true);
        }
        self.nameboard.clear();
        self.twin.clear();
        self.cab.clear();
        self.dec.reset();
        self.dc.clear();
        self.levels = [0.0; KEYS];
    }

    fn update(&mut self, c: &Controls) {
        let moving = self.reg_s[0].step(c.voicing) | self.reg_s[1].step(c.gap_mm) | self.reg_s[2].step(c.hammer);
        let mut reg = regulation(c);
        reg.voicing = self.reg_s[0].v;
        reg.gap_mm = self.reg_s[1].v;
        reg.hammer = self.reg_s[2].v;
        if self.reg != Some(reg) || moving {
            if self.reg != Some(reg) {
                for k in 1..=KEYS {
                    let km = self.unit.model(k, &reg, self.fs);
                    let v = &mut self.voices[k - 1];
                    v.retune(km, self.fs);
                    v.resync_pickup();
                }
            }
            self.reg = Some(reg);
        }
        self.board_s[0].step(c.stage_volume);
        self.board_s[1].step(c.bass_boost);
        let b = (c.board, self.board_s[0].v, self.board_s[1].v, c.harp_h, c.cable_m, c.keys88);
        if self.board != Some(b) {
            self.nameboard.set(b.0, f64::from(b.1), f64::from(b.2), f64::from(b.3), f64::from(b.4), b.5);
            self.board = Some(b);
        }
        self.twin.set(&c.knobs);
        self.cab.set_mic(c.mic);
    }

    /// Host-rate mono output.
    pub fn process(&mut self, c: &Controls, out: &mut [f32]) {
        let mut done = 0;
        while done < out.len() {
            let len = (out.len() - done).min(MAX_CHUNK);
            self.chunk(c, &mut out[done..done + len]);
            done += len;
        }
    }

    fn chunk(&mut self, c: &Controls, out: &mut [f32]) {
        let n = out.len();
        let ni = n * self.factor;
        let any = self.voices.iter().any(|v| v.active);
        if self.asleep && !any && c.noise <= 0.0 {
            out.fill(0.0);
            return;
        }
        self.update(c);
        let harp = &mut self.harp[..ni];
        harp.fill(0.0);
        for (k, v) in self.voices.iter_mut().enumerate() {
            v.process(harp);
            self.levels[k] = if v.active { v.level_mm } else { 0.0 };
        }
        // Mains hum induced in the harp: the alternating group polarity
        // cancels most of it; what remains is a few tens of µV.
        let hum = 4e-5 * c.noise;
        let dph = 60.0 / self.fs;
        let gain = 10f32.powf(c.output_db / 20.0);
        for i in 0..ni {
            let mut e = self.harp[i];
            if hum > 0.0 {
                self.hum_phase += dph;
                if self.hum_phase >= 1.0 {
                    self.hum_phase -= 1.0;
                }
                let p = std::f64::consts::TAU * self.hum_phase;
                e += hum * (p.sin() + 0.4 * (3.0 * p).sin() + 0.15 * (5.0 * p).sin()) as f32;
            }
            let jack = self.nameboard.run(e);
            self.over[i] = match c.amp {
                AmpMode::Twin | AmpMode::TwinLine => self.twin.run(jack, c.noise),
                AmpMode::Di => jack,
            };
            self.probe[0] = self.probe[0].max(jack.abs());
            self.probe[1] = self.probe[1].max(self.over[i].abs());
        }
        let mut loud = 0.0f32;
        for (j, o) in out.iter_mut().enumerate() {
            let x = self.dec.process(&self.over[j * self.factor..(j + 1) * self.factor]);
            let y = match c.amp {
                // Speaker volts → full scale at ~16 V across the voice coil.
                AmpMode::Twin => self.cab.run(x) * (1.0 / 20.0),
                AmpMode::TwinLine => x * (1.0 / 25.0),
                // Console line: the DI's ~+12 dB.
                AmpMode::Di => x * 2.0,
            };
            let y = self.dc.run(y) * gain;
            // Safety ceiling (the console, not the instrument): transparent
            // below −1 dBFS.
            let y = if y.abs() > 0.89 { y.signum() * (0.89 + 0.11 * ((y.abs() - 0.89) / 0.11).tanh()) } else { y };
            loud = loud.max(y.abs());
            *o = y;
        }
        // Sleep: no tine moving, no noise, reverb drained, output silent.
        if !any && c.noise <= 0.0 && loud < 3e-6 {
            self.quiet += n;
            if self.quiet as f64 > 0.5 * self.host {
                self.asleep = true;
                out.fill(0.0);
            }
        } else {
            self.quiet = 0;
            self.asleep = false;
        }
    }

    /// Tip excursion of a key (mm), for analysis.
    pub fn tip_mm(&self, note: u8) -> f32 {
        let k = i32::from(note) - i32::from(MIDI_LOW);
        if (0..KEYS as i32).contains(&k) { self.voices[k as usize].tip_amplitude_mm() } else { 0.0 }
    }
}

fn regulation(c: &Controls) -> Regulation {
    Regulation { voicing: c.voicing, gap_mm: c.gap_mm, hammer: c.hammer, condition: c.condition, a4: c.a4, stretch: c.stretch }
}
