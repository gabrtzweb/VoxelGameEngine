use crate::world::Voxel;

/// Returns the base RGBA map color for a given voxel and optional water depth.
pub fn voxel_map_color(voxel: Voxel, water_depth: u8) -> [u8; 4] {
    match voxel {
        Voxel::Air | Voxel::Occupied => [0, 0, 0, 0],
        Voxel::Liquid_Water | Voxel::WaterOccupied => {
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
        _ => voxel.fallback_color(),
    }
}

/// Returns the RGBA map color for a voxel at world coordinates (world_x, world_z),
/// applying biome-specific palettes and smooth multi-sample transitions.
pub fn voxel_map_color_at(voxel: Voxel, water_depth: u8, world_x: i32, world_z: i32) -> [u8; 4] {
    if voxel.is_tinted() {
        let tint = crate::generation::sample_blended_biome_color(
            voxel,
            world_x as f32,
            world_z as f32,
            1337,
        );
        if voxel.is_water() {
            let depth_factor = if water_depth <= 2 {
                1.15
            } else if water_depth <= 5 {
                1.00
            } else if water_depth <= 10 {
                0.85
            } else {
                0.70
            };
            return [
                (tint[0] * 255.0 * depth_factor).clamp(0.0, 255.0) as u8,
                (tint[1] * 255.0 * depth_factor).clamp(0.0, 255.0) as u8,
                (tint[2] * 255.0 * depth_factor).clamp(0.0, 255.0) as u8,
                255,
            ];
        } else if matches!(
            voxel,
            Voxel::Soil_Grass | Voxel::Soil_Peat_Grass | Voxel::Soil_Silt_Grass
        ) {
            return [
                (tint[0] * 175.0).clamp(0.0, 255.0) as u8,
                (tint[1] * 185.0).clamp(0.0, 255.0) as u8,
                (tint[2] * 145.0).clamp(0.0, 255.0) as u8,
                255,
            ];
        } else if voxel.is_leaves() {
            return [
                (tint[0] * 120.0).clamp(0.0, 255.0) as u8,
                (tint[1] * 130.0).clamp(0.0, 255.0) as u8,
                (tint[2] * 110.0).clamp(0.0, 255.0) as u8,
                255,
            ];
        }
    }
    voxel_map_color(voxel, water_depth)
}

/// Applies authentic North-up topographic hill shading to simulate sunlight and elevation relief.
#[inline]
pub fn apply_relief_shading(
    color: [u8; 4],
    current_height: i16,
    north_height: Option<i16>,
) -> [u8; 4] {
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
