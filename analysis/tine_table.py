import numpy as np, sys
rows=[l.split() for l in open(sys.argv[1])]
L=[int(r[2])+8 for r in rows if r[0]=="L"]   # 47 rows: 0(1-7),1-8 ... 46-53
R=[int(r[2])+8 for r in rows if r[0]=="R"]   # 35 rows top->bottom: 81-88 ... 47-54
R=R[::-1]                                    # 47-54 ... 81-88
px={}
for i,v in enumerate(L):
    if i==0:
        for k in range(1,8): px[k]=v
    else: px[i+7]=v
for i,v in enumerate(R): px[54+i]=v
assert sorted(px)==list(range(1,89)), sorted(px)
# scale: fit endpoints to Shear 157 mm (longest) and 18 mm (shortest)
a=(157-18)/(px[1]-px[88]); b=157-a*px[1]
E=200e9; rho=7850; r=0.75e-3
c=np.sqrt(E/rho)*r/2
names=['A','A#','B','C','C#','D','D#','E','F','F#','G','G#']
out=open(sys.argv[2],'w')
out.write("key88,key73,note,f0_hz,chart_px,length_mm_est,f_bare_EB_hz,ratio_bare_over_f0\n")
for k in range(1,89):
    f0=440*2**((k-49)/12); n=names[(k-1)%12]+str((k+8)//12)
    Lm=(a*px[k]+b)/1000; fb=(1.8751**2/(2*np.pi))*c/Lm**2
    k73 = k-7 if 8<=k<=80 else ''
    out.write(f"{k},{k73},{n},{f0:.2f},{px[k]},{Lm*1000:.1f},{fb:.1f},{fb/f0:.3f}\n")
print("scale mm/px",a,"offset",b)
