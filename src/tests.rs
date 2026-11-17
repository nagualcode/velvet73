use super::*;
use crate::dsp::engine::Engine;
use crate::params::controls_from_preset;
use crate::params::VelvetParamsParamId as P;
use std::time::Duration;
use truce_test::{InputSource, assert_no_audio_alloc, assertions, driver};

#[test]
fn info_and_buses_are_valid() {
    truce_test::assert_valid_info::<Plugin>();
    truce_test::assert_bus_config_instrument::<Plugin>();
    truce_test::assert_no_duplicate_param_ids::<Plugin>();
    truce_test::assert_param_defaults_match::<Plugin>();
    truce_test::assert_au_type_codes_ascii::<Plugin>();
    truce_test::assert_fourcc_roundtrip::<Plugin>();
}

/// Renders the editor for the README.
/// `cargo test --release --lib screenshots -- --ignored`
#[test]
#[ignore]
fn screenshots() {
    let params = Arc::new(VelvetParams::new());
    let preset = crate::presets::library().iter().find(|p| p.name == "Factory").unwrap();
    for (k, v) in &preset.values {
        if let Some(id) = crate::ui::key_id(k) {
            params.set_plain(id.into(), *v);
        }
    }
    let ui = &params.ui;
    // A pedalled F major 9 ringing out.
    for (key, mm) in [(21usize, 3.4f32), (32, 2.6), (35, 2.2), (37, 1.9), (40, 1.6), (44, 0.9), (56, 0.4), (28, 0.3), (47, 0.25), (52, 0.2)] {
        store_f32(&ui.level[key - 1], mm);
    }
    ui.sustain.store(1, Ordering::Relaxed);
    std::fs::create_dir_all("docs").unwrap();
    let mut editor = Velvet::editor(params.clone());
    if let Some((px, w, h)) = editor.screenshot(params.clone()) {
        truce_test::save_png(std::path::Path::new("docs/screenshot.png"), &px, w, h);
    }
    // The same piano into the Leslie, horn and drum caught mid-turn.
    params.set_plain(P::Amp.into(), 3.0);
    params.set_plain(P::Rotor.into(), 1.0);
    store_f32(&ui.horn, 2.4);
    store_f32(&ui.drum, 0.7);
    ui.rotor.store(1, Ordering::Relaxed);
    let mut editor = Velvet::editor(params.clone());
    if let Some((px, w, h)) = editor.screenshot(params.clone()) {
        truce_test::save_png(std::path::Path::new("docs/screenshot-leslie.png"), &px, w, h);
    }
}

fn rms(x: &[f32]) -> f32 {
    (x.iter().map(|s| s * s).sum::<f32>() / x.len().max(1) as f32).sqrt()
}

#[test]
fn state_round_trips_and_survives_garbage() {
    truce_test::assert_state_round_trip::<Plugin>();
    truce_test::assert_corrupt_state_no_crash::<Plugin>();
    truce_test::assert_empty_state_no_crash::<Plugin>();
}

/// Untouched, only the amp's own hum and hiss (well below −60 dBFS).
#[test]
fn untouched_it_is_quiet() {
    let r = driver!(Plugin).sample_rate(48_000.0).channels(2).block_size(256).duration(Duration::from_millis(1200)).input(InputSource::Silence).run();
    assertions::assert_no_nans(&r);
    let db = 20.0 * rms(&r.output[0][24_000..]).max(1e-9).log10();
    assert!(db < -60.0, "idle floor too loud: {db:.1} dBFS");
}

#[test]
fn a_key_sounds_and_fades() {
    let r = driver!(Plugin)
        .sample_rate(44_100.0)
        .channels(2)
        .block_size(128)
        .duration(Duration::from_millis(3000))
        .script(|s| {
            s.wait_ms(100);
            s.note_on(57, 0.8);
            s.wait_ms(800);
            s.note_off(57);
        })
        .run();
    assertions::assert_no_nans(&r);
    let sr = 44_100;
    let on = 20.0 * rms(&r.output[0][sr / 4..sr * 8 / 10]).max(1e-9).log10();
    let late = 20.0 * rms(&r.output[0][sr * 5 / 2..sr * 3]).max(1e-9).log10();
    assert!(on > -36.0, "key too quiet: {on:.1} dBFS");
    assert!(late < on - 40.0, "note does not fade after the damper: {on:.1} -> {late:.1}");
}

/// The pitch of the fundamental (from zero crossings of the DI signal at
/// low velocity, where the pickup adds little).
#[test]
fn keys_are_in_tune() {
    let fs = 96_000.0;
    let mut c = VelvetParams::new().controls();
    c.amp = crate::dsp::engine::AmpMode::Di;
    c.stretch = false;
    c.condition = 0.0;
    c.noise = 0.0;
    c.voicing = 1.0; // far off the axis: the fundamental dominates
    for note in [33u8, 45, 57, 69, 81] {
        let mut e = Engine::new(fs, SERIAL);
        e.note_on(note, 0.3, &c);
        let mut buf = vec![0.0f32; 96_000];
        e.process(&c, &mut buf);
        let seg = &buf[20_000..90_000];
        let mut crossings = Vec::new();
        for k in 1..seg.len() {
            if seg[k - 1] < 0.0 && seg[k] >= 0.0 {
                let t = (k - 1) as f64 + f64::from(-seg[k - 1] / (seg[k] - seg[k - 1]));
                crossings.push(t);
            }
        }
        let n = crossings.len() - 1;
        let f = n as f64 * fs / (crossings[n] - crossings[0]);
        let want = 440.0 * 2f64.powf((f64::from(note) - 69.0) / 12.0);
        let cents = 1200.0 * (f / want).log2();
        assert!(cents.abs() < 3.0, "note {note}: {f:.2} Hz, {cents:+.1} cents");
    }
}

#[test]
fn panic_silences_everything_at_once() {
    let r = driver!(Plugin)
        .sample_rate(48_000.0)
        .block_size(64)
        .duration(Duration::from_millis(1600))
        .script(|s| {
            s.cc(64, 1.0);
            for n in [41, 52, 55, 57, 60] {
                s.note_on(n, 1.0);
            }
            s.wait_ms(1000);
            s.cc(120, 0.0); // All Sound Off
            s.wait_ms(580);
        })
        .run();
    assertions::assert_no_nans(&r);
    let sr = 48_000;
    let before = rms(&r.output[0][sr * 8 / 10..sr]);
    let after = rms(&r.output[0][sr + sr / 10..sr + sr / 4]);
    assert!(before > 0.005, "nothing sounding before the panic");
    assert!(after < before * 0.05, "panic did not silence: {before} -> {after}");
}

#[test]
fn process_is_allocation_free() {
    assert_no_audio_alloc(|| {
        driver!(Plugin)
            .duration(Duration::from_millis(150))
            .script(|s| {
                s.cc(64, 1.0);
                s.note_on(60, 0.9);
                s.note_on(67, 0.5);
                s.set_param(P::Voicing, 0.1);
                s.set_param(P::Gap, 0.9);
                s.set_param(P::Treble, 1.0);
                s.set_param(P::Intensity, 0.8);
                s.wait_ms(40);
                s.note_on(72, 0.7);
                s.set_param(P::Board, 0.5);
                s.set_param(P::Condition, 1.0);
                s.set_param(P::Keys, 1.0);
                s.wait_ms(40);
                s.cc(64, 0.0);
                s.note_off(60);
                s.set_param(P::Amp, 3.0); // the Leslie (the driver writes plain values)
                s.cc(1, 1.0);
                s.note_on(64, 0.8);
                s.set_param(P::LeslieWidth, 0.2);
                s.wait_ms(20);
            })
            .run()
    });
}

/// The Leslie: stereo (the two microphones differ), the mod wheel takes
/// the rotor to fast and back, and nothing runs away.
#[test]
fn the_leslie_is_stereo_and_follows_the_wheel() {
    let r = driver!(Plugin)
        .sample_rate(48_000.0)
        .channels(2)
        .block_size(128)
        .duration(Duration::from_millis(3000))
        .script(|s| {
            s.set_param(P::Amp, 3.0);
            s.cc(64, 1.0);
            for n in [41, 52, 55, 57, 60] {
                s.note_on(n, 0.8);
            }
            s.wait_ms(500);
            s.cc(1, 1.0);
        })
        .run();
    assertions::assert_no_nans(&r);
    let (l, rr) = (&r.output[0], &r.output[1]);
    let diff: f32 = l.iter().zip(rr).map(|(a, b)| (a - b) * (a - b)).sum::<f32>() / l.len() as f32;
    let tot: f32 = l.iter().map(|a| a * a).sum::<f32>() / l.len() as f32;
    assert!(diff > 0.01 * tot, "left and right are the same: {diff} vs {tot}");
    let peak = l.iter().chain(rr).fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(peak < 1.0, "peak {peak}");
}

#[test]
fn extreme_settings_stay_finite_and_bounded() {
    let r = driver!(Plugin)
        .sample_rate(48_000.0)
        .duration(Duration::from_millis(2500))
        .script(|s| {
            for p in [P::Voicing, P::Hammer, P::Touch, P::Condition, P::AmpVolume, P::Treble, P::Bass, P::Middle, P::Reverb, P::Intensity, P::Noise, P::Output, P::Keys] {
                s.set_param(p, 1.0);
            }
            s.set_param(P::Gap, 0.0);
            s.cc(64, 1.0);
            for n in 21..109 {
                s.note_on(n, 1.0);
            }
            s.wait_ms(2000);
        })
        .run();
    assertions::assert_no_nans(&r);
    assertions::assert_peak_below(&r, 1.0);
}

#[test]
fn every_preset_parses_sounds_and_stays_bounded() {
    let mut count = 0;
    for cat in std::fs::read_dir("factory").unwrap() {
        for file in std::fs::read_dir(cat.unwrap().path()).unwrap() {
            let path = file.unwrap().path();
            let c = controls_from_preset(&std::fs::read_to_string(&path).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let mut e = Engine::new(48_000.0, SERIAL);
            e.sustain(true);
            for n in [41u8, 52, 55, 57, 60] {
                e.note_on(n, 0.8, &c);
            }
            let mut buf = vec![0.0f32; 512];
            let mut peak = 0.0f32;
            for _ in 0..150 {
                e.process(&c, &mut buf);
                for s in &buf {
                    assert!(s.is_finite() && s.abs() <= 1.0, "{}", path.display());
                    peak = peak.max(s.abs());
                }
            }
            assert!(peak > 0.02, "{} is silent ({peak})", path.display());
            count += 1;
        }
    }
    assert!(count >= 1);
}

/// With nothing ringing and no noise, the engine stops simulating and
/// emits exact zeros; a key wakes it at once.
#[test]
fn a_silent_piano_sleeps_and_wakes_on_a_key() {
    let fs = 48_000.0;
    let mut c = VelvetParams::new().controls();
    c.noise = 0.0;
    let mut e = Engine::new(fs, SERIAL);
    let mut buf = [0.0f32; 64];
    e.note_on(60, 0.7, &c);
    for _ in 0..200 {
        e.process(&c, &mut buf);
    }
    assert!(!e.is_asleep(), "asleep under a held key");
    e.note_off(60, &c);
    for _ in 0..(fs as usize * 8 / 64) {
        e.process(&c, &mut buf);
    }
    assert!(e.is_asleep(), "still simulating eight seconds after the release");
    assert!(buf.iter().all(|v| *v == 0.0));
    e.note_on(64, 0.7, &c);
    let mut peak = 0.0f32;
    for _ in 0..200 {
        e.process(&c, &mut buf);
        peak = buf.iter().fold(peak, |m, v| m.max(v.abs()));
    }
    assert!(!e.is_asleep() && peak > 0.01, "no sound after waking: {peak}");
}

/// Harder playing barks: the pickup's harmonics grow faster than the
/// fundamental (the defining Rhodes behaviour).
#[test]
fn harder_playing_barks() {
    let fs = 96_000.0;
    let mut c = VelvetParams::new().controls();
    c.amp = crate::dsp::engine::AmpMode::Di;
    c.noise = 0.0;
    let ratio = |vel: f32| {
        let mut e = Engine::new(fs, SERIAL);
        e.note_on(53, vel, &c);
        let mut buf = vec![0.0f32; 48_000];
        e.process(&c, &mut buf);
        let seg = &buf[9_600..9_600 + 16_384];
        let f0 = 174.61;
        let mag = |f: f64| {
            let (mut re, mut im) = (0.0f64, 0.0f64);
            for (k, x) in seg.iter().enumerate() {
                let ph = std::f64::consts::TAU * f * k as f64 / fs;
                re += f64::from(*x) * ph.cos();
                im += f64::from(*x) * ph.sin();
            }
            (re * re + im * im).sqrt()
        };
        20.0 * (mag(2.0 * f0) / mag(f0)).log10()
    };
    let soft = ratio(0.25);
    let hard = ratio(0.9);
    assert!(hard > soft + 10.0, "2nd harmonic re fundamental: pp {soft:.1} dB, ff {hard:.1} dB");
}

/// A key held down (damper lifted, no pedal) catches the blow of another
/// key through the harp rail; a key at rest does not.
#[test]
fn a_held_key_catches_the_rail() {
    let c = Controls::default();
    let fs = 48_000.0;
    let run = |hold: bool, strike: bool| -> f32 {
        let mut out = vec![0.0f32; 256];
        let mut e = Engine::new(fs, SERIAL);
        if hold {
            // F5 pressed softly and held while it rings down.
            e.note_on(77, 0.02, &c);
            for _ in 0..(fs as usize * 4 / 256) {
                e.process(&c, &mut out);
            }
        }
        if strike {
            e.note_on(65, 1.0, &c); // F4, hard
        }
        for _ in 0..40 {
            e.process(&c, &mut out);
        }
        e.tip_mm(77)
    };
    let held = (run(true, true) - run(true, false)).abs();
    let rest = run(false, true);
    eprintln!("F5 tip change from an F4 blow: held {held:.4} mm (of {:.4}), at rest {rest:.6} mm", run(true, false));
    assert!(held > 1e-3, "held key did not ring: {held}");
    assert!(rest < 1e-6, "damped key rang: {rest}");
}
