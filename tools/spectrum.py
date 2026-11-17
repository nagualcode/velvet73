"""Spectrum of the attack (first `dur` s after onset) at given ratios of f0; prints dB re the fundamental.
usage: spectrum.py file.wav f0 [dur] [onset]"""
import sys, numpy as np
from scipy.io import wavfile
sr, x = wavfile.read(sys.argv[1]); x = x[:, 0] if x.ndim > 1 else x
f0 = float(sys.argv[2]); dur = float(sys.argv[3]) if len(sys.argv) > 3 else 0.2; on = float(sys.argv[4]) if len(sys.argv) > 4 else 0.1
seg = x[int(on*sr):int((on+dur)*sr)].astype(float)
N = 1 << 18
X = np.abs(np.fft.rfft(seg*np.blackman(len(seg)), N)); fr = np.fft.rfftfreq(N, 1/sr)
def at(f, tol=0.03):
    m = (fr > f*(1-tol)) & (fr < f*(1+tol)); return 20*np.log10(X[m].max()+1e-20) if m.any() else -999
ref = at(f0)
ratios = [0.58, 1, 1.58, 2, 3, 4, 5, 6, 7.4, 20.66, 38.76]
print(" ".join(f"{r:>6}" for r in ratios)); print(" ".join(f"{at(r*f0, 0.025 if r>6 else 0.01)-ref:6.1f}" for r in ratios))
