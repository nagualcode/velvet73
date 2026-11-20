"""Onsets of a re-strike render ("rNN" script): high-frequency energy (>5 kHz)
in the first 15 ms of each strike, relative to the first (tine at rest)."""
import sys, numpy as np
from scipy.io import wavfile
from scipy.signal import butter, sosfilt
fn = sys.argv[1]
sr, x = wavfile.read(fn); x = x.astype(float); x = x.mean(axis=1) if x.ndim > 1 else x
hp = sosfilt(butter(4, 5000, "hp", fs=sr, output="sos"), x)
gaps = [0.31, 0.23, 0.37, 0.27, 0.41, 0.19, 0.33, 0.29, 0.43, 0.21, 0.35, 0.3]
vels = [0.35, 0.5, 0.42, 0.55, 0.3, 0.48, 0.38, 0.52, 0.45, 0.33, 0.58, 0.4]
t, rows = 0.1, []
for g, v in zip(gaps, vels):
    a = int(t * sr); b = a + int(0.015 * sr)
    e_hf = np.sqrt((hp[a:b] ** 2).mean()); e = np.sqrt((x[a:b] ** 2).mean())
    rows.append((t, v, 20 * np.log10(e_hf / e + 1e-12)))
    t += g
ref = rows[0][2]
print(fn, " ".join(f"{v:.2f}:{r - ref:+.0f}" for _, v, r in rows))
