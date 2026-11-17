# Magnetostatic model of the standard Rhodes pole piece (US 4,040,321 Fig. 8, "prior art" pole 54).
# Uniform axial magnetization -> surface magnetic charge sigma = M.n on pole surfaces.
# Geometry from the drawing (drawn to the same scale as the probe axis): units = inches.
import numpy as np
IN = 25.4
def pole_charges(r_flat=0.039, r_base=0.145, h=0.127, mag_len=0.5, r_mag=0.094, n=60, shape="cone"):
    """Return (points[N,3], q[N]) discretized surface charges. Pole tip flat at z=0, body toward -z."""
    P = []; Q = []
    if shape == "cone":
        # flat top disc z=0, sigma=+1
        for r in np.linspace(0, r_flat, n//3)[1:]:
            dr = r_flat/(n//3-1); m = max(6, int(2*np.pi*r/dr))
            for t in np.linspace(0, 2*np.pi, m, endpoint=False):
                P.append((r*np.cos(t), r*np.sin(t), 0)); Q.append(2*np.pi*r*dr/m)
        P.append((0,0,0)); Q.append(np.pi*(dr/2)**2)
        # conical flank from (r_flat,0) to (r_base,-h): normal has z-comp = (r_base-r_flat)/slant
        slant = np.hypot(r_base-r_flat, h); nz = (r_base-r_flat)/slant
        for s in np.linspace(0, 1, n):
            r = r_flat + s*(r_base-r_flat); z = -s*h; ds = slant/(n-1)
            m = max(8, int(2*np.pi*r/ds))
            for t in np.linspace(0, 2*np.pi, m, endpoint=False):
                P.append((r*np.cos(t), r*np.sin(t), z)); Q.append(nz*2*np.pi*r*ds/m)
        # underside annulus of head (z=-h-0.03 rim, approx at z=-h) between r_mag and r_base: sigma=-1
        for r in np.linspace(r_mag, r_base, n//3):
            dr = (r_base-r_mag)/(n//3-1); m = max(8, int(2*np.pi*r/dr))
            for t in np.linspace(0, 2*np.pi, m, endpoint=False):
                P.append((r*np.cos(t), r*np.sin(t), -h-0.03)); Q.append(-2*np.pi*r*dr/m)
        # far end of magnet: disc radius r_mag, sigma=-1 (closes the flux budget)
        for r in np.linspace(0, r_mag, n//3)[1:]:
            dr = r_mag/(n//3-1); m = max(6, int(2*np.pi*r/dr))
            for t in np.linspace(0, 2*np.pi, m, endpoint=False):
                P.append((r*np.cos(t), r*np.sin(t), -h-0.03-mag_len)); Q.append(-2*np.pi*r*dr/m)
    elif shape == "wedge":
        # chisel: flat strip |y|<=r_flat, |x|<=r_base at z=0; two flanks slanting in y; underside; magnet end
        nx = n
        xs = np.linspace(-r_base, r_base, nx); dx = xs[1]-xs[0]
        for x in xs:
            for y in np.linspace(-r_flat, r_flat, n//4):
                P.append((x,y,0)); Q.append(dx*(2*r_flat/(n//4)))
            slant = np.hypot(r_base-r_flat, h); nz=(r_base-r_flat)/slant
            for s in np.linspace(0,1,n//2):
                for sg in (-1,1):
                    P.append((x, sg*(r_flat+s*(r_base-r_flat)), -s*h)); Q.append(nz*dx*slant/(n//2))
        for r in np.linspace(0, r_mag, n//3)[1:]:
            dr = r_mag/(n//3-1); m = max(6, int(2*np.pi*r/dr))
            for t in np.linspace(0, 2*np.pi, m, endpoint=False):
                P.append((r*np.cos(t), r*np.sin(t), -h-0.03-mag_len)); Q.append(-2*np.pi*r*dr/m)
        # rebalance total charge to zero on the far end
    P = np.array(P); Q = np.array(Q)
    return P, Q

def B(points, P, Q):
    d = points[:,None,:] - P[None,:,:]
    r3 = np.linalg.norm(d, axis=2)**3 + 1e-12
    return (Q[None,:,None]*d/r3[:,:,None]).sum(1)

if __name__ == "__main__":
    import sys
    shape = sys.argv[1] if len(sys.argv)>1 else "cone"
    P,Q = pole_charges(shape=shape)
    ys = np.array([0,0.05,0.1,0.15,0.2,0.25,0.3,0.4,0.5,0.6])
    patent = {0.02:[0,None,None,None,40,None,None,None,None,12],
              0.04:[0,None,None,None,35,None,None,None,None,10],
              0.06:[0,None,None,None,31,None,None,None,None,10]}
    res = {}
    for z in (0.02,0.04,0.06):
        pts = np.stack([np.zeros_like(ys), ys, np.full_like(ys,z)],1)
        res[z] = B(pts,P,Q)
    k = 40/np.abs(res[0.02][:,1]).max()
    print(f"[{shape}] |By| scaled so max at 0.020in = 40 G ; columns = y (in)")
    print("y    ", " ".join(f"{y:5.2f}" for y in ys))
    for z in (0.02,0.04,0.06):
        print(f"z={z}", " ".join(f"{abs(v)*k:5.1f}" for v in res[z][:,1]), " peak@", ys[np.argmax(np.abs(res[z][:,1]))])
    for z in (0.02,0.04,0.06):
        print(f"Bz z={z}", " ".join(f"{v*k:5.1f}" for v in res[z][:,2]))
