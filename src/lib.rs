//! Velvet73 — a Rhodes Mark I Stage 73 and the Fender Twin Reverb it was
//! plugged into, modelled from the tine up. The DSP lives in [`dsp`]
//! (one file per physical stage); this file maps parameters and MIDI.

#![allow(clippy::too_many_arguments, clippy::needless_range_loop)]

pub mod denormals;
pub mod dsp;
pub mod params;
pub mod presets;
pub mod ui;

use dsp::KEYS;
use dsp::engine::{Controls, Engine};
use dsp::leslie::Rotor;
use params::VelvetParams;
use params::{UiShared, store_f32};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use truce::prelude::*;
use truce_core::bus::{BusLayout, ChannelConfig};
use truce_egui::EguiEditor;

const MAX_CHUNK: usize = 512;
/// The piano modelled: its serial seeds every tine's own spread.
pub const SERIAL: u32 = 1975;

pub struct VelvetState {
    engine: Option<Engine>,
    mono: Vec<f32>,
    right: Vec<f32>,
    last_epoch: u32,
    /// The mod wheel's say over the Leslie, until the Rotor control moves.
    wheel: Option<Rotor>,
    rotor_param: usize,
    tail: u32,
    sustain: bool,
}

impl Default for VelvetState {
    fn default() -> Self {
        Self { engine: None, mono: vec![0.0; MAX_CHUNK], right: vec![0.0; MAX_CHUNK], last_epoch: 0, wheel: None, rotor_param: 0, tail: 0, sustain: false }
    }
}

pub struct Velvet;

impl Velvet {
    /// Returns true on a panic request.
    fn handle_event(state: &mut VelvetState, c: &Controls, body: &EventBody) -> bool {
        let engine = state.engine.as_mut().expect("engine");
        match *body {
            EventBody::NoteOn { note, velocity, .. } => engine.note_on(note, f32::from(velocity) / 127.0, c),
            EventBody::NoteOff { note, .. } => engine.note_off(note, c),
            EventBody::ControlChange { cc: 64, value, .. } => {
                state.sustain = value >= 64;
                engine.sustain(state.sustain);
            }
            // Mod wheel: up is fast, down is slow, as organists use it.
            EventBody::ControlChange { cc: 1, value, .. } => state.wheel = Some(if value >= 64 { Rotor::Fast } else { Rotor::Slow }),
            EventBody::ControlChange { cc: 123, .. } => engine.all_notes_off(),
            EventBody::ControlChange { cc: 120, .. } => return true,
            _ => {}
        }
        false
    }

    /// A preset change puts a new instrument on the stand: silence the old one.
    fn check_preset(state: &mut VelvetState, params: &VelvetParams) {
        let epoch = params.ui.preset_epoch.load(Ordering::Relaxed);
        if epoch != state.last_epoch
            && let Some(e) = state.engine.as_mut()
        {
            e.panic();
            state.sustain = false;
        }
        state.last_epoch = epoch;
    }

    fn publish(state: &VelvetState, ui: &UiShared) {
        let Some(engine) = state.engine.as_ref() else { return };
        for k in 0..KEYS {
            store_f32(&ui.level[k], engine.levels[k]);
        }
        ui.sustain.store(u32::from(state.sustain), Ordering::Relaxed);
        store_f32(&ui.horn, engine.leslie.horn_angle);
        store_f32(&ui.drum, engine.leslie.drum_angle);
    }
}

impl PluginLogic for Velvet {
    type Params = VelvetParams;
    type DspState = VelvetState;

    fn bus_layouts() -> Vec<BusLayout> {
        vec![BusLayout::new().with_output("Main", ChannelConfig::Stereo)]
    }

    fn reset(state: &mut Self::DspState, params: &Self::Params, config: &AudioConfig) {
        let rebuild = state.engine.as_ref().is_none_or(|e| (e.host_rate() - config.sample_rate).abs() > 0.5);
        if rebuild {
            state.engine = Some(Engine::new(config.sample_rate, SERIAL));
        }
        state.last_epoch = params.ui.preset_epoch.load(Ordering::Relaxed);
        state.tail = (config.sample_rate * 6.0) as u32;
    }

    fn latency(state: &Self::DspState) -> u32 {
        state.engine.as_ref().map_or(0, |e| e.latency().round() as u32)
    }

    fn tail(state: &Self::DspState) -> u32 {
        state.tail
    }

    fn process(state: &mut Self::DspState, params: &Self::Params, buffer: &mut AudioBuffer, events: &EventList, _context: &mut ProcessContext) -> ProcessStatus {
        let _ftz = denormals::NoDenormals::new();
        let n = buffer.num_samples();
        if n == 0 || state.engine.is_none() {
            return ProcessStatus::Normal;
        }
        Self::check_preset(state, params);
        let mut c = params.controls();
        let rp = params.rotor.value().to_index();
        if rp != state.rotor_param {
            state.rotor_param = rp;
            state.wheel = None;
        }
        if let Some(r) = state.wheel {
            c.leslie.rotor = r;
        }
        let n_out = buffer.num_output_channels();
        let mut ev = 0;
        let mut done = 0;
        while done < n {
            while let Some(e) = events.get(ev) {
                if e.sample_offset as usize > done {
                    break;
                }
                if Self::handle_event(state, &c, &e.body) {
                    state.engine.as_mut().expect("engine").panic();
                    state.sustain = false;
                }
                if let Some(r) = state.wheel {
                    c.leslie.rotor = r;
                }
                ev += 1;
            }
            let next = events.get(ev).map_or(n, |e| (e.sample_offset as usize).min(n));
            let end = next.max(done + 1).min(done + MAX_CHUNK).min(n);
            let len = end - done;
            let engine = state.engine.as_mut().expect("engine");
            engine.process_stereo(&c, &mut state.mono[..len], &mut state.right[..len]);
            if n_out >= 2 {
                buffer.output(0)[done..end].copy_from_slice(&state.mono[..len]);
                buffer.output(1)[done..end].copy_from_slice(&state.right[..len]);
            } else if n_out == 1 {
                for (o, (l, r)) in buffer.output(0)[done..end].iter_mut().zip(state.mono[..len].iter().zip(&state.right[..len])) {
                    *o = 0.5 * (l + r);
                }
            }
            done = end;
        }
        params.ui.rotor.store(match c.leslie.rotor {
            Rotor::Slow => 0,
            Rotor::Fast => 1,
            Rotor::Stop => 2,
        }, Ordering::Relaxed);
        Self::publish(state, &params.ui);
        ProcessStatus::Normal
    }

    fn editor(params: Arc<Self::Params>) -> Box<dyn Editor> {
        EguiEditor::new(params, (ui::WINDOW_W, ui::WINDOW_H), ui::velvet_ui)
            .with_visuals(ui::visuals())
            .resizable(true)
            .min_size((ui::MIN_W, ui::MIN_H))
            .max_size((ui::MAX_W, ui::MAX_H))
            .aspect_ratio(Some((ui::WINDOW_W, ui::WINDOW_H)))
            .into_editor()
    }
}

truce::plugin! {
    logic: Velvet,
    params: VelvetParams,
}

#[cfg(test)]
mod tests;
