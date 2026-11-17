import sys, numpy as np
from PIL import Image
im = np.array(Image.open(sys.argv[1]).convert('L')) < 120
H, W = im.shape
def run_from(y, x, step):
    # walk while black, tolerating 2px gaps
    gap = 0; last = x
    while 0 <= x < W:
        if im[y, x]: gap = 0; last = x
        else:
            gap += 1
            if gap > 2: break
        x += step
    return last
# left frame column: find leftmost heavy column
colsum = im.sum(0)
xl = int(np.argmax(colsum[:80] > 300)); xr = W-1-int(np.argmax(colsum[::-1][:150] > 300))
print("frames", xl, xr, file=sys.stderr)
def bars(xstart, step, ylo, yhi):
    ends = np.array([abs(run_from(y, xstart, step) - xstart) for y in range(H)])
    ends[:ylo] = 0; ends[yhi:] = 0
    # bar rows: runs longer than 60 px; group consecutive rows
    out = []; y = 0
    while y < H:
        if ends[y] > 60:
            y0 = y
            while y < H and ends[y] > 60: y += 1
            seg = ends[y0:y]
            out.append(((y0+y-1)/2, int(np.median(seg)), y-y0))
        else: y += 1
    return out
L = bars(xl+8, 1, 70, 700)
R = bars(xr-8, -1, 240, 720)
print(len(L), len(R), file=sys.stderr)
for c,l,h in L: print("L", round(c,1), l, h)
for c,l,h in R: print("R", round(c,1), l, h)
