use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use super::generator::TerrainGenerator;
use crate::player::InspectorInteraction;

pub struct TerrainInspectorPlugin;

impl Plugin for TerrainInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            terrain_inspector_ui.run_if(|inspector: Res<InspectorInteraction>| inspector.active),
        );
    }
}

pub fn terrain_inspector_ui(
    mut contexts: EguiContexts,
    mut generator: ResMut<TerrainGenerator>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    env_status: Option<Res<crate::player::PlayerEnvironmentStatus>>,
    mut streaming_settings: Option<
        ResMut<crate::world::streaming::manager::ChunkStreamingSettings>,
    >,
    mesh_registry: Option<Res<crate::meshing::ChunkMeshRegistry>>,
    lod_registry: Option<Res<crate::meshing::LodMeshRegistry>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Terrain & World Generation Inspector")
        .id(egui::Id::new("terrain_world_gen_inspector_window"))
        .default_pos(egui::pos2(840.0, 30.0))
        .default_width(380.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("World Generation");
                if ui.button("Regenerate World").clicked() {
                    generator.version = generator.version.wrapping_add(1);
                }
            });
            ui.label("Tweak procedural parameters live and click 'Regenerate World' to reload all chunks.");
            ui.separator();

            let mut world_changed = false;

            egui::CollapsingHeader::new("Live Telemetry & Diagnostics")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(transform) = player_query.iter().next() {
                        let pos = transform.translation;
                        ui.label(format!("Player Pos: ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z));

                        let sample = generator.climate.sample_dithered(pos.x, pos.z, generator.seed);
                        ui.label(format!("Current Biome: {}", sample.biome.name()));
                        ui.label(format!(
                            "6D Noise: C: {:+.3} | E: {:+.3} | R: {:+.3}",
                            sample.continentalness, sample.erosion, sample.ridges
                        ));
                        ui.label(format!(
                            "Climate:  T: {:+.3} | H: {:+.3} | D: {:+.3}",
                            sample.temperature, sample.humidity, sample.depth
                        ));
                        ui.label("Vegetation: Bypassed during terrain overhaul");
                    }
                    if let Some(ref status) = env_status {
                        ui.label(format!(
                            "Camera Underwater: {} (submersion: {:.2})",
                            status.is_camera_in_water, status.submersion
                        ));
                    }
                });

            egui::CollapsingHeader::new("Geography & Elevation")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Seed:");
                        if ui.add(egui::DragValue::new(&mut generator.seed)).changed() {
                            world_changed = true;
                        }
                        if ui.button("Random").clicked() {
                            generator.seed = generator.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                            world_changed = true;
                        }
                    });
                    world_changed |= ui.add(egui::Slider::new(&mut generator.sea_level, 0..=24).text("Sea Level (voxels)")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.base_height, -10.0..=25.0).text("Base Height")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.macro_amplitude, 0.0..=25.0).text("Macro Amplitude")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.macro_frequency, 0.001..=0.040).text("Macro Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.detail_amplitude, 0.0..=12.0).text("Detail Amplitude")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.detail_frequency, 0.01..=0.10).text("Detail Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.mountain_ridge_height, 0.0..=120.0).text("Mountain Ridge Height")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.rolling_hills_amplitude, 0.0..=20.0).text("Rolling Hills Amp")).changed();
                });

            egui::CollapsingHeader::new("Rivers, Lakes & Hydrology")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Rivers & Streams").strong());
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.river_frequency, 0.0010..=0.0080).text("River Frequency"))
                        .on_hover_text("Density and spacing of continental river networks")
                        .changed();
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.river_width, 0.020..=0.100).text("River Channel Width"))
                        .on_hover_text("Channel width of rivers and streams")
                        .changed();
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.river_depth, 1.5..=6.5).text("River Channel Depth"))
                        .on_hover_text("Carve depth below river water surface (blocks)")
                        .changed();
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.tributary_abundance, 0.0..=1.0).text("Tributaries & Brooks"))
                        .on_hover_text("Frequency of secondary tributary branches and mountain cascades")
                        .changed();
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.reach_flatness, 1.0..=12.0).text("Reach Flatness (Pool Step)"))
                        .on_hover_text("Elevation step height for calm pools and waterfall cascades (higher = flatter pools)")
                        .changed();
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.stream_flow_speed, 0.5..=5.0).text("Stream Current Speed"))
                        .on_hover_text("Downhill fluid current push velocity in rivers (m/s)")
                        .changed();

                    ui.separator();
                    ui.label(egui::RichText::new("Organic Lakes & Ponds").strong());
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.lake_abundance, 0.0..=1.0).text("Lake Abundance"))
                        .on_hover_text("Spawn frequency of natural organic lakes in valley basins and meadows")
                        .changed();
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.lake_size, 8.0..=40.0).text("Lake Radius"))
                        .on_hover_text("Average radius of organic lake basins")
                        .changed();
                    world_changed |= ui
                        .add(egui::Slider::new(&mut generator.lake_depth, 1.5..=8.0).text("Lake Depth"))
                        .on_hover_text("Water depth of organic lakes (blocks)")
                        .changed();
                });

            egui::CollapsingHeader::new("Caves & Ravines")
                .default_open(false)
                .show(ui, |ui| {
                    world_changed |= ui.add(egui::Slider::new(&mut generator.caves.spaghetti_threshold, 0.06..=0.25).text("Tunnel Width (Spaghetti)")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.caves.spaghetti_freq, 0.01..=0.06).text("Tunnel Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.caves.cheese_threshold, 0.20..=0.60).text("Cavern Openness (Cheese)")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.caves.ravine_abundance, 0.00..=0.50).text("Ravine Abundance")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.caves.ravine_freq, 0.001..=0.015).text("Ravine Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.caves.ravine_width, 0.01..=0.05).text("Ravine Width")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.caves.deep_lava_y, -120..=-20).text("Deep Lava Y")).changed();
                });

            egui::CollapsingHeader::new("Strata & Minerals")
                .default_open(false)
                .show(ui, |ui| {
                    world_changed |= ui.add(egui::Slider::new(&mut generator.strata.mid_crust_y, -200..=0).text("Blackstone Level Y")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.strata.deep_crust_y, -250..=-50).text("Deep Crust Y")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.strata.vein_frequency, 0.02..=0.20).text("Vein Freq")).changed();
                });

            egui::CollapsingHeader::new("Climate & Biomes")
                .default_open(true)
                .show(ui, |ui| {
                    world_changed |= ui.add(egui::Slider::new(&mut generator.climate.continentalness_freq, 0.0005..=0.008).text("Continentalness Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.climate.erosion_freq, 0.0005..=0.008).text("Erosion Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.climate.ridges_freq, 0.0005..=0.008).text("Ridges Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.climate.temperature_freq, 0.0005..=0.008).text("Temperature Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.climate.humidity_freq, 0.0005..=0.008).text("Humidity Freq")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.climate.warp_amplitude, 0.0..=80.0).text("Domain Warp Amp")).changed();
                    world_changed |= ui.add(egui::Slider::new(&mut generator.climate.dither_amplitude, 0.0..=25.0).text("Micro Dither Amp")).changed();
                });

            egui::CollapsingHeader::new("Vegetation & Trees")
                .default_open(true)
                .show(ui, |ui| {
                    world_changed |= ui.add(
                        egui::Slider::new(&mut generator.tree_density, 0.0..=3.0)
                            .text("Tree Density Multiplier"),
                    ).changed();
                });

            if world_changed {
                generator.version = generator.version.wrapping_add(1);
            }

            egui::CollapsingHeader::new("Render Distance & LOD (Stage 13.1)")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(ref mut settings) = streaming_settings {
                        ui.add(
                            egui::Slider::new(&mut settings.render_distance, 2..=16)
                                .text("Real Render Distance (LOD 0)"),
                        );
                        ui.add(
                            egui::Slider::new(&mut settings.lod_render_distance, 0..=64)
                                .text("LOD Render Distance (Far Horizon)"),
                        );
                        if settings.lod_render_distance == 0 {
                            ui.label(
                                egui::RichText::new(
                                    "LOD chunks disabled (0 chunks beyond real distance)",
                                )
                                .italics()
                                .color(egui::Color32::from_rgb(200, 160, 80)),
                            );
                        } else {
                            let total_r = settings.render_distance + settings.lod_render_distance;
                            ui.label(format!(
                                "Total Horizon: {} chunks (~{}m radius)",
                                total_r,
                                total_r * 16
                            ));
                        }
                    }
                    if let Some(ref real_reg) = mesh_registry {
                        ui.label(format!(
                            "Real Mesh Chunks (LOD 0): {} | {} verts ({} tris)",
                            real_reg.len(),
                            real_reg.total_vertices(),
                            real_reg.total_triangles()
                        ));
                    }
                    if let Some(ref lod_reg) = lod_registry {
                        let lod1_count = lod_reg.lod_count(crate::meshing::ChunkLod::Lod1);
                        let lod2_count = lod_reg.lod_count(crate::meshing::ChunkLod::Lod2);
                        ui.label(format!(
                            "LOD Meshes: {} (LOD 1: {}, LOD 2 [2x2 Batched]: {}) | {} verts ({} tris)",
                            lod_reg.len(),
                            lod1_count,
                            lod2_count,
                            lod_reg.total_vertices(),
                            lod_reg.total_triangles()
                        ));
                    }
                });

            ui.add_space(8.0);
            if ui.button("Reset Defaults").clicked() {
                let v = generator.version.wrapping_add(1);
                *generator = TerrainGenerator::default();
                generator.version = v;
                if let Some(ref mut settings) = streaming_settings {
                    **settings =
                        crate::world::streaming::manager::ChunkStreamingSettings::default();
                }
            }
        });
}
