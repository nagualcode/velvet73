"""Frequency response of harp (R+L source) -> Stage nameboard (50k rev-audio rheostat // 47nF, 10k audio volume)
-> cable C -> amp input (1M). Nodal analysis, complex phasors. Analysis only."""
import numpy as np
f = np.logspace(np.log10(20), np.log10(20000), 400); w = 2*np.pi*f
def resp(L=0.5, Rdc=1425., bass=1.0, vol=1.0, Ccab=500e-12, Rin=1e6, nameboard=True, Cbb=47e-9, Rbb=50e3, Rvol=10e3):
    Zs = Rdc + 1j*w*L
    Yload_amp = 1/Rin + 1j*w*Ccab
    if not nameboard:
        return 1/(1+Zs*Yload_amp)
    # bass knob: bass=1 -> rheostat 0 ohm (full bass); reverse-audio: R = Rbb*(1-bass)^~ (approx audio law)
    Rr = Rbb*((10**(2*(1-bass))-1)/99) + 1e-3
    Zbb = 1/(1/Rr + 1j*w*Cbb)
    # volume 10k audio: wiper fraction a (audio law), top part Rt, bottom Rb; output at wiper loaded by amp
    a = (10**(2*vol)-1)/99
    Rb = max(Rvol*a,1e-3); Rt = max(Rvol-Rb,1e-3)
    # node A (after bass network): current through Zs+Zbb into (Rt + (Rb || Zamp))
    Zamp = 1/Yload_amp
    Zlow = 1/(1/Rb + 1/Zamp)
    Zleg = Rt + Zlow
    vA = Zleg/(Zs + Zbb + Zleg)
    return vA * Zlow/Zleg
def db(x): return 20*np.log10(np.abs(x))
def m3(h):
    d = db(h); ref = d[np.argmin(abs(f-500))]; idx = np.where((f>1000)&(d<ref-3))[0]
    return f[idx[0]] if len(idx) else float('nan')
at = lambda h,F: db(h)[np.argmin(abs(f-F))]
print("A) Treble -3 dB point (re 500 Hz), volume full, bass full, cable 500 pF, amp 1M")
for L in (0.33,0.5,0.85):
    print(f"  L={L} H: Stage nameboard {m3(resp(L)):6.0f} Hz | direct harp->1M (no nameboard) peak {f[np.argmax(db(resp(L,nameboard=False)))]:6.0f} Hz, +{db(resp(L,nameboard=False)).max():.1f} dB")
print("\nB) Bass Boost knob (L=0.5): level re full-bass at 50/100/300/1k/3k Hz")
ref = resp(0.5, bass=1)
for b in (1.0,0.75,0.5,0.25,0.0):
    h=resp(0.5,bass=b); print(f"  bass={b:4.2f}: "+" ".join(f"{at(h,F)-at(ref,F):6.1f}" for F in (50,100,300,1000,3000)))
print("\nC) Volume knob (L=0.5, bass full): level at 1 kHz and -3 dB point")
for v in (1.0,0.8,0.6,0.4):
    h=resp(0.5,vol=v); print(f"  vol={v:3.1f}: {at(h,1000):6.1f} dB  f-3dB {m3(h):6.0f} Hz")
print("\nD) Cable capacitance (L=0.5, nameboard, vol full)")
for C in (150e-12,500e-12,1500e-12):
    print(f"  C={C*1e12:.0f} pF: f-3dB {m3(resp(0.5,Ccab=C)):6.0f} Hz")
print("\nE) Mod popular (50k audio vol, 250k bass, 10 nF) L=0.5: f-3dB", round(m3(resp(0.5,Rvol=50e3,Rbb=250e3,Cbb=10e-9))))
