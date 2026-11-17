"""Per-key modal table for a 73-key Mark I (keys 8..80 of 88). Sources:
 - ratios: Gabrielli 2020 Fig.20 table (mean, std) for keys with tonebar
 - fundamental decay: Shear 2011 Q values (1974 Mk I) -> dB/s, interpolated log-log; Gabrielli Table I as range
 - inharmonic mode decays: Gabrielli Table II (F1, F3), interpolated vs f in Hz (heuristic)
 - 10 kHz ceiling: Gabrielli (no overtones found above ~10 kHz)
Output: docs/data/modes.csv"""
import numpy as np, sys
shear = np.array([[77.8,949],[155.6,731],[311.1,1520],[622.3,2175],[1244.5,1761]])
dbps = 8.686*np.pi*shear[:,0]/shear[:,1]            # amplitude decay rate of fundamental
gab_assembled = {43.65:6.5, 174.6:3.0, 1396.9:7.5}  # F1, F3, D6? (D6=1174.7 Hz) -- see note
ratios = [("sub",0.48,0.19),("m2",7.1,0.3),("m3",20.4,0.4),("m4",39.7,0.9),("m5",62.9,1.9),("m6",93.1,2.7)]
# inharmonic decay heuristic: rate grows with mode frequency; anchors from Table II (Hz, dB/s)
anch = np.array([[306.2,21.1],[881.8,67.7],[1307,294],[3636,37],[6814,161]])
# robust fit log(rate) ~ a + b log(f)
b,a = np.polyfit(np.log(anch[:,0]), np.log(anch[:,1]), 1)
names=['A','A#','B','C','C#','D','D#','E','F','F#','G','G#']
out=open(sys.argv[1],'w')
out.write("key73,key88,note,f0_hz,fund_decay_dbps,"+",".join(f"{n}_ratio,{n}_std,{n}_hz,{n}_decay_dbps" for n,_,_ in ratios)+"\n")
print("fundamental decay anchors (Shear):", [f"{f:.0f}Hz:{r:.1f}dB/s" for f,r in zip(shear[:,0],dbps)])
print(f"inharmonic decay fit: rate = {np.exp(a):.3f} * f^{b:.2f}  (dB/s, f in Hz) -- weak fit, heuristic")
for k88 in range(8,81):
    f0 = 440*2**((k88-49)/12); n = names[(k88-1)%12]+str((k88+8)//12)
    fd = float(np.exp(np.interp(np.log(f0), np.log(shear[:,0]), np.log(dbps))))
    row=[str(k88-7),str(k88),n,f"{f0:.2f}",f"{fd:.2f}"]
    for nm,mu,sd in ratios:
        fm = mu*f0
        present = fm < 10000 and not (nm in ("m5","m6") and k88 > 30)  # m5/m6 only seen on lowest keys (Fig.20)
        if nm=="sub": dec = fd*1.5   # sub-fundamental: tonebar mode; F1 slow, F3 fast -> heuristic
        else: dec = float(np.exp(a)*fm**b)
        row += [f"{mu}",f"{sd}",f"{fm:.1f}" if present else "",f"{dec:.1f}" if present else ""]
    out.write(",".join(row)+"\n")
