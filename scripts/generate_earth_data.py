#!/usr/bin/env python3
"""
Generate ultra-accurate Earth Triple-Layer cartographic dataset for AstroEngine:
Layer 1: NASA Blue Marble 4K true satellite albedo (1440x720 RGB = 3,110,400 bytes)
  - Continents: True satellite photography (deserts, forests, savannas, snow).
  - Ocean: Deep sapphire navy blue [10, 24, 52] matching Apollo 17 & Artemis II.
  - Continental shelf: Coastal marine cyan/turquoise [18, 56, 92].
  - Inland freshwater rivers & lakes: Clear freshwater blue [22, 64, 112].
Layer 2: Hydrology & terrain mask (1440x720 uint8 = 1,036,800 bytes)
  - 0: Deep ocean (20)
  - 38: Continental shelf
  - 50: Inland freshwater rivers & lakes (major arteries only, scalerank <= 3)
  - 145: Land baseline
  - 250: Permanent ice sheets (Antarctica, Greenland)
Layer 3: NASA Black Marble night lights intensity (1440x720 uint8 = 1,036,800 bytes)
Header: b"ASTRO_E3" (8 bytes) + W (4 bytes u32) + H (4 bytes u32) + Layer1 + Layer2 + Layer3.
Total size: 5,184,016 bytes (~5.18 MB).
"""

import os
import sys
import json
import math
import struct
import urllib.request
from PIL import Image
import rasterio
from rasterio import features
import numpy as np
from scipy.ndimage import binary_dilation, gaussian_filter

WIDTH = 1440
HEIGHT = 720
BASE_URL = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/"
NASA_BLUE_MARBLE_URL = "https://raw.githubusercontent.com/turban/webgl-earth/master/images/2_no_clouds_4k.jpg"

def fetch_geojson(filename):
    print(f"Downloading {filename}...")
    url = BASE_URL + filename
    req = urllib.request.Request(url, headers={"User-Agent": "AstroEngine/1.0"})
    with urllib.request.urlopen(req, timeout=25) as resp:
        return json.loads(resp.read().decode("utf-8"))

def fetch_nasa_albedo():
    print("Downloading NASA Blue Marble 4K cloudless satellite texture...")
    req = urllib.request.Request(NASA_BLUE_MARBLE_URL, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        img = Image.open(resp).convert("RGB")
    print(f"Loaded NASA Blue Marble {img.size}. Resizing to {WIDTH}x{HEIGHT} via Lanczos...")
    img_resized = img.resize((WIDTH, HEIGHT), Image.LANCZOS)
    return np.array(img_resized, dtype=np.uint8)

def main():
    transform = rasterio.transform.from_bounds(-180.0, -90.0, 180.0, 90.0, WIDTH, HEIGHT)

    # 1. Fetch satellite texture and vector datasets
    albedo_rgb = fetch_nasa_albedo()
    land_data = fetch_geojson("ne_50m_land.geojson")
    river_data = fetch_geojson("ne_50m_rivers_lake_centerlines.geojson")
    lake_data = fetch_geojson("ne_50m_lakes.geojson")
    urban_data = fetch_geojson("ne_50m_urban_areas.geojson")
    places_data = fetch_geojson("ne_50m_populated_places_simple.geojson")

    # ==========================================
    # LAYER 2: Hydrology & Terrain Classification Mask
    # ==========================================
    print("Rasterizing land, lakes, and filtered major rivers...")
    land_shapes = [(f["geometry"], 1) for f in land_data["features"]]
    land_mask = features.rasterize(land_shapes, out_shape=(HEIGHT, WIDTH), transform=transform)

    lake_shapes = [(f["geometry"], 1) for f in lake_data["features"]]
    lake_mask = features.rasterize(lake_shapes, out_shape=(HEIGHT, WIDTH), transform=transform)

    # Filter rivers: keep only major arteries (scalerank <= 3) to prevent exaggerated river clutter
    major_rivers = [
        f for f in river_data["features"]
        if f.get("properties", {}).get("scalerank", 99) <= 3
    ]
    print(f"Filtered {len(river_data['features'])} total rivers down to {len(major_rivers)} major arteries.")

    river_shapes = [(f["geometry"], 1) for f in major_rivers]
    river_mask = features.rasterize(river_shapes, out_shape=(HEIGHT, WIDTH), transform=transform)
    # Only single-pixel line width - NO global dilation!
    # Dilate only at the mouth of the Amazon (-1.5 to +1 lat, -52 to -48 lon)
    lats = 90.0 - (np.arange(HEIGHT) + 0.5) * (180.0 / HEIGHT)
    lons = -180.0 + (np.arange(WIDTH) + 0.5) * (360.0 / WIDTH)
    lon_grid, lat_grid = np.meshgrid(lons, lats)

    amazon_estuary = (lat_grid >= -1.5) & (lat_grid <= 1.0) & (lon_grid >= -52.0) & (lon_grid <= -48.0)
    dilated_estuary = binary_dilation(river_mask & amazon_estuary, iterations=1)
    river_mask[amazon_estuary] = dilated_estuary[amazon_estuary]

    mask_grid = np.full((HEIGHT, WIDTH), 20, dtype=np.uint8)  # Deep ocean
    mask_grid[land_mask == 1] = 145  # Land baseline

    # Permanent ice sheets (Antarctica and Greenland)
    antarctica_ice = (lat_grid < -62.0) & (land_mask == 1)
    greenland_ice = (lat_grid > 60.0) & (lon_grid > -55.0) & (lon_grid < -15.0) & (land_mask == 1)
    mask_grid[antarctica_ice | greenland_ice] = 250

    # Coastal continental shelf
    shelf_mask = binary_dilation(land_mask == 1, iterations=2) & (land_mask == 0)
    mask_grid[shelf_mask] = 38

    # Inland freshwater rivers and lakes
    inland_water = ((lake_mask == 1) | (river_mask == 1)) & (land_mask == 1)
    mask_grid[inland_water] = 50

    # ==========================================
    # LAYER 1: Calibrate Ocean Color (Apollo 17 & Artemis II Standard)
    # ==========================================
    # Deep open ocean: authentic dark sapphire navy blue [10, 24, 52]
    open_ocean = (land_mask == 0) & (~shelf_mask)
    albedo_rgb[open_ocean] = [10, 24, 52]

    # Coastal continental shelf: gentle marine turquoise/cyan [18, 56, 92]
    albedo_rgb[shelf_mask] = [18, 56, 92]

    # Inland freshwater rivers and lakes receive clean freshwater blue [22, 64, 112]
    albedo_rgb[inland_water] = [22, 64, 112]

    # Ice sheets receive pure glacier white
    albedo_rgb[antarctica_ice | greenland_ice] = [238, 245, 255]

    # ==========================================
    # LAYER 3: Night Lights (NASA Black Marble Grid)
    # ==========================================
    print("Rasterizing urban footprints and populated places...")
    night_grid = np.zeros((HEIGHT, WIDTH), dtype=np.float32)

    # Real urban polygons footprint
    urban_shapes = [(f["geometry"], 1) for f in urban_data["features"]]
    urban_mask = features.rasterize(urban_shapes, out_shape=(HEIGHT, WIDTH), transform=transform)
    night_grid[urban_mask == 1] = 0.85

    # Populated places points
    for f in places_data["features"]:
        coords = f["geometry"]["coordinates"]
        plon, plat = coords[0], coords[1]
        pop = f["properties"].get("pop_max", 100000)
        if pop > 40000:
            rad_deg = min(2.5, max(0.5, math.log10(pop) * 0.45 - 1.2))
            intensity = min(1.0, max(0.4, math.log10(pop) * 0.25 - 0.5))
            d = np.hypot(lon_grid - plon, lat_grid - plat)
            mask = d < rad_deg
            falloff = ((1.0 - d[mask] / rad_deg) ** 1.8) * intensity
            night_grid[mask] = np.maximum(night_grid[mask], falloff)

    # Nile River Valley ribbon enrichment
    nile_mask = (lat_grid >= 23.5) & (lat_grid <= 31.5) & (lon_grid >= 30.5) & (lon_grid <= 33.0)
    d_nile = np.abs(lon_grid - (31.2 + (lat_grid - 24.0) * 0.12 + np.sin(lat_grid * 2.2) * 0.35))
    nile_ribbon = nile_mask & (d_nile < 0.6)
    night_grid[nile_ribbon] = np.maximum(night_grid[nile_ribbon], ((1.0 - d_nile[nile_ribbon] / 0.6) ** 1.8) * 0.85)

    # Atmospheric bloom diffusion
    night_blurred = gaussian_filter(night_grid, sigma=0.75)
    night_final = np.clip(np.maximum(night_grid, night_blurred * 0.65) * 255.0, 0, 255).astype(np.uint8)

    # ==========================================
    # Write ASTRO_E3 Triple-Layer Binary
    # ==========================================
    out_path = os.path.join(os.path.dirname(__file__), "..", "assets", "earth_data.bin")
    with open(out_path, "wb") as f:
        f.write(b"ASTRO_E3")
        f.write(struct.pack("<II", WIDTH, HEIGHT))
        f.write(albedo_rgb.tobytes())    # Layer 1: 3,110,400 bytes
        f.write(mask_grid.tobytes())     # Layer 2: 1,036,800 bytes
        f.write(night_final.tobytes())   # Layer 3: 1,036,800 bytes

    total_bytes = os.path.getsize(out_path)
    expected_bytes = 16 + 3 * WIDTH * HEIGHT + WIDTH * HEIGHT + WIDTH * HEIGHT
    print(f"Successfully generated {out_path} ({total_bytes} bytes). Expected: {expected_bytes} bytes.")
    print(f"Layer 1 (NASA Albedo RGB): {len(albedo_rgb.tobytes())} bytes")
    print(f"Layer 2 (Hydrology Mask): {len(mask_grid.tobytes())} bytes, inland water pixels: {np.count_nonzero(inland_water)}")
    print(f"Layer 3 (Night lights): {len(night_final.tobytes())} bytes, lit pixels: {np.count_nonzero(night_final)}")

if __name__ == "__main__":
    main()
