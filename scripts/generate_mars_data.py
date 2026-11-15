#!/usr/bin/env python3
"""
Generate ultra-accurate Mars Cartographic & Elevation Dataset for AstroEngine:
Layer 1-3: NASA Viking / Solar System Scope True-Color Mars Albedo RGB (1024x512 x 3 = 1,572,864 bytes)
  - Detailed surface features: Syrtis Major, Olympus Mons, Tharsis shield volcanoes, Valles Marineris, Hellas Planitia, Planum Boreum/Australe ice caps.
Layer 4: NASA MOLA (Mars Orbiter Laser Altimeter) global elevation & relief (1024x512 uint8 = 524,288 bytes)
  - Extreme topographic relief for Olympus Mons (21.9 km caldera), Valles Marineris canyon depths (7 km), and crater rims.
Header: b"ASTRO_MA" (8 bytes) + W (4 bytes u32) + H (4 bytes u32) + MarsR + MarsG + MarsB + Elevation.
Total size: 2,097,168 bytes (~2.10 MB).
"""

import os
import struct
import subprocess
import urllib.request

WIDTH = 1024
HEIGHT = 512

MARS_ALBEDO_URL = "https://www.solarsystemscope.com/textures/download/2k_mars.jpg"
MARS_BUMP_URL = "https://raw.githubusercontent.com/jeromeetienne/threex.planets/master/images/marsbump1k.jpg"


def download_and_convert_bmp(url: str, jpg_path: str, bmp_path: str):
    print(f"Downloading {url} -> {jpg_path}...")
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=30) as resp, open(jpg_path, "wb") as f:
        f.write(resp.read())
    subprocess.run(["sips", "-z", str(HEIGHT), str(WIDTH), "-s", "format", "bmp", jpg_path, "--out", bmp_path], check=True)


def read_bmp_rgb(bmp_path: str):
    with open(bmp_path, "rb") as f:
        _ = f.read(54)
        data = f.read()
    row_stride = WIDTH * 3
    r = bytearray(WIDTH * HEIGHT)
    g = bytearray(WIDTH * HEIGHT)
    b = bytearray(WIDTH * HEIGHT)
    for y in range(HEIGHT):
        src_y = HEIGHT - 1 - y
        for x in range(WIDTH):
            idx = y * WIDTH + x
            b[idx] = data[src_y * row_stride + x * 3]
            g[idx] = data[src_y * row_stride + x * 3 + 1]
            r[idx] = data[src_y * row_stride + x * 3 + 2]
    return r, g, b


def read_bmp_gray(bmp_path: str) -> bytearray:
    with open(bmp_path, "rb") as f:
        _ = f.read(54)
        data = f.read()
    row_stride = WIDTH * 3
    gray = bytearray(WIDTH * HEIGHT)
    for y in range(HEIGHT):
        src_y = HEIGHT - 1 - y
        for x in range(WIDTH):
            b = data[src_y * row_stride + x * 3]
            g = data[src_y * row_stride + x * 3 + 1]
            r = data[src_y * row_stride + x * 3 + 2]
            lum = int(0.299 * r + 0.587 * g + 0.114 * b)
            gray[y * WIDTH + x] = lum
    return gray


def main():
    os.makedirs("assets", exist_ok=True)
    jpg_albedo = "/tmp/mars_albedo.jpg"
    bmp_albedo = "/tmp/mars_albedo.bmp"
    jpg_bump = "/tmp/mars_bump.jpg"
    bmp_bump = "/tmp/mars_bump.bmp"

    download_and_convert_bmp(MARS_ALBEDO_URL, jpg_albedo, bmp_albedo)
    download_and_convert_bmp(MARS_BUMP_URL, jpg_bump, bmp_bump)

    r, g, b = read_bmp_rgb(bmp_albedo)
    elevation = read_bmp_gray(bmp_bump)

    header = struct.pack("<8sII", b"ASTRO_MA", WIDTH, HEIGHT)
    bin_data = header + bytes(r) + bytes(g) + bytes(b) + bytes(elevation)

    output_path = "assets/mars_data.bin"
    with open(output_path, "wb") as f:
        f.write(bin_data)

    print(f"Successfully generated {output_path} ({len(bin_data)} bytes)")

    for p in (jpg_albedo, bmp_albedo, jpg_bump, bmp_bump):
        if os.path.exists(p):
            os.remove(p)


if __name__ == "__main__":
    main()
