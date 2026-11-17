"""Harmonic analysis of Gabrielli et al. companion audio (these are SIMULATIONS by the authors, not recordings)."""
import sys, glob, os, numpy as np
from scipy.io import wavfile
from scipy.signal import find_peaks
d = sys.argv[1]
for fn in sorted(glob.glob(os.path.join(d,"*.wav"))):
    sr, x = wavfile.read(fn); x = x.astype(float); x = x if x.ndim==1 else x[:,0]
    n = len(x); seg = x[int(0.05*sr):int(0.05*sr)+min(16384, n-int(0.05*sr))]
    X = np.abs(np.fft.rfft(seg*np.blackman(len(seg)), 1<<17)); fr = np.fft.rfftfreq(1<<17, 1/sr)
    lo = fr>30; f0 = fr[lo][np.argmax(X[lo])]
    # refine f0 as lowest strong peak
    pk,_ = find_peaks(20*np.log10(X+1e-12), prominence=20)
    strong = [p for p in pk if 20*np.log10(X[p]/X.max())>-30 and fr[p]>30]
    f0 = fr[strong[0]]
    H = [20*np.log10(X[np.argmin(abs(fr-k*f0))-3:np.argmin(abs(fr-k*f0))+4].max()/X.max()) for k in range(1,9)]
    env = np.abs(x); blk = int(0.05*sr); e = [20*np.log10(env[i:i+blk].max()+1e-9) for i in range(0, n-blk, blk)]
    slope = np.polyfit(np.arange(len(e))*0.05, e, 1)[0]
    print(f"{os.path.basename(fn):22s} sr={sr} dur={n/sr:.2f}s f0={f0:7.1f}  H1..H8 dB: "+" ".join(f"{h:6.1f}" for h in H)+f"  decay {slope:6.1f} dB/s")
