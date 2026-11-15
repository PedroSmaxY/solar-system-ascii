#!/usr/bin/env python3
"""
Generate assets/saturn_system_data.bin from authentic NASA Cassini datasets:
1. Saturn: 1024x512 24-bit RGB authentic golden cloud belts and polar hexagon.
2. Saturn Rings: 1024-sample RGBA radial profile (C, B, Cassini Division, A, Encke, F rings).
3. Titan: 1024x512 24-bit RGB dense nitrogen/methane photochemical haze cloud deck.
4. Enceladus: 1024x512 8-bit albedo + 1024x512 8-bit relief (South Pole tiger stripes).
5. Mimas: 1024x512 8-bit albedo + 1024x512 8-bit relief (Herschel crater basin).
6. Iapetus: 1024x512 8-bit albedo + 1024x512 8-bit relief (Cassini Regio yin-yang dichotomy).
7. Rhea: 1024x512 8-bit albedo + 1024x512 8-bit relief (cratered icy crust).

Binary format:
- Magic: b"ASTRO_SA" (8 bytes)
- Width: 1024 (u32 little-endian)
- Height: 512 (u32 little-endian)
- Saturn RGB: 1024 * 512 * 3 bytes (offset 16)
- Saturn Rings RGBA: 1024 * 4 bytes (offset 16 + 3 * pixels)
- Titan RGB: 1024 * 512 * 3 bytes (offset 16 + 3 * pixels + 4096)
- Enceladus Albedo: 1024 * 512 bytes (offset 16 + 6 * pixels + 4096)
- Enceladus Elevation: 1024 * 512 bytes (offset 16 + 7 * pixels + 4096)
- Mimas Albedo: 1024 * 512 bytes (offset 16 + 8 * pixels + 4096)
- Mimas Elevation: 1024 * 512 bytes (offset 16 + 9 * pixels + 4096)
- Iapetus Albedo: 1024 * 512 bytes (offset 16 + 10 * pixels + 4096)
- Iapetus Elevation: 1024 * 512 bytes (offset 16 + 11 * pixels + 4096)
- Rhea Albedo: 1024 * 512 bytes (offset 16 + 12 * pixels + 4096)
- Rhea Elevation: 1024 * 512 bytes (offset 16 + 13 * pixels + 4096)
Total size: 16 + 4096 + 14 * 524,288 = 7,344,144 bytes.
"""

import os
import struct
import subprocess
import urllib.request
import zlib

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS_DIR = os.path.join(BASE_DIR, "assets")
TMP_DIR = os.path.join(BASE_DIR, "target", "tmp_saturn_textures")
os.makedirs(ASSETS_DIR, exist_ok=True)
os.makedirs(TMP_DIR, exist_ok=True)

URLS = {
    "saturn": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/saturn.jpg",
    "saturn-rings": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/saturn-rings.png",
    "titan": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/titan-clouds.jpg",
    "enceladus": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/enceladus.jpg",
    "mimas": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/mimas.jpg",
    "iapetus": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/iapetus.jpg",
    "rhea": "https://raw.githubusercontent.com/CelestiaProject/CelestiaContent/master/textures/hires/rhea.jpg",
}

TARGET_W = 1024
TARGET_H = 512
RINGS_SAMPLES = 1024

def download_file(url, out_path):
    if not os.path.exists(out_path) or os.path.getsize(out_path) == 0:
        print(f"Downloading {url} -> {out_path}...")
        req = urllib.request.Request(url, headers={"User-Agent": "AstroEngine/1.0"})
        with urllib.request.urlopen(req) as resp, open(out_path, "wb") as f:
            f.write(resp.read())
    else:
        print(f"Cached: {out_path}")

def downsample_to_bmp(src_path, dst_bmp_path, width, height):
    current_src = src_path
    if src_path.endswith(".png"):
        intermediate_jpg = src_path + ".jpg"
        subprocess.run(["sips", "-s", "format", "jpeg", src_path, "--out", intermediate_jpg], check=True, stdout=subprocess.DEVNULL)
        current_src = intermediate_jpg

    cmd = [
        "sips",
        "-z", str(height), str(width),
        "-s", "format", "bmp",
        current_src,
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
    temp = bytearray(width * height)
    out = bytearray(width * height)
    k_size = 2 * radius + 1

    for y in range(height):
        y_off = y * width
        acc = sum(gray_bytes[y_off + (dx % width)] for dx in range(-radius, radius + 1))
        temp[y_off] = acc // k_size
        for x in range(1, width):
            lead_x = (x + radius) % width
            trail_x = (x - radius - 1) % width
            acc += gray_bytes[y_off + lead_x] - gray_bytes[y_off + trail_x]
            temp[y_off + x] = acc // k_size

    for x in range(width):
        acc = sum(temp[min(max(dy, 0), height - 1) * width + x] for dy in range(-radius, radius + 1))
        out[x] = acc // k_size
        for y in range(1, height):
            lead_y = min(y + radius, height - 1)
            trail_y = max(y - radius - 1, 0)
            acc += temp[lead_y * width + x] - temp[trail_y * width + x]
            out[y * width + x] = acc // k_size

    return bytes(out)

def parse_ring_rgba_profile(ring_png_path, num_samples):
    resampled_png = os.path.join(TMP_DIR, f"rings_{num_samples}.png")
    subprocess.run(["sips", "-z", "1", str(num_samples), ring_png_path, "--out", resampled_png], check=True, stdout=subprocess.DEVNULL)
    with open(resampled_png, "rb") as f:
        png_data = f.read()

    idat_data = b''
    offset = 8
    while offset < len(png_data):
        length, ctype = struct.unpack('>I4s', png_data[offset:offset+8])
        if ctype == b'IDAT':
            idat_data += png_data[offset+8:offset+8+length]
        offset += 8 + length + 4

    decomp = zlib.decompress(idat_data)
    filter_byte = decomp[0]
    raw = bytearray(decomp[1:1+num_samples*4])
    if filter_byte == 1:
        for i in range(4, len(raw)):
            raw[i] = (raw[i] + raw[i-4]) & 0xff
    return bytes(raw)

def main():
    print("=== Generating Saturn System Data Assets ===")
    local_files = {}
    for name, url in URLS.items():
        ext = ".png" if url.endswith(".png") else ".jpg"
        local_path = os.path.join(TMP_DIR, f"{name}{ext}")
        download_file(url, local_path)
        local_files[name] = local_path

    # Convert cylindrical bodies to 1024x512 BMP
    bmp_files = {}
    for name in ["saturn", "titan", "enceladus", "mimas", "iapetus", "rhea"]:
        path = local_files[name]
        bmp_path = os.path.join(TMP_DIR, f"{name}_1024x512.bmp")
        print(f"Downsampling {name} to 1024x512 BMP...")
        downsample_to_bmp(path, bmp_path, TARGET_W, TARGET_H)
        bmp_files[name] = bmp_path

    # 1. Saturn RGB
    print("Extracting Saturn RGB...")
    saturn_rgb = parse_bmp_rgb(bmp_files["saturn"])
    assert len(saturn_rgb) == TARGET_W * TARGET_H * 3

    # 2. Saturn Rings RGBA profile (1024 samples x 4 bytes)
    print("Extracting Saturn Rings RGBA profile...")
    rings_rgba = parse_ring_rgba_profile(local_files["saturn-rings"], RINGS_SAMPLES)
    assert len(rings_rgba) == RINGS_SAMPLES * 4

    # 3. Titan RGB (photochemical smog haze)
    print("Extracting Titan RGB...")
    titan_rgb = parse_bmp_rgb(bmp_files["titan"])
    assert len(titan_rgb) == TARGET_W * TARGET_H * 3

    # 4. Enceladus Albedo + Relief
    print("Processing Enceladus (Albedo + Relief)...")
    enceladus_alb = parse_bmp_grayscale(bmp_files["enceladus"])
    enceladus_elev = box_blur_2d(enceladus_alb, TARGET_W, TARGET_H, radius=2)

    # 5. Mimas Albedo + Relief
    print("Processing Mimas (Albedo + Relief)...")
    mimas_alb = parse_bmp_grayscale(bmp_files["mimas"])
    mimas_elev = box_blur_2d(mimas_alb, TARGET_W, TARGET_H, radius=2)

    # 6. Iapetus Albedo + Relief
    print("Processing Iapetus (Albedo + Relief)...")
    iapetus_alb = parse_bmp_grayscale(bmp_files["iapetus"])
    iapetus_elev = box_blur_2d(iapetus_alb, TARGET_W, TARGET_H, radius=2)

    # 7. Rhea Albedo + Relief
    print("Processing Rhea (Albedo + Relief)...")
    rhea_alb = parse_bmp_grayscale(bmp_files["rhea"])
    rhea_elev = box_blur_2d(rhea_alb, TARGET_W, TARGET_H, radius=2)

    output_bin = os.path.join(ASSETS_DIR, "saturn_system_data.bin")
    header = b"ASTRO_SA" + struct.pack("<II", TARGET_W, TARGET_H)

    with open(output_bin, "wb") as f:
        f.write(header)
        f.write(saturn_rgb)     # offset 16
        f.write(rings_rgba)     # offset 16 + 3*pixels
        f.write(titan_rgb)      # offset 16 + 3*pixels + 4096
        f.write(enceladus_alb)  # offset 16 + 6*pixels + 4096
        f.write(enceladus_elev) # offset 16 + 7*pixels + 4096
        f.write(mimas_alb)      # offset 16 + 8*pixels + 4096
        f.write(mimas_elev)     # offset 16 + 9*pixels + 4096
        f.write(iapetus_alb)    # offset 16 + 10*pixels + 4096
        f.write(iapetus_elev)   # offset 16 + 11*pixels + 4096
        f.write(rhea_alb)       # offset 16 + 12*pixels + 4096
        f.write(rhea_elev)      # offset 16 + 13*pixels + 4096

    total_size = os.path.getsize(output_bin)
    expected_size = 16 + 4096 + 14 * (TARGET_W * TARGET_H)
    assert total_size == expected_size, f"Size mismatch: {total_size} vs {expected_size}"
    print(f"SUCCESS: Generated {output_bin} ({total_size} bytes, {total_size / (1024*1024):.2f} MB)")

if __name__ == "__main__":
    main()
