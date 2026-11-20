"""Low-note comparison: harmonic levels, energy outside the harmonic series, and
amplitude wobble of the fundamental, per note of a "lowcmp"/"low" render
(C1, E1, G1 at 0.1 + k·T, T = 3 s).

python3 tools/bass_compare.py a.wav b.wav ...
"""
import sys
import numpy as np
from scipy.io import wavfile
from scipy.signal import butter, sosfiltfilt, hilbert

NOTES = [(24, 32.70), (28, 41.20), (31, 49.00)]


def mono(fn):
    sr, x = wavfile.read(fn)
    x = x.astype(float)
    if x.ndim > 1:
        x = x.mean(axis=1)
    return sr, x


def harmonics(seg, sr, f0, n=8):
    w = np.hanning(len(seg))
    S = np.abs(np.fft.rfft(seg * w))
    f = np.fft.rfftfreq(len(seg), 1 / sr)
    # Actual fundamental: strongest peak within ±4 % (stretch, tuning).
    band = (f > f0 * 0.96) & (f < f0 * 1.04)
    f1 = f[band][np.argmax(S[band])]
    lv = []
    for h in range(1, n + 1):
        b = (f > f1 * h * 0.985) & (f < f1 * h * 1.015)
        lv.append(S[b].max() if b.any() else 1e-12)
    lv = 20 * np.log10(np.array(lv) / lv[0])
    # Energy within ±1.5 % of a harmonic vs everything 20 Hz..8 kHz.
    P = S ** 2
    tot = P[(f > 20) & (f < 8000)].sum()
    harm = sum(P[(f > f1 * h * 0.985) & (f < f1 * h * 1.015)].sum() for h in range(1, 200) if f1 * h < 8000)
    return f1, lv, 10 * np.log10(max(tot - harm, 1e-30) / tot)


def wobble(seg, sr, f1):
    """Peak-to-trough (dB) of the fundamental's envelope, decay removed."""
    sos = butter(4, [f1 * 0.8, f1 * 1.2], btype="band", fs=sr, output="sos")
    env = np.abs(hilbert(sosfiltfilt(sos, seg)))
    k = int(0.02 * sr)
    env = env[k:-k]
    db = 20 * np.log10(env + 1e-12)
    t = np.arange(len(db))
    trend = np.polyval(np.polyfit(t, db, 2), t)
    r = db - trend
    return np.percentile(r, 97) - np.percentile(r, 3)


for fn in sys.argv[1:]:
    sr, x = mono(fn)
    print(f"\n{fn}")
    for k, (midi, f0) in enumerate(NOTES):
        t0 = 0.1 + k * 3.0
        early = x[int((t0 + 0.15) * sr):int((t0 + 0.95) * sr)]
        late = x[int((t0 + 1.6) * sr):int((t0 + 2.5) * sr)]
        if np.abs(early).max() < 1e-6:
            print(f"  midi {midi}: silent")
            continue
        f1, lv, inh = harmonics(early, sr, f0)
        _, lv2, inh2 = harmonics(late, sr, f0)
        wb = wobble(x[int((t0 + 0.15) * sr):int((t0 + 2.5) * sr)], sr, f1)
        print(f"  midi {midi} f1 {f1:6.2f} Hz | early H2..H6 " + " ".join(f"{v:6.1f}" for v in lv[1:6])
              + f" | inharm {inh:6.1f} dB | late H2..H4 " + " ".join(f"{v:6.1f}" for v in lv2[1:4])
              + f" inharm {inh2:6.1f} | f1 wobble {wb:4.1f} dB")
