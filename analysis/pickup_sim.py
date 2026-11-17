import sys, os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from field import pole_charges, B
IN=25.4
P,Q = pole_charges(shape="cone", n=36)
# grid in mm: y lateral, s along rod from tip (rod extends away from pole)
ys = np.linspace(-9,9,145); ss = np.linspace(0,10,21)
def proxies(gap_mm):
    Y,S = np.meshgrid(ys, ss, indexing='ij')
    pts = np.stack([np.zeros(Y.size), Y.ravel()/IN, (gap_mm+S.ravel())/IN],1)
    b = np.concatenate([B(pts[i:i+200],P,Q) for i in range(0,len(pts),200)]).reshape(len(ys),len(ss),3)
    bz, by = b[...,2], b[...,1]
    w = np.gradient(ss)
    p1 = (bz**2 * w).sum(1)           # axial-magnetized rod, reciprocity  ~ int Bz^2 ds
    p2 = np.abs(by[:,0])              # Lover-style |By| at tip
    p3 = bz[:,0]                      # point-tip Bz (Pfeifle-style flux proxy)
    return {"rodBz2":p1, "absBy":p2, "tipBz":p3}
def harmonics(phi_of_y, y0, A, nh=8, N=4096):
    t = np.arange(N)/N
    y = y0 + A*np.sin(2*np.pi*t)
    phi = np.interp(y, ys, phi_of_y)
    v = np.gradient(phi)              # v = -N dPhi/dt (sign irrelevant)
    X = np.abs(np.fft.rfft(v))[1:nh+1]
    return 20*np.log10(X/X.max()+1e-9), v
def show(gap, A):
    pr = proxies(gap)
    print(f"\n=== gap {gap} mm, amplitude {A} mm (G3, steady state) — dB of H1..H8 rel. strongest")
    for name, phi in pr.items():
        for y0 in (0,2,5):
            h,_ = harmonics(phi, y0, A)
            print(f"{name:7s} off={y0}mm  " + " ".join(f"{x:6.1f}" for x in h))
if __name__=="__main__":
    print("Shear measured (G3): 5mm: H1 0, H2 -15, H3 -29, H4 -30, H5 -44 | on-axis: H1 -19, H2 0, H3 -25, H4 -13, H5 -23, H6 -33")
    for gap in (1.6, 3.2):
        for A in (1.0, 2.0, 3.0):
            show(gap, A)
