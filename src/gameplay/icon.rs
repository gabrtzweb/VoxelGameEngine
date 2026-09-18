use std::{collections::HashMap, path::Path};

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::world::Voxel;

pub const ICON_SIZE: u32 = 32;

#[derive(Resource, Clone, Default)]
pub struct BlockIcons {
    icons: HashMap<Voxel, Handle<Image>>,
    fallback: Handle<Image>,
}

impl BlockIcons {
    pub fn get(&self, voxel: Voxel) -> Handle<Image> {
        self.icons
            .get(&voxel)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }
}

pub struct BlockIconPlugin;

impl Plugin for BlockIconPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockIcons>()
            .add_systems(Startup, setup_block_icons);
    }
}

pub fn setup_block_icons(mut images: ResMut<Assets<Image>>, mut block_icons: ResMut<BlockIcons>) {
    let mut fallback_image = None;

    for &voxel in &Voxel::ALL {
        if voxel.is_empty() || voxel == Voxel::Occupied || voxel == Voxel::WaterOccupied {
            continue;
        }

        let mut raw_side = if let Some(side_name) = voxel.side_texture_name() {
            load_raw_16x16_by_name(side_name, voxel.fallback_color())
        } else {
            load_voxel_raw_16x16(voxel)
        };

        if voxel == Voxel::Grass {
            let overlay_path = "assets/textures/blocks/terr_grass_side_overlay.png";
            if Path::new(overlay_path).exists()
                && let Ok(img) = image::open(overlay_path)
            {
                let rgba = img.into_rgba8();
                let tint = [0.58f32, 0.90, 0.44];
                let raw = rgba.into_raw();
                for idx in (0..raw_side.len().min(raw.len())).step_by(4) {
                    let ov_a = (raw[idx + 3] as f32) / 255.0;
                    if ov_a > 0.001 {
                        let ov_r = raw[idx] as f32 * tint[0];
                        let ov_g = raw[idx + 1] as f32 * tint[1];
                        let ov_b = raw[idx + 2] as f32 * tint[2];

                        let base_r = raw_side[idx] as f32;
                        let base_g = raw_side[idx + 1] as f32;
                        let base_b = raw_side[idx + 2] as f32;

                        raw_side[idx] = (base_r * (1.0 - ov_a) + ov_r * ov_a).round().clamp(0.0, 255.0) as u8;
                        raw_side[idx + 1] = (base_g * (1.0 - ov_a) + ov_g * ov_a).round().clamp(0.0, 255.0) as u8;
                        raw_side[idx + 2] = (base_b * (1.0 - ov_a) + ov_b * ov_a).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }

        let raw_top = if let Some(top_name) = voxel.top_texture_name() {
            load_raw_16x16_by_name(top_name, voxel.fallback_color())
        } else if voxel.side_texture_name().is_some() {
            load_voxel_raw_16x16(voxel)
        } else {
            raw_side.clone()
        };

        let side_tint = if voxel == Voxel::Grass {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            voxel.tint_color()
        };
        let top_tint = voxel.tint_color();

        let icon_image = render_isometric_block_icon_multi(&raw_side, &raw_top, side_tint, top_tint);
        let handle = images.add(icon_image);

        if fallback_image.is_none() {
            fallback_image = Some(handle.clone());
        }

        block_icons.icons.insert(voxel, handle);
    }

    // Also support legacy Light alias pointing to LightWarm
    if let Some(warm_handle) = block_icons.icons.get(&Voxel::LightWarm).cloned() {
        block_icons.icons.insert(Voxel::Light, warm_handle);
    }

    if let Some(fb) = fallback_image {
        block_icons.fallback = fb;
    }

    info!(
        "Generated {} 3D isometric block icons on-the-fly",
        block_icons.icons.len()
    );
}

fn load_voxel_raw_16x16(voxel: Voxel) -> Vec<u8> {
    if let Some(name) = voxel.texture_name() {
        load_raw_16x16_by_name(name, voxel.fallback_color())
    } else {
        solid_raw_16x16(voxel.fallback_color())
    }
}

fn load_raw_16x16_by_name(name: &str, fallback_color: [u8; 4]) -> Vec<u8> {
    let candidate_paths = [
        format!("assets/textures/blocks/{name}.png"),
        format!("assets/textures/blocks/{name}0.png"),
        format!("assets/textures/blocks/{name}_0.png"),
        format!("assets/textures/blocks/{name}_1.png"),
        format!("assets/textures/blocks/{name}1.png"),
        format!("assets/textures/blocks/block_{name}.png"),
    ];

    for path in &candidate_paths {
        if Path::new(path).exists()
            && let Ok(img) = image::open(path)
        {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            if w >= 16 && h >= 16 {
                let mut frame = Vec::with_capacity(16 * 16 * 4);
                for y in 0..16 {
                    for x in 0..16 {
                        let p = rgba.get_pixel(x, y);
                        frame.extend_from_slice(&p.0);
                    }
                }
                return frame;
            }
        }
    }

    solid_raw_16x16(fallback_color)
}

fn solid_raw_16x16(color: [u8; 4]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..256 {
        frame.extend_from_slice(&color);
    }
    frame
}

/// Renders an authentic 2:1 pixel-art isometric cube with Top, Left, and Right faces,
/// directional face shading (1.0 / 0.80 / 0.60), vertex tinting, and a subtle silhouette outline.
#[allow(dead_code)]
pub fn render_isometric_block_icon(tex_16x16: &[u8], tint: [f32; 4]) -> Image {
    render_isometric_block_icon_multi(tex_16x16, tex_16x16, tint, tint)
}

/// Renders an authentic 2:1 pixel-art isometric cube with distinct Top and Side textures and tints.
pub fn render_isometric_block_icon_multi(
    side_16x16: &[u8],
    top_16x16: &[u8],
    side_tint: [f32; 4],
    top_tint: [f32; 4],
) -> Image {
    let mut canvas = vec![0u8; (ICON_SIZE * ICON_SIZE * 4) as usize];

    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let x_f = x as f32;
            let y_f = y as f32;

            // 1. Check Top Face (Rhombus with T=(15, 4), TR=(27, 10), C=(15, 16), TL=(3, 10))
            let u_top = ((x_f - 15.0) + 2.0 * (y_f - 4.0)) / 24.0;
            let v_top = (-(x_f - 15.0) + 2.0 * (y_f - 4.0)) / 24.0;

            if (0.0..1.0).contains(&u_top) && (0.0..1.0).contains(&v_top) {
                let tx = (u_top * 16.0).floor().clamp(0.0, 15.0) as usize;
                let ty = (v_top * 16.0).floor().clamp(0.0, 15.0) as usize;
                draw_texel(&mut canvas, x, y, top_16x16, tx, ty, top_tint, 1.00);
                continue;
            }

            // 2. Check Left Face (Parallelogram with TL=(3, 10), C=(15, 16), B=(15, 28), BL=(3, 22))
            if (3..15).contains(&x) {
                let u_left = (x_f - 3.0) / 12.0;
                let y_top = 10.0 + (x_f - 3.0) * 0.5;
                let v_left = (y_f - y_top) / 12.0;

                if (0.0..1.0).contains(&u_left) && (0.0..1.0).contains(&v_left) {
                    let tx = (u_left * 16.0).floor().clamp(0.0, 15.0) as usize;
                    let ty = (v_left * 16.0).floor().clamp(0.0, 15.0) as usize;
                    draw_texel(&mut canvas, x, y, side_16x16, tx, ty, side_tint, 0.80);
                    continue;
                }
            }

            // 3. Check Right Face (Parallelogram with C=(15, 16), TR=(27, 10), BR=(27, 22), B=(15, 28))
            if (15..28).contains(&x) {
                let u_right = (x_f - 15.0) / 12.0;
                let y_top = 16.0 - (x_f - 15.0) * 0.5;
                let v_right = (y_f - y_top) / 12.0;

                if (0.0..1.0).contains(&u_right) && (0.0..1.0).contains(&v_right) {
                    let tx = (u_right * 16.0).floor().clamp(0.0, 15.0) as usize;
                    let ty = (v_right * 16.0).floor().clamp(0.0, 15.0) as usize;
                    draw_texel(&mut canvas, x, y, side_16x16, tx, ty, side_tint, 0.60);
                    continue;
                }
            }
        }
    }

    apply_silhouette_outline(&mut canvas);

    let mut image = Image::new_fill(
        Extent3d {
            width: ICON_SIZE,
            height: ICON_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &canvas,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::nearest();
    image
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn draw_texel(
    canvas: &mut [u8],
    cx: u32,
    cy: u32,
    tex: &[u8],
    tx: usize,
    ty: usize,
    tint: [f32; 4],
    shading: f32,
) {
    let src_idx = (ty * 16 + tx) * 4;
    if src_idx + 3 >= tex.len() {
        return;
    }

    let r = tex[src_idx] as f32 / 255.0;
    let g = tex[src_idx + 1] as f32 / 255.0;
    let b = tex[src_idx + 2] as f32 / 255.0;
    let a = tex[src_idx + 3] as f32 / 255.0;

    let out_idx = ((cy * ICON_SIZE + cx) * 4) as usize;
    canvas[out_idx] = ((r * tint[0] * shading).clamp(0.0, 1.0) * 255.0) as u8;
    canvas[out_idx + 1] = ((g * tint[1] * shading).clamp(0.0, 1.0) * 255.0) as u8;
    canvas[out_idx + 2] = ((b * tint[2] * shading).clamp(0.0, 1.0) * 255.0) as u8;
    canvas[out_idx + 3] = ((a * tint[3]).clamp(0.0, 1.0) * 255.0) as u8;
}

fn apply_silhouette_outline(canvas: &mut [u8]) {
    let original = canvas.to_vec();

    for y in 1..(ICON_SIZE - 1) {
        for x in 1..(ICON_SIZE - 1) {
            let idx = ((y * ICON_SIZE + x) * 4) as usize;
            if original[idx + 3] > 0 {
                continue;
            }

            let mut has_opaque_neighbor = false;
            for dy in [-1i32, 0, 1] {
                for dx in [-1i32, 0, 1] {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let n_idx = ((ny as u32 * ICON_SIZE + nx as u32) * 4) as usize;
                    if original[n_idx + 3] > 50 {
                        has_opaque_neighbor = true;
                        break;
                    }
                }
                if has_opaque_neighbor {
                    break;
                }
            }

            if has_opaque_neighbor {
                canvas[idx] = 16;
                canvas[idx + 1] = 16;
                canvas[idx + 2] = 20;
                canvas[idx + 3] = 160;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isometric_icon_rasterization_produces_valid_rgba_image() {
        let test_texture = vec![255u8; 16 * 16 * 4];
        let image = render_isometric_block_icon(&test_texture, [1.0, 1.0, 1.0, 1.0]);

        assert_eq!(image.texture_descriptor.size.width, 32);
        assert_eq!(image.texture_descriptor.size.height, 32);

        let data = image.data.as_ref().expect("Image data must be present");
        assert_eq!(data.len(), 32 * 32 * 4);

        let center_idx = (16 * 32 + 15) * 4;
        assert!(data[center_idx + 3] > 0);

        assert_eq!(data[3], 0);
    }
}
