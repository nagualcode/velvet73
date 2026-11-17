import sys, os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from pickup_sim import proxies, harmonics
T5 = np.array([0,-15,-29,-30,-44]); TA = np.array([-19,0,-25,-13,-23,-33])
res=[]
for gap in (0.5,1.0,1.6,2.4,3.2):
    pr = proxies(gap)
    for name,phi in pr.items():
        for A in (0.5,1.0,1.5,2.0,2.5,3.0):
            # on-axis case: small residual offset d
            bestA=min(((np.abs(harmonics(phi,d,A)[0][:6]-TA).mean(),d) for d in np.arange(0,0.81,0.1)))
            best5=min(((np.abs(harmonics(phi,y,A)[0][:5]-T5).mean(),y) for y in np.arange(1,7.01,0.25)))
            res.append((bestA[0]+best5[0],name,gap,A,bestA[1],best5[1],bestA[0],best5[0]))
res.sort()
print("score | proxy | gap mm | amp mm | on-axis residual offset mm | '5mm' offset that fits | errA dB | err5 dB")
for r in res[:12]: print(" ".join(f"{x:.2f}" if isinstance(x,float) else str(x) for x in r))
