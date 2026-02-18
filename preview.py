#!/usr/bin/env python3
"""Quick preview of 16x16 PNGs as halfblock pixel art in the terminal.

Usage: python3 preview.py image1.png image2.png ...
"""
import struct, sys, zlib

def decode_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    pos, w, h, ct, bd = 8, 0, 0, 0, 0
    raw, pal, trns = b"", [], b""
    while pos < len(data):
        ln = struct.unpack(">I", data[pos:pos+4])[0]
        tp, cd = data[pos+4:pos+8], data[pos+8:pos+8+ln]
        pos += 12 + ln
        if   tp == b"IHDR": w, h = struct.unpack(">II", cd[:8]); bd, ct = cd[8], cd[9]
        elif tp == b"PLTE": pal = [(cd[i], cd[i+1], cd[i+2]) for i in range(0, len(cd), 3)]
        elif tp == b"tRNS": trns = cd
        elif tp == b"IDAT": raw += cd
        elif tp == b"IEND": break
    dec = zlib.decompress(raw)

    def paeth(a, b, c):
        p = a+b-c; pa, pb, pc = abs(p-a), abs(p-b), abs(p-c)
        return a if pa<=pb and pa<=pc else (b if pb<=pc else c)

    def unfilt(rows, bpp):
        out = []
        for y, row in enumerate(rows):
            f, r = row[0], list(row[1:])
            prev = out[-1] if out else [0]*len(r)
            if   f==1:
                for i in range(bpp,len(r)): r[i]=(r[i]+r[i-bpp])&0xff
            elif f==2:
                for i in range(len(r)): r[i]=(r[i]+prev[i])&0xff
            elif f==3:
                for i in range(len(r)): r[i]=(r[i]+((r[i-bpp] if i>=bpp else 0)+prev[i])//2)&0xff
            elif f==4:
                for i in range(len(r)): r[i]=(r[i]+paeth(r[i-bpp] if i>=bpp else 0, prev[i], prev[i-bpp] if i>=bpp else 0))&0xff
            out.append(r)
        return out

    pixels = []
    if ct in (2, 6):
        bpp = 4 if ct==6 else 3
        rb = w*bpp+1
        rows = [dec[y*rb:(y+1)*rb] for y in range(h)]
        sc = unfilt(rows, bpp)
        for r in sc:
            for x in range(w):
                if bpp==4: pixels.append((r[x*4], r[x*4+1], r[x*4+2], r[x*4+3]))
                else:      pixels.append((r[x*3], r[x*3+1], r[x*3+2], 255))
    elif ct == 3:
        rb = ({8:w, 4:(w+1)//2, 2:(w+3)//4, 1:(w+7)//8}[bd]) + 1
        rows = [dec[y*rb:(y+1)*rb] for y in range(h)]
        sc = unfilt(rows, 1)
        for r in sc:
            for x in range(w):
                if   bd==8: idx=r[x]
                elif bd==4: idx=(r[x//2]>>(4*(1-x%2)))&0xf
                elif bd==2: idx=(r[x//4]>>(2*(3-x%4)))&0x3
                else:       idx=(r[x//8]>>(7-x%8))&0x1
                c = pal[idx]; a = trns[idx] if idx<len(trns) else 255
                pixels.append((c[0], c[1], c[2], a))
    return w, h, pixels

def render(w, h, px):
    for y in range(0, h, 2):
        print("  ", end="")
        for x in range(w):
            t = px[y*w+x]; b = px[(y+1)*w+x] if y+1<h else (0,0,0,0)
            ta, ba = t[3]>=128, b[3]>=128
            if   not ta and not ba: print(" ", end="")
            elif not ta:            print(f"\033[38;2;{b[0]};{b[1]};{b[2]}m▄\033[0m", end="")
            elif not ba:            print(f"\033[38;2;{t[0]};{t[1]};{t[2]}m▀\033[0m", end="")
            else:                   print(f"\033[38;2;{t[0]};{t[1]};{t[2]};48;2;{b[0]};{b[1]};{b[2]}m▀\033[0m", end="")
        print()

if len(sys.argv) < 2:
    print("Usage: python3 preview.py image1.png image2.png ...")
    sys.exit(1)

for path in sys.argv[1:]:
    w, h, px = decode_png(path)
    print(f"\n  {path} ({w}x{h})")
    render(w, h, px)
print()
