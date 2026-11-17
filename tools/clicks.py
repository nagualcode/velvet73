"""Impulsive events: samples whose 2nd difference exceeds k·σ of a local (50 ms) robust estimate."""
import sys, numpy as np
from scipy.io import wavfile
for fn in sys.argv[1:]:
    sr, x = wavfile.read(fn); x = x[:, 0].astype(float) if x.ndim > 1 else x.astype(float)
    d = np.diff(x, 2); a = np.abs(d)
    w = int(0.05 * sr); n = len(a) // w
    ev = []
    for i in range(n):
        blk = a[i*w:(i+1)*w]; med = np.median(blk) + 1e-9
        m = blk.max()
        if m > 40 * med and m > 1e-3: ev.append((i*w + int(blk.argmax())) / sr)
    print(f"{fn}: {len(ev)} suspicious blocks", [round(t, 3) for t in ev[:12]])
