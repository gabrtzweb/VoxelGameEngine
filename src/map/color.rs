use crate::world::Voxel;

/// Returns the base RGBA map color for a given voxel and optional water depth.
pub fn voxel_map_color(voxel: Voxel, water_depth: u8) -> [u8; 4] {
    match voxel {
        Voxel::Air | Voxel::Occupied => [0, 0, 0, 0],

        // Natural terrain & soil
        Voxel::Grass => [92, 172, 60, 255],
        Voxel::SnowyGrass => [242, 246, 252, 255],
        Voxel::Dirt | Voxel::PackedDirt => [134, 96, 67, 255],
        Voxel::Sand => [218, 204, 150, 255],
        Voxel::RedSand => [195, 102, 50, 255],
        Voxel::Clay => [150, 155, 175, 255],
        Voxel::Gravel => [130, 125, 125, 255],
        Voxel::Moss => [80, 130, 60, 255],
        Voxel::RedMoss => [175, 45, 45, 255],
        Voxel::Mud | Voxel::PackedMud => [85, 60, 45, 255],
        Voxel::Mulch => [90, 55, 35, 255],
        Voxel::Snow => [240, 245, 255, 255],
        Voxel::Ice | Voxel::PackedIce => [155, 195, 245, 255],

        // Rocks & Minerals
        Voxel::Stone | Voxel::Andesite => [125, 125, 128, 255],
        Voxel::Diorite => [180, 180, 185, 255],
        Voxel::Granite => [150, 105, 90, 255],
        Voxel::Cobblestone => [112, 112, 116, 255],
        Voxel::MossyCobblestone | Voxel::MossyStone => [95, 120, 85, 255],
        Voxel::Slate | Voxel::Cobbleslate => [80, 85, 95, 255],
        Voxel::Blackstone | Voxel::Cobbleblackstone => [45, 42, 48, 255],
        Voxel::Flint => [55, 55, 60, 255],
        Voxel::Dreadstone => [32, 28, 38, 255],
        Voxel::Tuff => [105, 108, 100, 255],
        Voxel::Sandstone => [215, 205, 150, 255],
        Voxel::RedSandstone => [185, 95, 45, 255],
        Voxel::Blueschist => [75, 95, 115, 255],
        Voxel::Calcite => [220, 220, 225, 255],
        Voxel::Dripstone => [135, 105, 90, 255],
        Voxel::Limestone => [195, 185, 165, 255],
        Voxel::Ochrestone => [185, 135, 55, 255],
        Voxel::Rhodonite => [195, 110, 135, 255],
        Voxel::Serpentinite => [70, 115, 85, 255],
        Voxel::Basalt => [75, 75, 80, 255],

        // Woods & Foliage
        Voxel::OakWood | Voxel::OakWoodLog => [133, 94, 56, 255],
        Voxel::BirchWood | Voxel::BirchWoodLog => [215, 210, 198, 255],
        Voxel::PineWood | Voxel::PineWoodLog => [74, 48, 28, 255],
        Voxel::OakLeaves => [72, 140, 42, 255],
        Voxel::BirchLeaves => [115, 165, 50, 255],
        Voxel::PineLeaves => [40, 95, 55, 255],
        Voxel::Cactus => [85, 135, 45, 255],

        // Fluids with depth tinting
        Voxel::Water | Voxel::WaterFlowing | Voxel::WaterOccupied => {
            if water_depth <= 2 {
                // Shallow coastal water
                [72, 165, 235, 255]
            } else if water_depth <= 5 {
                // Shelf water
                [52, 132, 218, 255]
            } else if water_depth <= 10 {
                // Deep water
                [38, 98, 192, 255]
            } else {
                // Oceanic abyss
                [25, 70, 160, 255]
            }
        }
        Voxel::Lava => [235, 105, 20, 255],

        // Light sources
        Voxel::Light | Voxel::LightWarm => [255, 205, 80, 255],
        Voxel::LightCold => [200, 230, 255, 255],
        Voxel::LightRed => [255, 70, 70, 255],
        Voxel::LightGreen => [70, 255, 70, 255],
        Voxel::LightBlue => [70, 140, 255, 255],
        Voxel::Magma => [180, 70, 30, 255],
    }
}

/// Applies authentic North-up topographic hill shading to simulate sunlight and elevation relief.
#[inline]
pub fn apply_relief_shading(color: [u8; 4], current_height: i16, north_height: Option<i16>) -> [u8; 4] {
    let Some(north_h) = north_height else {
        return color;
    };

    let diff = current_height - north_h;
    if diff == 0 {
        return color;
    }

    let factor = if diff > 0 {
        // Slope faces south towards sun -> highlight
        1.0 + (diff as f32 * 0.08).min(0.25)
    } else {
        // Slope faces north in shadow -> darken
        1.0 - ((-diff) as f32 * 0.08).min(0.25)
    };

    [
        (color[0] as f32 * factor).clamp(0.0, 255.0) as u8,
        (color[1] as f32 * factor).clamp(0.0, 255.0) as u8,
        (color[2] as f32 * factor).clamp(0.0, 255.0) as u8,
        color[3],
    ]
}

/// Returns the RGBA color for unexplored territory with a subtle chunk grid pattern.
#[inline]
pub fn unexplored_color(wx: i32, wz: i32) -> [u8; 4] {
    let on_chunk_border = wx.rem_euclid(16) == 0 || wz.rem_euclid(16) == 0;
    if on_chunk_border {
        [28, 30, 38, 255]
    } else {
        [18, 19, 24, 255]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxel_map_colors() {
        let grass = voxel_map_color(Voxel::Grass, 0);
        assert_eq!(grass[3], 255);
        assert!(grass[1] > grass[0]); // Green channel dominant

        let shallow_water = voxel_map_color(Voxel::Water, 1);
        let deep_water = voxel_map_color(Voxel::Water, 15);
        assert_ne!(shallow_water, deep_water);
        // Deep water should be darker blue than shallow water
        assert!(deep_water[2] < shallow_water[2] || deep_water[0] < shallow_water[0]);
    }

    #[test]
    fn test_relief_shading() {
        let base = [100, 100, 100, 255];

        // Flat ground -> identical
        let flat = apply_relief_shading(base, 50, Some(50));
        assert_eq!(flat, base);

        // South-facing slope (higher than north) -> brighter
        let highlighted = apply_relief_shading(base, 55, Some(50));
        assert!(highlighted[0] > base[0]);

        // North-facing slope (lower than north) -> darker
        let shadowed = apply_relief_shading(base, 45, Some(50));
        assert!(shadowed[0] < base[0]);
    }

    #[test]
    fn test_unexplored_color_grid() {
        let grid_pixel = unexplored_color(16, 32);
        let void_pixel = unexplored_color(5, 7);
        assert_ne!(grid_pixel, void_pixel);
        assert_eq!(grid_pixel, [28, 30, 38, 255]);
        assert_eq!(void_pixel, [18, 19, 24, 255]);
    }
}

