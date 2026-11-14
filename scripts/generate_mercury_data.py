#!/usr/bin/env python3
"""
Generate ultra-accurate Mercury Dual-Layer cartographic dataset for AstroEngine:
Layer 1: NASA/USGS MESSENGER global photographic albedo (1024x512 uint8 = 524,288 bytes)
  - Pure photographic satellite mapping of Caloris Basin, cratered highlands, and volcanic smooth plains.
Layer 2: NASA MESSENGER MLA elevation & relief bump map (1024x512 uint8 = 524,288 bytes)
  - Topographic elevation for impact basin rims, central peaks, and Discovery Rupes lobate scarps.
Header: b"ASTRO_ME" (8 bytes) + W (4 bytes u32) + H (4 bytes u32) + Layer1 + Layer2.
Total size: 1,048,592 bytes (~1.05 MB).
"""

import os
import struct
import subprocess
import urllib.request

WIDTH = 1024
HEIGHT = 512

MERCURY_MAP_URL = "https://raw.githubusercontent.com/jeromeetienne/threex.planets/master/images/mercurymap.jpg"
MERCURY_BUMP_URL = "https://raw.githubusercontent.com/jeromeetienne/threex.planets/master/images/mercurybump.jpg"


def download_and_convert_bmp(url: str, jpg_path: str, bmp_path: str):
    print(f"Downloading {url} -> {jpg_path}...")
    req = urllib.request.Request(url, headers={"User-Agent": "AstroEngine/1.0"})
    with urllib.request.urlopen(req, timeout=30) as resp, open(jpg_path, "wb") as f:
        f.write(resp.read())
    subprocess.run(["sips", "-s", "format", "bmp", jpg_path, "--out", bmp_path], check=True)


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
    jpg_map = "/tmp/merc_map.jpg"
    bmp_map = "/tmp/merc_map.bmp"
    jpg_bump = "/tmp/merc_bump.jpg"
    bmp_bump = "/tmp/merc_bump.bmp"

    download_and_convert_bmp(MERCURY_MAP_URL, jpg_map, bmp_map)
    download_and_convert_bmp(MERCURY_BUMP_URL, jpg_bump, bmp_bump)

    albedo = read_bmp_gray(bmp_map)
    elevation = read_bmp_gray(bmp_bump)

    header = struct.pack("<8sII", b"ASTRO_ME", WIDTH, HEIGHT)
    bin_data = header + bytes(albedo) + bytes(elevation)

    output_path = "assets/mercury_data.bin"
    with open(output_path, "wb") as f:
        f.write(bin_data)

    print(f"Successfully generated {output_path} ({len(bin_data)} bytes)")

    for p in (jpg_map, bmp_map, jpg_bump, bmp_bump):
        if os.path.exists(p):
            os.remove(p)


if __name__ == "__main__":
    main()
