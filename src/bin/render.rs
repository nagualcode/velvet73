//! Offline renderer for listening tests and analysis.
//!
//! velvet73-render <out.wav> <preset file | default> [script] [seconds] [rate] [velocity]
//!
//! Scripts: single (G3 alone), dyn (a Fmaj9 voicing at pp, mf, ff),
//! chord (held Fmaj9 with pedal), ballad (a voiced progression with
//! pedal), scale (E1..E7 in fifths), low (bass octaves).
//! VELVET_PROBE=1 prints tine and voltage levels every 100 ms.

use velvet73::dsp::engine::Engine;
use velvet73::params::{VelvetParams, controls_from_preset};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "velvet73.wav".into());
    let preset = a.get(2).cloned().unwrap_or_else(|| "default".into());
    let script = a.get(3).cloned().unwrap_or_else(|| "dyn".into());
    let secs: f64 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(8.0);
    let fs: f64 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(48_000.0);
    let vel: f32 = a.get(6).and_then(|s| s.parse().ok()).unwrap_or(0.65);
    let c = if preset == "default" {
        VelvetParams::new().controls()
    } else {
        controls_from_preset(&std::fs::read_to_string(&preset).expect("preset file")).expect("preset")
    };
    // (time on, time off, note, velocity); pedal: (time down, time up)
    let fmaj9 = [41u8, 52, 55, 57, 60];
    let mut notes: Vec<(f64, f64, u8, f32)> = Vec::new();
    let mut pedal: Vec<(f64, f64)> = Vec::new();
    match script.as_str() {
        "single" => notes.push((0.1, secs * 0.6, 55, vel)),
        "f3" => notes.push((0.1, secs * 0.8, 53, vel)),
        "f1" => notes.push((0.1, secs * 0.8, 29, vel)),
        "idle" => {}
        "chord" => {
            pedal.push((0.05, secs * 0.7));
            for &n in &fmaj9 {
                notes.push((0.1, 0.6, n, vel));
            }
        }
        "dyn" => {
            for (k, v) in [0.25f32, 0.6, 1.0].iter().enumerate() {
                let t = 0.1 + k as f64 * secs / 3.0;
                for &n in &fmaj9 {
                    notes.push((t, t + secs / 3.0 * 0.8, n, *v));
                }
            }
        }
        "ballad" => {
            let prog: [&[u8]; 4] = [&[41, 52, 55, 57, 60, 64], &[38, 50, 53, 57, 60, 64], &[43, 53, 57, 60, 62, 65], &[36, 52, 55, 58, 62, 67]];
            let bar = secs / 4.2;
            for (k, ch) in prog.iter().enumerate() {
                let t = 0.1 + k as f64 * bar;
                pedal.push((t + 0.02, t + bar - 0.05));
                for (i, &n) in ch.iter().enumerate() {
                    let v = if i == 0 { 0.75 } else { 0.45 + 0.05 * i as f32 };
                    notes.push((t + 0.012 * i as f64, t + bar * 0.5, n, v));
                }
            }
        }
        "low" => {
            for (k, &n) in [28u8, 33, 36, 40].iter().enumerate() {
                let t = 0.1 + k as f64 * secs / 4.2;
                notes.push((t, t + secs / 4.2 * 0.85, n, vel));
                notes.push((t, t + secs / 4.2 * 0.85, n + 12, vel * 0.8));
            }
        }
        _ => {
            for (k, &n) in [28u8, 35, 43, 50, 57, 64, 71, 78, 85, 92, 100].iter().enumerate() {
                let t = 0.1 + k as f64 * secs / 11.5;
                notes.push((t, t + secs / 11.5 * 0.85, n, vel));
            }
        }
    }
    let probe = std::env::var("VELVET_PROBE").is_ok();
    let block = 64usize;
    let mut e = Engine::new(fs, velvet73::SERIAL);
    let n = (secs * fs) as usize;
    let mut buf = vec![0.0f32; block];
    let mut all = Vec::with_capacity(n);
    let start = std::time::Instant::now();
    let mut t = 0usize;
    let mut next_probe = 0.0;
    while t < n {
        let time = t as f64 / fs;
        let next = (t + block) as f64 / fs;
        for &(d, u) in &pedal {
            if time <= d && next > d {
                e.sustain(true);
            }
            if time <= u && next > u {
                e.sustain(false);
            }
        }
        for &(on, off, note, v) in &notes {
            if time <= on && next > on {
                e.note_on(note, v, &c);
            }
            if time <= off && next > off {
                e.note_off(note, &c);
            }
        }
        let len = block.min(n - t);
        e.process(&c, &mut buf[..len]);
        all.extend_from_slice(&buf[..len]);
        if probe && time >= next_probe {
            let pk = buf[..len].iter().fold(0.0f32, |m, s| m.max(s.abs()));
            eprintln!("t={time:5.2}s tipG3={:5.2}mm tipF3={:5.2}mm jack={:6.3}V amp={:6.2}V out={:6.3}", e.tip_mm(55), e.tip_mm(53), e.probe[0], e.probe[1], pk);
            e.probe = [0.0; 2];
            next_probe += 0.1;
        }
        t += len;
    }
    let el = start.elapsed().as_secs_f64();
    let peak = all.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let rms = (all.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / all.len() as f64).sqrt();
    eprintln!("{preset} [{script}]: {secs:.1}s in {el:.2}s ({:.1}x rt) peak {:.1} dBFS rms {:.1} dBFS", secs / el, 20.0 * f64::from(peak).log10(), 20.0 * rms.log10());
    let spec = hound::WavSpec { channels: 2, sample_rate: fs as u32, bits_per_sample: 32, sample_format: hound::SampleFormat::Float };
    let mut w = hound::WavWriter::create(&out, spec).expect("wav");
    for s in &all {
        w.write_sample(*s).unwrap();
        w.write_sample(*s).unwrap();
    }
    w.finalize().unwrap();
}
