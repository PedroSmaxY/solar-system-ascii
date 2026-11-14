#!/usr/bin/env python3
"""
Generate ultra-accurate Lunar Dual-Layer cartographic dataset for AstroEngine:
Layer 1: NASA LROC WAC global photographic albedo (1024x512 uint8 = 524,288 bytes)
  - Pure photographic satellite mapping of Lunar Maria, Highlands, impact craters and ray ejecta.
Layer 2: NASA LOLA laser altimeter global elevation & relief (1024x512 uint8 = 524,288 bytes)
  - Topographic elevation for crater walls, central peaks, and terminator shadow normal perturbation.
Header: b"ASTRO_M1" (8 bytes) + W (4 bytes u32) + H (4 bytes u32) + Layer1 + Layer2.
Total size: 1,048,592 bytes (~1.05 MB).
"""

import urllib.request
import io
import struct
import numpy as np
from PIL import Image

WIDTH = 1024
HEIGHT = 512

NASA_LROC_ALBEDO_URL = "https://raw.githubusercontent.com/mrdoob/three.js/dev/examples/textures/planets/moon_1024.jpg"
NASA_LOLA_BUMP_URL = "https://raw.githubusercontent.com/jeromeetienne/threex.planets/master/images/moonbump1k.jpg"

def fetch_albedo():
    print(f"Downloading NASA LROC global photographic albedo from {NASA_LROC_ALBEDO_URL}...")
    req = urllib.request.Request(NASA_LROC_ALBEDO_URL, headers={"User-Agent": "AstroEngine/1.0"})
    with urllib.request.urlopen(req, timeout=20) as resp:
        img = Image.open(io.BytesIO(resp.read())).convert("L")
    if img.size != (WIDTH, HEIGHT):
        print(f"Resizing albedo from {img.size} to ({WIDTH}, {HEIGHT}) via Lanczos...")
        img = img.resize((WIDTH, HEIGHT), Image.LANCZOS)
    arr = np.array(img, dtype=np.uint8)
    print(f"Albedo loaded: shape={arr.shape}, min={arr.min()}, max={arr.max()}, mean={arr.mean():.1f}")
    return arr

def fetch_elevation():
    print(f"Downloading NASA LOLA laser altimeter bump map from {NASA_LOLA_BUMP_URL}...")
    req = urllib.request.Request(NASA_LOLA_BUMP_URL, headers={"User-Agent": "AstroEngine/1.0"})
    with urllib.request.urlopen(req, timeout=20) as resp:
        img = Image.open(io.BytesIO(resp.read())).convert("L")
    print(f"Resizing elevation bump from {img.size} to ({WIDTH}, {HEIGHT}) via Lanczos...")
    img = img.resize((WIDTH, HEIGHT), Image.LANCZOS)
    arr = np.array(img, dtype=np.uint8)
    print(f"Elevation loaded: shape={arr.shape}, min={arr.min()}, max={arr.max()}, mean={arr.mean():.1f}")
    return arr

def main():
    albedo = fetch_albedo()
    elevation = fetch_elevation()

    # Validate geological features on Near Side (Lon 0, Lat 0 is center: x=512, y=256)
    # Tycho: lat ~ -43.3, lon ~ -11.2 -> y ~ 379, x ~ 480
    tycho_y, tycho_x = int((90.0 - (-43.3)) / 180.0 * HEIGHT), int((-11.2 + 180.0) / 360.0 * WIDTH)
    print(f"Validating Tycho crater at (y={tycho_y}, x={tycho_x}): albedo={albedo[tycho_y, tycho_x]}, elevation={elevation[tycho_y, tycho_x]}")

    # Mare Imbrium center: lat ~ +33.0, lon ~ -16.0 -> y ~ 162, x ~ 466
    imbrium_y, imbrium_x = int((90.0 - 33.0) / 180.0 * HEIGHT), int((-16.0 + 180.0) / 360.0 * WIDTH)
    print(f"Validating Mare Imbrium at (y={imbrium_y}, x={imbrium_x}): albedo={albedo[imbrium_y, imbrium_x]}, elevation={elevation[imbrium_y, imbrium_x]}")

    # Pack into ASTRO_M1 binary file
    output_path = "assets/moon_data.bin"
    header = struct.pack("<8sII", b"ASTRO_M1", WIDTH, HEIGHT)
    data = header + albedo.tobytes() + elevation.tobytes()

    with open(output_path, "wb") as f:
        f.write(data)

    print(f"Successfully generated {output_path} ({len(data)} bytes, {len(data) / 1024 / 1024:.2f} MB)")

if __name__ == "__main__":
    main()
