//! The controls (automatable parameters) and the lock-free channel
//! between the audio thread and the editor.

use crate::dsp::KEYS;
use crate::dsp::amp::AmpKnobs;
use crate::dsp::engine::{AmpMode, Controls};
use crate::dsp::stage::Board;
use std::sync::atomic::{AtomicU32, Ordering};
use truce::prelude::*;

#[derive(ParamEnum)]
pub enum KeysP {
    #[name = "73"]
    K73,
    #[name = "88"]
    K88,
}

#[derive(ParamEnum)]
pub enum BoardP {
    Original,
    Modded,
    Direct,
}

#[derive(ParamEnum)]
pub enum AmpP {
    #[name = "Twin"]
    Twin,
    #[name = "Twin Line"]
    TwinLine,
    #[name = "DI"]
    Di,
}

#[derive(Params)]
pub struct VelvetParams {
    // ---- the piano ---------------------------------------------------
    /// Tine tip height against the pickup axis: on the axis (bell) …
    /// factory, slightly above (bark) … well off it (deep, round).
    #[param(name = "Voicing", group = "Piano", range = "linear(0, 1)", default = 0.45, format = "fmt_voicing")]
    pub voicing: FloatParam,
    /// Tine-to-pickup distance: closer is louder and barks sooner.
    #[param(name = "Pickup", group = "Piano", range = "linear(0.5, 3.2)", default = 2.0, format = "fmt_mm")]
    pub gap: FloatParam,
    /// Neoprene tip hardness against the factory grading.
    #[param(name = "Hammer", group = "Piano", range = "linear(0.4, 2.5)", default = 1.0, format = "fmt_hammer")]
    pub hammer: FloatParam,
    /// How much of the key velocity reaches the hammer.
    #[param(name = "Touch", group = "Piano", range = "linear(0, 1)", default = 0.5, unit = "%")]
    pub touch: FloatParam,
    /// Freshly regulated … forty years on the road.
    #[param(name = "Condition", group = "Piano", range = "linear(0, 1)", default = 0.25, format = "fmt_condition")]
    pub condition: FloatParam,
    #[param(name = "Tuning", group = "Piano", range = "linear(430, 450)", default = 440.0, unit = "Hz")]
    pub a4: FloatParam,
    /// The Service Manual's stretch tuning chart.
    #[param(name = "Stretch", group = "Piano", default = 1)]
    pub stretch: BoolParam,
    #[param(name = "Keys", group = "Piano", default = 0)]
    pub keys: EnumParam<KeysP>,

    // ---- the Stage nameboard -----------------------------------------
    #[param(name = "Stage Volume", short_name = "Volume", group = "Stage", range = "linear(0, 1)", default = 1.0, unit = "%")]
    pub stage_volume: FloatParam,
    /// A bass cut that vanishes at full rotation.
    #[param(name = "Bass Boost", group = "Stage", range = "linear(0, 1)", default = 0.8, unit = "%")]
    pub bass_boost: FloatParam,
    /// Inductance of the whole pickup harp.
    #[param(name = "Harp", group = "Stage", range = "linear(0.33, 0.85)", default = 0.5, format = "fmt_henry")]
    pub harp: FloatParam,
    #[param(name = "Cable", group = "Stage", range = "linear(1, 15)", default = 5.0, format = "fmt_m")]
    pub cable: FloatParam,
    #[param(name = "Nameboard", group = "Stage", default = 0)]
    pub board: EnumParam<BoardP>,

    // ---- the Twin ----------------------------------------------------
    #[param(name = "Amp", group = "Amp", default = 0)]
    pub amp: EnumParam<AmpP>,
    #[param(name = "Amp Volume", short_name = "AmpVol", group = "Amp", range = "linear(1, 10)", default = 4.0, format = "fmt_dial")]
    pub amp_volume: FloatParam,
    #[param(name = "Treble", group = "Amp", range = "linear(1, 10)", default = 5.5, format = "fmt_dial")]
    pub treble: FloatParam,
    #[param(name = "Middle", group = "Amp", range = "linear(1, 10)", default = 4.5, format = "fmt_dial")]
    pub middle: FloatParam,
    #[param(name = "Bass", group = "Amp", range = "linear(1, 10)", default = 5.0, format = "fmt_dial")]
    pub bass: FloatParam,
    #[param(name = "Bright", group = "Amp", default = 0)]
    pub bright: BoolParam,
    #[param(name = "Reverb", group = "Amp", range = "linear(1, 10)", default = 3.0, format = "fmt_dial")]
    pub reverb: FloatParam,
    #[param(name = "Speed", group = "Amp", range = "linear(1, 10)", default = 4.5, format = "fmt_dial")]
    pub speed: FloatParam,
    #[param(name = "Intensity", group = "Amp", range = "linear(1, 10)", default = 1.0, format = "fmt_dial")]
    pub intensity: FloatParam,
    /// Microphone from the dust cap (bright) to the cone edge (dark).
    #[param(name = "Mic", group = "Amp", range = "linear(0, 1)", default = 0.35, format = "fmt_mic")]
    pub mic: FloatParam,
    /// Hum and hiss, as built (100 %).
    #[param(name = "Noise", group = "Amp", range = "linear(0, 1.5)", default = 1.0, unit = "%")]
    pub noise: FloatParam,
    #[param(name = "Output", group = "Amp", range = "linear(-24, 12)", default = 0.0, unit = "dB")]
    pub output: FloatParam,

    /// Momentary: rising edge = panic (also MIDI CC 120).
    #[param(name = "Panic", default = 0)]
    pub panic: BoolParam,

    #[meter]
    pub meter_l: MeterSlot,
    #[meter]
    pub meter_r: MeterSlot,

    #[skip]
    pub ui: UiShared,
}

/// Fender dial (1..10) → pot rotation (0..1).
#[inline]
fn dial(v: f32) -> f32 {
    ((v - 1.0) / 9.0).clamp(0.0, 1.0)
}

impl VelvetParams {
    pub fn fmt_voicing(&self, v: f64) -> String {
        if v < 0.15 {
            "Bell".into()
        } else if v < 0.32 {
            "Bell-bark".into()
        } else if v < 0.58 {
            "Factory".into()
        } else if v < 0.8 {
            "Round".into()
        } else {
            "Deep".into()
        }
    }
    pub fn fmt_hammer(&self, v: f64) -> String {
        if (v - 1.0).abs() < 0.06 {
            "Factory".into()
        } else if v < 1.0 {
            format!("Soft {:.0}%", (1.0 - v) * 100.0)
        } else {
            format!("Hard ×{v:.1}")
        }
    }
    pub fn fmt_condition(&self, v: f64) -> String {
        match v {
            v if v < 0.12 => "Regulated".into(),
            v if v < 0.4 => "Studio".into(),
            v if v < 0.7 => "Gigged".into(),
            _ => "Road worn".into(),
        }
    }
    pub fn fmt_mm(&self, v: f64) -> String {
        format!("{v:.2} mm")
    }
    pub fn fmt_henry(&self, v: f64) -> String {
        format!("{v:.2} H")
    }
    pub fn fmt_m(&self, v: f64) -> String {
        format!("{v:.0} m")
    }
    pub fn fmt_dial(&self, v: f64) -> String {
        format!("{v:.1}")
    }
    pub fn fmt_mic(&self, v: f64) -> String {
        if v < 0.33 {
            "Cap".into()
        } else if v < 0.66 {
            "Cone".into()
        } else {
            "Edge".into()
        }
    }

    pub fn controls(&self) -> Controls {
        Controls {
            voicing: self.voicing.value(),
            gap_mm: self.gap.value(),
            hammer: self.hammer.value(),
            touch: self.touch.value(),
            condition: self.condition.value(),
            a4: self.a4.value(),
            stretch: self.stretch.value(),
            keys88: self.keys.value().to_index() == 1,
            stage_volume: self.stage_volume.value(),
            bass_boost: self.bass_boost.value(),
            harp_h: self.harp.value(),
            cable_m: self.cable.value(),
            board: [Board::Original, Board::Modded, Board::Direct][self.board.value().to_index().min(2)],
            amp: [AmpMode::Twin, AmpMode::TwinLine, AmpMode::Di][self.amp.value().to_index().min(2)],
            knobs: AmpKnobs {
                volume: dial(self.amp_volume.value()),
                treble: dial(self.treble.value()),
                middle: dial(self.middle.value()),
                bass: dial(self.bass.value()),
                bright: self.bright.value(),
                reverb: dial(self.reverb.value()),
                speed: dial(self.speed.value()),
                intensity: dial(self.intensity.value()),
            },
            mic: self.mic.value(),
            noise: self.noise.value(),
            output_db: self.output.value(),
        }
    }
}

/// Set a control by its preset key (the parameter's field name), in
/// plain units. Used by the offline renderer and the preset tests.
pub fn set_named(c: &mut Controls, key: &str, v: f64) -> bool {
    let f = v as f32;
    let idx = v.round().max(0.0) as usize;
    match key {
        "voicing" => c.voicing = f,
        "gap" => c.gap_mm = f,
        "hammer" => c.hammer = f,
        "touch" => c.touch = f,
        "condition" => c.condition = f,
        "a4" => c.a4 = f,
        "stretch" => c.stretch = v >= 0.5,
        "keys" => c.keys88 = idx == 1,
        "stage_volume" => c.stage_volume = f,
        "bass_boost" => c.bass_boost = f,
        "harp" => c.harp_h = f,
        "cable" => c.cable_m = f,
        "board" => c.board = [Board::Original, Board::Modded, Board::Direct][idx.min(2)],
        "amp" => c.amp = [AmpMode::Twin, AmpMode::TwinLine, AmpMode::Di][idx.min(2)],
        "amp_volume" => c.knobs.volume = dial(f),
        "treble" => c.knobs.treble = dial(f),
        "middle" => c.knobs.middle = dial(f),
        "bass" => c.knobs.bass = dial(f),
        "bright" => c.knobs.bright = v >= 0.5,
        "reverb" => c.knobs.reverb = dial(f),
        "speed" => c.knobs.speed = dial(f),
        "intensity" => c.knobs.intensity = dial(f),
        "mic" => c.mic = f,
        "noise" => c.noise = f,
        "output" => c.output_db = f,
        "panic" => {}
        _ => return false,
    }
    true
}

/// Controls of a `.preset` file's `[params]` table on top of the defaults.
pub fn controls_from_preset(text: &str) -> Result<Controls, String> {
    let mut c = VelvetParams::new().controls();
    let mut in_params = false;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            in_params = line == "[params]";
            continue;
        }
        if !in_params || line.is_empty() {
            continue;
        }
        let (k, v) = line.split_once('=').ok_or_else(|| format!("bad line: {line}"))?;
        let v: f64 = v.trim().parse().map_err(|_| format!("bad value: {line}"))?;
        if !set_named(&mut c, k.trim(), v) {
            return Err(format!("unknown parameter: {}", k.trim()));
        }
    }
    Ok(c)
}

/// Editor ⇄ audio thread, lock-free.
pub struct UiShared {
    /// Tine tip excursion per key, mm.
    pub level: [AtomicU32; KEYS],
    pub sustain: AtomicU32,
    pub panic: AtomicU32,
    /// Bumped by the editor when it loads a preset: voices reset.
    pub preset_epoch: AtomicU32,
}

impl Default for UiShared {
    fn default() -> Self {
        Self {
            level: std::array::from_fn(|_| AtomicU32::new(0)),
            sustain: AtomicU32::new(0),
            panic: AtomicU32::new(0),
            preset_epoch: AtomicU32::new(0),
        }
    }
}

#[inline]
pub fn store_f32(a: &AtomicU32, v: f32) {
    a.store(v.to_bits(), Ordering::Relaxed);
}

#[inline]
pub fn load_f32(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::Relaxed))
}
