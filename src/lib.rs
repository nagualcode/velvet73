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
use params::VelvetParams;
use params::VelvetParamsParamId as P;
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
    last_panic: u32,
    last_epoch: u32,
    panic_param: bool,
    tail: u32,
    sustain: bool,
}

impl Default for VelvetState {
    fn default() -> Self {
        Self { engine: None, mono: vec![0.0; MAX_CHUNK], last_panic: 0, last_epoch: 0, panic_param: false, tail: 0, sustain: false }
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
            EventBody::ControlChange { cc: 123, .. } => engine.all_notes_off(),
            EventBody::ControlChange { cc: 120, .. } => return true,
            _ => {}
        }
        false
    }

    fn check_panic(state: &mut VelvetState, params: &VelvetParams) {
        let n = params.ui.panic.load(Ordering::Relaxed);
        let epoch = params.ui.preset_epoch.load(Ordering::Relaxed);
        let p = params.panic.value();
        let rising = p && !state.panic_param;
        state.panic_param = p;
        if (n != state.last_panic || epoch != state.last_epoch || rising)
            && let Some(e) = state.engine.as_mut()
        {
            e.panic();
            state.sustain = false;
        }
        state.last_panic = n;
        state.last_epoch = epoch;
    }

    fn publish(state: &VelvetState, ui: &UiShared) {
        let Some(engine) = state.engine.as_ref() else { return };
        for k in 0..KEYS {
            store_f32(&ui.level[k], engine.levels[k]);
        }
        ui.sustain.store(u32::from(state.sustain), Ordering::Relaxed);
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
        state.last_panic = params.ui.panic.load(Ordering::Relaxed);
        state.last_epoch = params.ui.preset_epoch.load(Ordering::Relaxed);
        state.tail = (config.sample_rate * 6.0) as u32;
    }

    fn latency(state: &Self::DspState) -> u32 {
        state.engine.as_ref().map_or(0, |e| e.latency().round() as u32)
    }

    fn tail(state: &Self::DspState) -> u32 {
        state.tail
    }

    fn process(state: &mut Self::DspState, params: &Self::Params, buffer: &mut AudioBuffer, events: &EventList, context: &mut ProcessContext) -> ProcessStatus {
        let _ftz = denormals::NoDenormals::new();
        let n = buffer.num_samples();
        if n == 0 || state.engine.is_none() {
            return ProcessStatus::Normal;
        }
        Self::check_panic(state, params);
        let c = params.controls();
        let n_out = buffer.num_output_channels();
        let mut ev = 0;
        let mut done = 0;
        let mut peak = 0.0f32;
        while done < n {
            while let Some(e) = events.get(ev) {
                if e.sample_offset as usize > done {
                    break;
                }
                if Self::handle_event(state, &c, &e.body) {
                    state.engine.as_mut().expect("engine").panic();
                    state.sustain = false;
                }
                ev += 1;
            }
            let next = events.get(ev).map_or(n, |e| (e.sample_offset as usize).min(n));
            let end = next.max(done + 1).min(done + MAX_CHUNK).min(n);
            let len = end - done;
            let engine = state.engine.as_mut().expect("engine");
            engine.process(&c, &mut state.mono[..len]);
            for k in 0..len {
                peak = peak.max(state.mono[k].abs());
            }
            for ch in 0..n_out.min(2) {
                buffer.output(ch)[done..end].copy_from_slice(&state.mono[..len]);
            }
            done = end;
        }
        Self::publish(state, &params.ui);
        context.set_meter(P::MeterL, peak.min(1.0));
        context.set_meter(P::MeterR, peak.min(1.0));
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
