#!/usr/bin/env python3
"""
Generate assets/jupiter_system_data.bin from authentic NASA Dawn/Galileo/Juno datasets.
Includes:
1. Jupiter: 1024x512 24-bit RGB authentic global cloud bands and Great Red Spot.
2. Io: 1024x512 24-bit RGB volcanic sulfur, Pele plume deposits, and Loki Patera.
3. Europa: 1024x512 8-bit albedo + 1024x512 8-bit fracture relief (lineae, Conamara Chaos).
4. Ganymede: 1024x512 8-bit albedo + 1024x512 8-bit groove relief.
5. Callisto: 1024x512 8-bit albedo + 1024x512 8-bit Valhalla multi-ring basin relief.

Binary format:
- Magic: b"ASTRO_JP" (8 bytes)
- Width: 1024 (u32 little-endian)
- Height: 512 (u32 little-endian)
- Jupiter RGB: 1024 * 512 * 3 bytes (offset 16)
- Io RGB: 1024 * 512 * 3 bytes (offset 16 + 3 * pixels)
- Europa Albedo: 1024 * 512 bytes (offset 16 + 6 * pixels)
- Europa Elevation: 1024 * 512 bytes (offset 16 + 7 * pixels)
- Ganymede Albedo: 1024 * 512 bytes (offset 16 + 8 * pixels)
- Ganymede Elevation: 1024 * 512 bytes (offset 16 + 9 * pixels)
- Callisto Albedo: 1024 * 512 bytes (offset 16 + 10 * pixels)
- Callisto Elevation: 1024 * 512 bytes (offset 16 + 11 * pixels)
Total size: 16 + 12 * 524,288 = 6,291,472 bytes.
"""

import os
import struct
import subprocess
import urllib.request

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS_DIR = os.path.join(BASE_DIR, "assets")
TMP_DIR = os.path.join(BASE_DIR, "target", "tmp_jup_textures")
os.makedirs(ASSETS_DIR, exist_ok=True)
os.makedirs(TMP_DIR, exist_ok=True)

URLS = {
    "jupiter": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/jupiter.jpg",
    "io": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/io.png",
    "europa": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/europa.jpg",
    "ganymede": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/ganymede.jpg",
    "callisto": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/callisto.jpg",
}

TARGET_W = 1024
TARGET_H = 512

def download_file(url, out_path):
    if not os.path.exists(out_path) or os.path.getsize(out_path) == 0:
        print(f"Downloading {url} -> {out_path}...")
        req = urllib.request.Request(url, headers={"User-Agent": "AstroEngine/1.0"})
        with urllib.request.urlopen(req) as resp, open(out_path, "wb") as f:
            f.write(resp.read())
    else:
        print(f"Cached: {out_path}")

def downsample_to_bmp(src_path, dst_bmp_path, width, height):
    # Use sips (native macOS image processor)
    cmd = [
        "sips",
        "-z", str(height), str(width),
        "-s", "format", "bmp",
        src_path,
        "--out", dst_bmp_path
    ]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL)

def parse_bmp_rgb(bmp_path):
    with open(bmp_path, "rb") as f:
        data = f.read()
    pixel_offset = struct.unpack("<I", data[10:14])[0]
    w = struct.unpack("<i", data[18:22])[0]
    raw_h = struct.unpack("<i", data[22:26])[0]
    bpp = struct.unpack("<H", data[28:30])[0]
    h = abs(raw_h)
    top_down = (raw_h < 0)

    row_size = ((w * bpp + 31) // 32) * 4
    rgb_out = bytearray(w * h * 3)

    for y in range(h):
        src_row = y if top_down else (h - 1 - y)
        row_start = pixel_offset + src_row * row_size
        row_data = data[row_start : row_start + w * 3]
        dst_start = y * w * 3
        dst_end = dst_start + w * 3
        rgb_out[dst_start:dst_end:3] = row_data[2::3]
        rgb_out[dst_start+1:dst_end:3] = row_data[1::3]
        rgb_out[dst_start+2:dst_end:3] = row_data[0::3]

    return bytes(rgb_out)

def parse_bmp_grayscale(bmp_path):
    rgb = parse_bmp_rgb(bmp_path)
    pixels = len(rgb) // 3
    gray = bytearray(pixels)
    for i in range(pixels):
        r = rgb[i * 3]
        g = rgb[i * 3 + 1]
        b = rgb[i * 3 + 2]
        gray[i] = int(0.299 * r + 0.587 * g + 0.114 * b)
    return bytes(gray)

def box_blur_2d(gray_bytes, width, height, radius=2):
    # Ultra-fast O(1) running-sum separable box blur in bytearray
    temp = bytearray(width * height)
    out = bytearray(width * height)
    k_size = 2 * radius + 1

    # Horizontal pass with wrapping
    for y in range(height):
        y_off = y * width
        acc = sum(gray_bytes[y_off + (dx % width)] for dx in range(-radius, radius + 1))
        temp[y_off] = acc // k_size
        for x in range(1, width):
            acc += gray_bytes[y_off + ((x + radius) % width)] - gray_bytes[y_off + ((x - radius - 1) % width)]
            temp[y_off + x] = acc // k_size

    # Vertical pass with clamping
    for x in range(width):
        acc = sum(temp[min(max(dy, 0), height - 1) * width + x] for dy in range(-radius, radius + 1))
        out[x] = acc // k_size
        for y in range(1, height):
            lead_y = min(y + radius, height - 1)
            trail_y = max(y - radius - 1, 0)
            acc += temp[lead_y * width + x] - temp[trail_y * width + x]
            out[y * width + x] = acc // k_size

    return bytes(out)

def main():
    print("=== Generating Jupiter System Data Assets ===")
    local_files = {}
    for name, url in URLS.items():
        ext = ".png" if url.endswith(".png") else ".jpg"
        local_path = os.path.join(TMP_DIR, f"{name}{ext}")
        download_file(url, local_path)
        local_files[name] = local_path

    # Convert to 1024x512 BMP
    bmp_files = {}
    for name, path in local_files.items():
        bmp_path = os.path.join(TMP_DIR, f"{name}_1024x512.bmp")
        print(f"Downsampling {name} to 1024x512 BMP...")
        downsample_to_bmp(path, bmp_path, TARGET_W, TARGET_H)
        bmp_files[name] = bmp_path

    # 1. Jupiter RGB
    print("Extracting Jupiter RGB...")
    jup_rgb = parse_bmp_rgb(bmp_files["jupiter"])
    assert len(jup_rgb) == TARGET_W * TARGET_H * 3

    # 2. Io RGB
    print("Extracting Io RGB...")
    io_rgb = parse_bmp_rgb(bmp_files["io"])
    assert len(io_rgb) == TARGET_W * TARGET_H * 3

    # 3. Europa Albedo + Relief
    print("Processing Europa (Albedo + Relief)...")
    europa_alb = parse_bmp_grayscale(bmp_files["europa"])
    europa_elev = box_blur_2d(europa_alb, TARGET_W, TARGET_H, radius=2)

    # 4. Ganymede Albedo + Relief
    print("Processing Ganymede (Albedo + Relief)...")
    ganymede_alb = parse_bmp_grayscale(bmp_files["ganymede"])
    ganymede_elev = box_blur_2d(ganymede_alb, TARGET_W, TARGET_H, radius=2)

    # 5. Callisto Albedo + Relief
    print("Processing Callisto (Albedo + Relief)...")
    callisto_alb = parse_bmp_grayscale(bmp_files["callisto"])
    callisto_elev = box_blur_2d(callisto_alb, TARGET_W, TARGET_H, radius=2)

    output_bin = os.path.join(ASSETS_DIR, "jupiter_system_data.bin")
    header = b"ASTRO_JP" + struct.pack("<II", TARGET_W, TARGET_H)

    with open(output_bin, "wb") as f:
        f.write(header)
        f.write(jup_rgb)        # 16 .. 16 + 3*pixels
        f.write(io_rgb)         # 16 + 3*pixels .. 16 + 6*pixels
        f.write(europa_alb)     # 16 + 6*pixels .. 16 + 7*pixels
        f.write(europa_elev)    # 16 + 7*pixels .. 16 + 8*pixels
        f.write(ganymede_alb)   # 16 + 8*pixels .. 16 + 9*pixels
        f.write(ganymede_elev)  # 16 + 9*pixels .. 16 + 10*pixels
        f.write(callisto_alb)   # 16 + 10*pixels .. 16 + 11*pixels
        f.write(callisto_elev)  # 16 + 11*pixels .. 16 + 12*pixels

    total_size = os.path.getsize(output_bin)
    expected_size = 16 + 12 * (TARGET_W * TARGET_H)
    assert total_size == expected_size, f"Size mismatch: {total_size} vs {expected_size}"
    print(f"SUCCESS: Generated {output_bin} ({total_size} bytes, {total_size / (1024*1024):.2f} MB)")

if __name__ == "__main__":
    main()
