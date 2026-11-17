"""Cross-check: can our cone-pole field model reproduce the harmonic spectrum of Gabrielli's pickup model
(Pfeifle-Muenster field) applied to a pure sine (F3-fund-only, A2-fund-only)?"""
import sys, os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from pickup_sim import proxies, harmonics
T = {"F3":np.array([0,-9.2,-18.0,-50.5,-44.7,-61.3]), "A2":np.array([0,-11.7,-11.5,-23.6,-38.5,-37.4])}
for note,tgt in T.items():
    best=[]
    for gap in (1.0,1.6,2.4,3.2):
        phi=proxies(gap)["tipBz"]
        for A in np.arange(0.25,4.01,0.25):
            for y0 in np.arange(0.5,6.01,0.25):
                h,_=harmonics(phi,y0,A); e=np.abs(np.maximum(h[:6],-70)-np.maximum(tgt,-70)).mean()
                best.append((e,gap,A,y0,h[:6]))
    best.sort(key=lambda r:r[0])
    for e,g,A,y0,h in best[:3]:
        print(f"{note}: err {e:4.1f} dB  gap {g} mm  A {A:.2f} mm  offset {y0:.2f} mm  model "+" ".join(f"{x:6.1f}" for x in h)+"  target "+" ".join(f"{x:6.1f}" for x in tgt))
