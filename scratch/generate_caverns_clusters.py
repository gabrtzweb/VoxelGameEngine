import os
import random
from PIL import Image

CAVERNS_DIR = r"docs/worldbuilding/blocks/caverns"

CRYSTALS = [
    "amethyst",
    "aquamarine",
    "citrine",
    "fluorite",
    "pink_salt",
    "selenite",
]

# Color themes for crystal glints: (core_white, aura_primary, aura_secondary)
GLINT_THEMES = {
    "amethyst": {
        "white": (255, 255, 255, 255),
        "aura1": (245, 220, 255, 255),
        "aura2": (210, 165, 245, 220),
        "flare": (255, 240, 255, 255),
    },
    "aquamarine": {
        "white": (255, 255, 255, 255),
        "aura1": (200, 255, 255, 255),
        "aura2": (120, 245, 235, 220),
        "flare": (240, 255, 255, 255),
    },
    "citrine": {
        "white": (255, 255, 255, 255),
        "aura1": (255, 250, 200, 255),
        "aura2": (255, 210, 110, 220),
        "flare": (255, 255, 230, 255),
    },
    "fluorite": {
        "white": (255, 255, 255, 255),
        "aura1": (210, 255, 235, 255),
        "aura2": (130, 250, 190, 220),
        "flare": (240, 255, 245, 255),
    },
    "pink_salt": {
        "white": (255, 255, 255, 255),
        "aura1": (255, 230, 225, 255),
        "aura2": (255, 175, 160, 220),
        "flare": (255, 245, 240, 255),
    },
    "selenite": {
        "white": (255, 255, 255, 255),
        "aura1": (245, 250, 255, 255),
        "aura2": (195, 215, 230, 220),
        "flare": (255, 255, 255, 255),
    },
}

def get_sorted_palette(im):
    """Sorted palette of non-transparent colors by luminance."""
    colors = set(im.getpixel((x, y)) for y in range(im.height) for x in range(im.width) if im.getpixel((x, y))[3] > 0)
    sorted_colors = sorted(list(colors), key=lambda c: 0.299*c[0] + 0.587*c[1] + 0.114*c[2])
    return sorted_colors

def generate_cluster_variant(base_im, seed, variant_idx):
    """Generates variant 1 or 2 of a 16x16 normal cluster image."""
    rng = random.Random(seed + variant_idx * 2026)
    pal = get_sorted_palette(base_im)
    n_pal = len(pal)
    if n_pal <= 1:
        return base_im.copy()
        
    c_to_idx = {c: i for i, c in enumerate(pal)}
    out = base_im.copy()
    
    # Identify spire tips and facets
    vis_pixels = [(x, y) for y in range(16) for x in range(16) if base_im.getpixel((x, y))[3] > 0]
    
    # Form 2-3 small contiguous patches of pixels to step up/down in palette
    target_count = int(len(vis_pixels) * (0.19 if variant_idx == 1 else 0.23))
    selected = set()
    num_seeds = rng.randint(2, 4)
    seed_pts = rng.sample(vis_pixels, num_seeds)
    queue = list(seed_pts)
    selected.update(seed_pts)
    
    while queue and len(selected) < target_count:
        cx, cy = queue.pop(0)
        neighbors = [(cx+1, cy), (cx-1, cy), (cx, cy+1), (cx, cy-1)]
        rng.shuffle(neighbors)
        for nx, ny in neighbors:
            if (nx, ny) in vis_pixels and (nx, ny) not in selected and len(selected) < target_count:
                if rng.random() < 0.7:
                    selected.add((nx, ny))
                    queue.append((nx, ny))
                    
    # Apply palette stepping
    for x, y in selected:
        old_c = base_im.getpixel((x, y))
        old_idx = c_to_idx[old_c]
        if variant_idx == 1:
            d = 1 if (x + y) % 2 == 0 else -1
        else:
            d = -1 if (x * 2 + y) % 3 == 0 else 1
        new_idx = old_idx + d
        if new_idx < 0:
            new_idx = 1 if n_pal > 1 else 0
        elif new_idx >= n_pal:
            new_idx = n_pal - 2 if n_pal > 1 else n_pal - 1
        out.putpixel((x, y), pal[new_idx])
        
    # Also apply subtle 1-pixel tip variation:
    # In variant 1, slightly adjust one spire top edge; in variant 2, adjust another
    min_y = min(p[1] for p in vis_pixels)
    top_pixels = [p for p in vis_pixels if p[1] <= min_y + 1]
    if top_pixels:
        tp = rng.choice(top_pixels)
        # Nudge top pixel color to highest highlight
        out.putpixel(tp, pal[-1])
        
    return out

def apply_glint_star(frame, center_x, center_y, theme, intensity=1.0):
    """
    Draws a crisp, radiant 4-point crystal sparkle star centered at (center_x, center_y).
    center: white core
    cross arms: aura1
    diagonals/edges: aura2 (faint glow)
    """
    w, a1, a2, fl = theme["white"], theme["aura1"], theme["aura2"], theme["flare"]
    cx, cy = center_x, center_y
    
    # Blend helper
    def put_glint(x, y, color):
        if 0 <= x < 16 and 0 <= y < 16:
            frame.putpixel((x, y), color)
            
    if intensity >= 0.8:
        # Full flash star: 1 core white, 4 arms aura1, subtle glow aura2
        put_glint(cx, cy, w)
        put_glint(cx + 1, cy, a1)
        put_glint(cx - 1, cy, a1)
        put_glint(cx, cy + 1, a1)
        put_glint(cx, cy - 1, a1)
        # 1-pixel diagonal softening if space permits
        if intensity >= 1.0:
            put_glint(cx + 1, cy - 1, a2)
            put_glint(cx - 1, cy - 1, a2)
    elif intensity >= 0.5:
        # Moderate sparkle: core aura1, subtle cross
        put_glint(cx, cy, fl)
        put_glint(cx + 1, cy, a2)
        put_glint(cx - 1, cy, a2)
        put_glint(cx, cy - 1, a2)
    else:
        # Faint nascent/lingering twinkle
        put_glint(cx, cy, a1)

def highlight_facets(frame, coords, theme):
    """Brightens selected crystal body coordinates to simulate a wave of light."""
    fl = theme["flare"]
    a1 = theme["aura1"]
    for i, (x, y) in enumerate(coords):
        if 0 <= x < 16 and 0 <= y < 16:
            frame.putpixel((x, y), a1 if i % 2 == 0 else fl)

def generate_animated_glint_strip(cluster_16x16, crystal_type, variant_idx):
    """
    Creates a 16x64 vertical strip with 4 frames (each 16x16).
    The glint sweeps smoothly across the crystal spires and facets over the 4 frames.
    """
    theme = GLINT_THEMES[crystal_type]
    strip = Image.new("RGBA", (16, 64), (0, 0, 0, 0))
    
    vis_pixels = [(x, y) for y in range(16) for x in range(16) if cluster_16x16.getpixel((x, y))[3] > 0]
    min_y = min(p[1] for p in vis_pixels)
    
    # Identify key landmarks of this crystal cluster:
    # 1. Primary apex (highest prominent point)
    # 2. Secondary spire (second peak, usually left or right)
    # 3. Mid-body facets
    # 4. Lower-right / base facet
    top_pixels = sorted([p for p in vis_pixels if p[1] <= min_y + 1], key=lambda p: p[0])
    primary_apex = top_pixels[len(top_pixels)//2]
    
    left_spires = [p for p in vis_pixels if p[0] < primary_apex[0] and p[1] <= min_y + 3]
    secondary_apex = left_spires[0] if left_spires else (primary_apex[0] - 2, primary_apex[1] + 2)
    
    mid_facets = [p for p in vis_pixels if min_y + 3 <= p[1] <= min_y + 6 and abs(p[0] - primary_apex[0]) <= 3]
    lower_facets = [p for p in vis_pixels if p[1] >= min_y + 7 and p[0] >= primary_apex[0]]
    
    # Shift timing/locations slightly per variant so the 3 variants don't have identical glint tracks
    if variant_idx == 1:
        # Swap sweep order or alternate spire focus
        apex1 = secondary_apex
        apex2 = primary_apex
    elif variant_idx == 2:
        # Sweep right-to-left
        right_spires = [p for p in vis_pixels if p[0] > primary_apex[0] and p[1] <= min_y + 3]
        apex1 = right_spires[-1] if right_spires else primary_apex
        apex2 = secondary_apex
    else:
        # Base sweep: primary apex -> mid facets -> lower facets
        apex1 = primary_apex
        apex2 = secondary_apex
        
    for frame_idx in range(4):
        fr = cluster_16x16.copy()
        
        if frame_idx == 0:
            # Frame 0: Ambient state + nascent gathering spark on primary tip
            apply_glint_star(fr, apex1[0], apex1[1], theme, intensity=0.4)
            
        elif frame_idx == 1:
            # Frame 1: Full apex flash! Radiant star bursts on top spire
            apply_glint_star(fr, apex1[0], apex1[1], theme, intensity=1.0)
            # Secondary peak begins to catch reflection
            apply_glint_star(fr, apex2[0], apex2[1], theme, intensity=0.5)
            
        elif frame_idx == 2:
            # Frame 2: Glint wave cascades down into mid-body facets and second spire
            apply_glint_star(fr, apex2[0], apex2[1], theme, intensity=0.9)
            if mid_facets:
                highlight_facets(fr, mid_facets[:5], theme)
                
        elif frame_idx == 3:
            # Frame 3: Residual twinkle on lower body facets as glint recedes, ready to loop
            if lower_facets:
                lp = lower_facets[len(lower_facets)//2]
                apply_glint_star(fr, lp[0], lp[1], theme, intensity=0.6)
            if mid_facets:
                highlight_facets(fr, mid_facets[2:4], theme)
                
        strip.paste(fr, (0, frame_idx * 16))
        
    return strip

def main():
    print("=== Processing Caverns Crystal Clusters ===")
    
    for c in CRYSTALS:
        base_cluster_file = f"crystal_{c}_cluster.png"
        base_path = os.path.join(CAVERNS_DIR, base_cluster_file)
        base_im = Image.open(base_path)
        
        # 1. Generate normal cluster variants 1 and 2 (16x16)
        c_seed = abs(hash(c)) % 100000
        var1_im = generate_cluster_variant(base_im, c_seed, 1)
        var2_im = generate_cluster_variant(base_im, c_seed, 2)
        
        var1_path = os.path.join(CAVERNS_DIR, f"crystal_{c}_cluster1.png")
        var2_path = os.path.join(CAVERNS_DIR, f"crystal_{c}_cluster2.png")
        var1_im.save(var1_path)
        var2_im.save(var2_path)
        print(f"Generated normal variants: crystal_{c}_cluster1.png and crystal_{c}_cluster2.png")
        
        # 2. Generate animated glinted clusters (16x64, 4 frames each)
        # Base glinted (on base_im)
        glint_base = generate_animated_glint_strip(base_im, c, 0)
        glint_base_path = os.path.join(CAVERNS_DIR, f"crystal_{c}_cluster_glinted.png")
        glint_base.save(glint_base_path)
        print(f"Updated animated base: crystal_{c}_cluster_glinted.png (size={glint_base.size})")
        
        # Glinted variant 1 (on var1_im)
        glint_var1 = generate_animated_glint_strip(var1_im, c, 1)
        glint_var1_path = os.path.join(CAVERNS_DIR, f"crystal_{c}_cluster_glinted1.png")
        glint_var1.save(glint_var1_path)
        print(f"Generated animated variant 1: crystal_{c}_cluster_glinted1.png (size={glint_var1.size})")
        
        # Glinted variant 2 (on var2_im)
        glint_var2 = generate_animated_glint_strip(var2_im, c, 2)
        glint_var2_path = os.path.join(CAVERNS_DIR, f"crystal_{c}_cluster_glinted2.png")
        glint_var2.save(glint_var2_path)
        print(f"Generated animated variant 2: crystal_{c}_cluster_glinted2.png (size={glint_var2.size})")

    print("\nAll crystal clusters and animated glinted variants successfully generated!")

if __name__ == "__main__":
    main()
