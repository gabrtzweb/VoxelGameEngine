use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

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

pub fn terrain_inspector_ui(mut contexts: EguiContexts, mut generator: ResMut<TerrainGenerator>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Terrain & World Generation Inspector")
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

            egui::CollapsingHeader::new("Geography & Elevation")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Seed:");
                        ui.add(egui::DragValue::new(&mut generator.seed));
                    });
                    ui.add(egui::Slider::new(&mut generator.sea_level, 0..=24).text("Sea Level (voxels)"));
                    ui.add(egui::Slider::new(&mut generator.base_height, -10.0..=25.0).text("Base Height"));
                    ui.add(egui::Slider::new(&mut generator.macro_amplitude, 0.0..=25.0).text("Macro Amplitude"));
                    ui.add(egui::Slider::new(&mut generator.macro_frequency, 0.001..=0.030).text("Macro Freq"));
                    ui.add(egui::Slider::new(&mut generator.detail_amplitude, 0.0..=10.0).text("Detail Amplitude"));
                    ui.add(egui::Slider::new(&mut generator.detail_frequency, 0.01..=0.10).text("Detail Freq"));
                });

            egui::CollapsingHeader::new("Rivers & Water")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut generator.river_frequency, 0.001..=0.010).text("River Freq"));
                    ui.add(egui::Slider::new(&mut generator.river_width, 0.01..=0.08).text("River Width"));
                });

            egui::CollapsingHeader::new("Caves & Ravines")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut generator.caves.spaghetti_threshold, 0.06..=0.25).text("Tunnel Width (Spaghetti)"));
                    ui.add(egui::Slider::new(&mut generator.caves.spaghetti_freq, 0.01..=0.06).text("Tunnel Freq"));
                    ui.add(egui::Slider::new(&mut generator.caves.cheese_threshold, 0.20..=0.60).text("Cavern Openness (Cheese)"));
                    ui.add(egui::Slider::new(&mut generator.caves.ravine_abundance, 0.00..=0.50).text("Ravine Abundance"));
                    ui.add(egui::Slider::new(&mut generator.caves.ravine_freq, 0.001..=0.015).text("Ravine Freq"));
                    ui.add(egui::Slider::new(&mut generator.caves.ravine_width, 0.01..=0.05).text("Ravine Width"));
                    ui.add(egui::Slider::new(&mut generator.caves.deep_lava_y, -120..=-20).text("Deep Lava Y"));
                });

            egui::CollapsingHeader::new("Strata & Minerals")
                .default_open(false)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut generator.strata.mid_crust_y, -50..=0).text("Slate Level Y"));
                    ui.add(egui::Slider::new(&mut generator.strata.deep_crust_y, -120..=-30).text("Blackstone Level Y"));
                    ui.add(egui::Slider::new(&mut generator.strata.vein_frequency, 0.02..=0.20).text("Vein Freq"));
                });

            egui::CollapsingHeader::new("Climate & Biomes")
                .default_open(false)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut generator.climate.continentalness_freq, 0.0005..=0.008).text("Continentalness Freq"));
                    ui.add(egui::Slider::new(&mut generator.climate.temperature_freq, 0.0005..=0.008).text("Temperature Freq"));
                    ui.add(egui::Slider::new(&mut generator.climate.humidity_freq, 0.0005..=0.008).text("Humidity Freq"));
                });

            ui.add_space(8.0);
            if ui.button("Reset Defaults").clicked() {
                let v = generator.version.wrapping_add(1);
                *generator = TerrainGenerator::default();
                generator.version = v;
            }
        });
}
