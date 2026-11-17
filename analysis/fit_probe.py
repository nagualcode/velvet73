import numpy as np
import sys, os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from field import pole_charges, B
ys = np.linspace(0,0.6,61)
for shape in ("cone","wedge"):
    P,Q = pole_charges(shape=shape)
    best=None
    for d0 in np.arange(0.0,0.30,0.02):
        prof={}
        for z in (0.02,0.04,0.06):
            pts=np.stack([np.zeros_like(ys),ys,np.full_like(ys,z+d0)],1)
            prof[z]=np.abs(B(pts,P,Q)[:,1])
        k=40/prof[0.02].max()
        pk=[ys[np.argmax(prof[z])] for z in prof]; mx=[prof[z].max()*k for z in prof]; tail=[prof[z][-1]*k for z in prof]
        err=sum((p-0.2)**2/0.03**2 for p in pk)+(mx[1]-35)**2/4+(mx[2]-31)**2/4+sum((t-11)**2/9 for t in tail)
        print(f"{shape} d0={d0:.2f} peaks={pk} max={[round(m,1) for m in mx]} tail0.6={[round(t,1) for t in tail]} err={err:.1f}")
        if best is None or err<best[0]: best=(err,d0)
    print("BEST",shape,best)
