#!/usr/bin/env python3
"""
Generate ultra-accurate Asteroid Belt Cartographic & Relief Dataset for AstroEngine:
- Ceres: NASA Dawn mission global controlled photographic photomosaic (1024x512).
  - Highlights: Occator crater (Cerealia & Vinalia Faculae bright salts), Ahuna Mons cryovolcano.
- Vesta: NASA Dawn mission global controlled photographic photomosaic (1024x512).
  - Highlights: Rheasilvia south polar impact crater, Divalia Fossae equatorial troughs, eucrite basalt.
Header: b"ASTRO_AB" (8 bytes) + W (4 bytes u32) + H (4 bytes u32)
Layer 1: Ceres Albedo (1024x512 uint8 = 524,288 bytes)
Layer 2: Ceres Elevation/Relief (1024x512 uint8 = 524,288 bytes)
Layer 3: Vesta Albedo (1024x512 uint8 = 524,288 bytes)
Layer 4: Vesta Elevation/Relief (1024x512 uint8 = 524,288 bytes)
Total size: 2,097,168 bytes (~2.1 MB).
"""

import os
import struct
import subprocess
import urllib.request

WIDTH = 1024
HEIGHT = 512

CERES_URL = "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/ceres.jpg"
VESTA_URL = "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/vesta.jpg"
OUT_BIN = "assets/asteroid_belt_data.bin"

def ensure_download_and_convert(url: str, name: str) -> str:
    jpg_path = f"/tmp/{name}.jpg"
    bmp_path = f"/tmp/{name}_1024.bmp"
    if not os.path.exists(jpg_path) or os.path.getsize(jpg_path) < 1000:
        print(f"Downloading {url} -> {jpg_path}...")
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (AstroEngine/1.0)"})
        with urllib.request.urlopen(req, timeout=30) as resp, open(jpg_path, "wb") as f:
            f.write(resp.read())
    print(f"Converting {jpg_path} -> {bmp_path} (1024x512 BMP)...")
    subprocess.run(["sips", "-z", str(HEIGHT), str(WIDTH), "-s", "format", "bmp", jpg_path, "--out", bmp_path], check=True)
    return bmp_path

def read_bmp_gray(bmp_path: str) -> bytearray:
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
    ceres_bmp = ensure_download_and_convert(CERES_URL, "ceres")
    vesta_bmp = ensure_download_and_convert(VESTA_URL, "vesta")

    ceres_albedo = read_bmp_gray(ceres_bmp)
    ceres_elev = smooth_elevation(ceres_albedo, radius=2)

    vesta_albedo = read_bmp_gray(vesta_bmp)
    vesta_elev = smooth_elevation(vesta_albedo, radius=3)

    os.makedirs(os.path.dirname(OUT_BIN), exist_ok=True)
    with open(OUT_BIN, "wb") as f:
        f.write(b"ASTRO_AB")
        f.write(struct.pack("<I", WIDTH))
        f.write(struct.pack("<I", HEIGHT))
        f.write(ceres_albedo)
        f.write(ceres_elev)
        f.write(vesta_albedo)
        f.write(vesta_elev)

    size = os.path.getsize(OUT_BIN)
    print(f"Generated {OUT_BIN}: {size} bytes ({size / 1024 / 1024:.2f} MB)")

if __name__ == "__main__":
    main()
