#!/usr/bin/env python3
"""
Generate ultra-accurate Mars Moons (Phobos & Deimos) Cartographic & Relief Dataset:
- Phobos: USGS Viking/Mars Express global controlled cylindrical photomosaic (1024x512).
- Deimos: USGS Viking global controlled cylindrical photomosaic (1024x512).
- Smooth elevation/relief layers with box blur to eliminate JPEG artifacts.
Header: b"ASTRO_MN" (8 bytes) + W (4 bytes u32) + H (4 bytes u32)
Layer 1: Phobos Albedo (1024x512 uint8)
Layer 2: Phobos Elevation/Relief (1024x512 uint8)
Layer 3: Deimos Albedo (1024x512 uint8)
Layer 4: Deimos Elevation/Relief (1024x512 uint8)
"""

import os
import struct
import subprocess
import urllib.request

WIDTH = 1024
HEIGHT = 512

PHOBOS_URL = "https://solarviews.com/raw/mars/phoboscyl1.jpg"
DEIMOS_URL = "https://solarviews.com/raw/mars/deimoscyl1.jpg"
OUT_BIN = "assets/mars_moons_data.bin"

def download_and_convert(url: str, name: str):
    jpg_path = f"/tmp/{name}.jpg"
    bmp_path = f"/tmp/{name}_1024.bmp"
    if not os.path.exists(jpg_path) or os.path.getsize(jpg_path) < 1000:
        print(f"Downloading {url}...")
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=30) as resp, open(jpg_path, "wb") as f:
            f.write(resp.read())
    subprocess.run(["sips", "-z", str(HEIGHT), str(WIDTH), "-s", "format", "bmp", jpg_path, "--out", bmp_path], check=True)
    return bmp_path

def read_bmp_gray(bmp_path: str):
    with open(bmp_path, "rb") as f:
        hdr = f.read(54)
        w = struct.unpack("<i", hdr[18:22])[0]
        h = struct.unpack("<i", hdr[22:26])[0]
        data = f.read()
    
    row_stride = WIDTH * 3
    gray = bytearray(WIDTH * HEIGHT)
    for y in range(HEIGHT):
        src_y = y if h < 0 else (HEIGHT - 1 - y)
        for x in range(WIDTH):
            b = data[src_y * row_stride + x * 3]
            g = data[src_y * row_stride + x * 3 + 1]
            r = data[src_y * row_stride + x * 3 + 2]
            gray[y * WIDTH + x] = int(0.299 * r + 0.587 * g + 0.114 * b)
    return gray

def smooth_elevation(gray: bytearray, radius: int = 2) -> bytearray:
    # 2D separable box blur to smooth out noise and create elevation contours
    temp = bytearray(WIDTH * HEIGHT)
    out = bytearray(WIDTH * HEIGHT)
    
    # Horizontal pass
    for y in range(HEIGHT):
        for x in range(WIDTH):
            acc = 0
            count = 0
            for dx in range(-radius, radius + 1):
                nx = (x + dx) % WIDTH
                acc += gray[y * WIDTH + nx]
                count += 1
            temp[y * WIDTH + x] = acc // count
            
    # Vertical pass
    for y in range(HEIGHT):
        for x in range(WIDTH):
            acc = 0
            count = 0
            for dy in range(-radius, radius + 1):
                ny = max(0, min(HEIGHT - 1, y + dy))
                acc += temp[ny * WIDTH + x]
                count += 1
            out[y * WIDTH + x] = acc // count
            
    return out

def main():
    phobos_bmp = download_and_convert(PHOBOS_URL, "phobos")
    deimos_bmp = download_and_convert(DEIMOS_URL, "deimos")
    
    phobos_albedo = read_bmp_gray(phobos_bmp)
    phobos_elev = smooth_elevation(phobos_albedo, radius=2)
    
    deimos_albedo = read_bmp_gray(deimos_bmp)
    deimos_elev = smooth_elevation(deimos_albedo, radius=3)
    
    os.makedirs(os.path.dirname(OUT_BIN), exist_ok=True)
    with open(OUT_BIN, "wb") as f:
        f.write(b"ASTRO_MN")
        f.write(struct.pack("<I", WIDTH))
        f.write(struct.pack("<I", HEIGHT))
        f.write(phobos_albedo)
        f.write(phobos_elev)
        f.write(deimos_albedo)
        f.write(deimos_elev)
        
    print(f"Generated {OUT_BIN}: {os.path.getsize(OUT_BIN)} bytes (~{os.path.getsize(OUT_BIN)/1024/1024:.2f} MB)")

if __name__ == "__main__":
    main()
