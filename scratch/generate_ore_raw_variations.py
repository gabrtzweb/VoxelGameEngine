import os
import random
from PIL import Image

ORES_DIR = r"docs/worldbuilding/blocks/ores"

# All 26 unrefined solid blocks
RAW_ORES = [
    "ore_aluminum_raw.png",
    "ore_cinnabar_raw.png",
    "ore_coal_raw.png",
    "ore_copper_raw.png",
    "ore_gold_raw.png",
    "ore_iron_raw.png",
    "ore_lazurite_raw.png",
    "ore_lead_raw.png",
    "ore_manganese_raw.png",
    "ore_nickel_raw.png",
    "ore_platinum_raw.png",
    "ore_quartz_raw.png",
    "ore_saltpeter_raw.png",
    "ore_silver_raw.png",
    "ore_sulfur_raw.png",
    "ore_tin_raw.png",
    "ore_uranium_raw.png",
    "ore_zinc_raw.png",
]

ROUGH_GEMS = [
    "gem_diamond_rough.png",
    "gem_emerald_rough.png",
    "gem_jade_rough.png",
    "gem_onyx_rough.png",
    "gem_opal_rough.png",
    "gem_ruby_rough.png",
    "gem_sapphire_rough.png",
    "gem_topaz_rough.png",
]

def get_sorted_palette(img_frame_rgb):
    """Returns unique RGB colors sorted by luminance."""
    pixels = [img_frame_rgb.getpixel((x, y)) for y in range(img_frame_rgb.height) for x in range(img_frame_rgb.width)]
    unique = list(set(pixels))
    unique.sort(key=lambda c: 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2])
    return unique

def generate_cluster_mask(width, height, target_ratio, rng):
    """Generates an organic cluster mask where ~target_ratio of pixels are selected in 2-4 small contiguous patches."""
    mask = [[False for _ in range(width)] for _ in range(height)]
    num_clusters = rng.randint(2, 4)
    target_count = int(width * height * target_ratio)
    
    seeds = [(rng.randint(0, width - 1), rng.randint(0, height - 1)) for _ in range(num_clusters)]
    queue = list(seeds)
    count = 0
    for sx, sy in seeds:
        if not mask[sy][sx]:
            mask[sy][sx] = True
            count += 1
        
    while queue and count < target_count:
        cx, cy = queue.pop(0)
        neighbors = [
            ((cx + 1) % width, cy),
            ((cx - 1) % width, cy),
            (cx, (cy + 1) % height),
            (cx, (cy - 1) % height),
        ]
        rng.shuffle(neighbors)
        for nx, ny in neighbors:
            if not mask[ny][nx] and count < target_count:
                if rng.random() < 0.65:
                    mask[ny][nx] = True
                    count += 1
                    queue.append((nx, ny))
    return mask

def vary_single_frame(frame, seed_offset, variant_idx):
    """Produces variant 1 or 2 of a 16x16 frame, shifting palette by 1 step in organic clusters."""
    orig_mode = frame.mode
    frame_rgb = frame.convert("RGB")
    palette = get_sorted_palette(frame_rgb)
    n_pal = len(palette)
    
    if n_pal <= 1:
        return frame.copy()
        
    color_to_idx = {c: i for i, c in enumerate(palette)}
    
    rng = random.Random(seed_offset + variant_idx * 1337)
    target_ratio = 0.19 if variant_idx == 1 else 0.24
    mask = generate_cluster_mask(16, 16, target_ratio, rng)
    
    direction_map = [[0 for _ in range(16)] for _ in range(16)]
    for y in range(16):
        for x in range(16):
            if mask[y][x]:
                if variant_idx == 1:
                    direction_map[y][x] = 1 if (x + y) % 2 == 0 else -1
                else:
                    direction_map[y][x] = -1 if (x * 2 + y) % 3 == 0 else 1

    out_rgb = frame_rgb.copy()
    for y in range(16):
        for x in range(16):
            if mask[y][x]:
                old_c = frame_rgb.getpixel((x, y))
                old_idx = color_to_idx[old_c]
                d = direction_map[y][x]
                new_idx = old_idx + d
                if new_idx < 0:
                    new_idx = 1 if n_pal > 1 else 0
                elif new_idx >= n_pal:
                    new_idx = n_pal - 2 if n_pal > 1 else n_pal - 1
                out_rgb.putpixel((x, y), palette[new_idx])
                
    if orig_mode == "RGBA":
        out = Image.new("RGBA", (16, 16))
        for y in range(16):
            for x in range(16):
                r, g, b = out_rgb.getpixel((x, y))
                orig_p = frame.getpixel((x, y))
                a = orig_p[3] if len(orig_p) > 3 else 255
                out.putpixel((x, y), (r, g, b, a))
        return out
    elif orig_mode == "P":
        return out_rgb.quantize(palette=frame)
    else:
        return out_rgb

def process_file(filename):
    filepath = os.path.join(ORES_DIR, filename)
    base_name, ext = os.path.splitext(filename)
    
    im = Image.open(filepath)
    width, height = im.size
    orig_mode = im.mode
    is_animated = (height > 16)
    num_frames = height // 16 if is_animated else 1
    
    name_hash = abs(hash(base_name)) % 100000
    
    for variant in [1, 2]:
        var_filename = f"{base_name}{variant}{ext}"
        var_filepath = os.path.join(ORES_DIR, var_filename)
        
        if not is_animated:
            var_im = vary_single_frame(im, name_hash, variant)
        else:
            var_im = Image.new(orig_mode if orig_mode != "P" else "RGB", (width, height))
            for f_idx in range(num_frames):
                frame = im.crop((0, f_idx * 16, 16, (f_idx + 1) * 16))
                var_frame = vary_single_frame(frame, name_hash, variant)
                var_im.paste(var_frame, (0, f_idx * 16))
            if orig_mode == "P":
                var_im = var_im.quantize(palette=im)
                
        var_im.save(var_filepath)
        print(f"Generated: {var_filename} (size={var_im.size}, mode={var_im.mode})")

def main():
    print(f"Generating variations 1 and 2 for {len(RAW_ORES)} raw ores and {len(ROUGH_GEMS)} rough gems...")
    for f in RAW_ORES + ROUGH_GEMS:
        process_file(f)
    print("Done generating all 52 solid raw and rough textures!")

if __name__ == "__main__":
    main()
