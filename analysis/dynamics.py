import sys, os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from pickup_sim import proxies, harmonics, ys
def lvl(phi,y0,A,N=4096):
    t=np.arange(N)/N; v=np.gradient(np.interp(y0+A*np.sin(2*np.pi*t),ys,phi))
    X=np.abs(np.fft.rfft(v))[1:11]; rms=np.sqrt((X**2).sum())
    return 20*np.log10(rms), 10*np.log10((X[1:]**2).sum()/X[0]**2+1e-12), X
print("A) Voicing sweep at gap 3.2 mm, A=3 mm (tipBz): offset -> H1..H6 dB rel strongest")
phi=proxies(3.2)["tipBz"]
for y0 in (0,0.5,1,1.5,2,3,4,5,6,8):
    h,_=harmonics(phi,y0,3.0); print(f"  off {y0:4.1f} mm: "+" ".join(f"{x:6.1f}" for x in h[:6]))
print("\nB) Touch dynamics at factory voicing (offset 1.5 mm): amplitude -> output level dB, harmonic/fundamental ratio dB")
for gap in (0.5,1.6,3.2):
    phi=proxies(gap)["tipBz"]; ref=None
    row=[]
    for A in (0.25,0.5,1,2,3,4):
        L,H,_=lvl(phi,1.5,A); ref=L if ref is None else ref
        row.append(f"A{A}:{L-ref:+5.1f}dB/{H:+5.1f}")
    print(f"  gap {gap} mm  "+"  ".join(row))
print("\nC) Absolute sensitivity vs gap (A=1 mm, offset 1.5): dB rel gap 3.2")
base=lvl(proxies(3.2)["tipBz"],1.5,1)[0]
for gap in (0.5,1.0,1.6,2.4,3.2): print(f"  gap {gap}: {lvl(proxies(gap)['tipBz'],1.5,1)[0]-base:+.1f} dB")
