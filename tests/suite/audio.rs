use std::path::Path;
use voxel_game_engine::{
    audio::{PlaySfxEvent, PlaySpatialSfxEvent, SoundMaterial, UiSound},
    world::Voxel,
};

#[test]
fn test_sound_material_mapping() {
    assert_eq!(SoundMaterial::from_voxel(Voxel::LoamGrass), SoundMaterial::Grass);
    assert_eq!(SoundMaterial::from_voxel(Voxel::CalicheGrass), SoundMaterial::Grass);
    assert_eq!(SoundMaterial::from_voxel(Voxel::MossForest), SoundMaterial::Grass);

    assert_eq!(SoundMaterial::from_voxel(Voxel::OakLog), SoundMaterial::Wood);
    assert_eq!(SoundMaterial::from_voxel(Voxel::OakPlanks), SoundMaterial::Wood);
    assert_eq!(SoundMaterial::from_voxel(Voxel::BambooStalk), SoundMaterial::Wood);

    assert_eq!(SoundMaterial::from_voxel(Voxel::LoamDirt), SoundMaterial::Dirt);
    assert_eq!(SoundMaterial::from_voxel(Voxel::PeatDirt), SoundMaterial::Dirt);
    assert_eq!(SoundMaterial::from_voxel(Voxel::CalicheDirt), SoundMaterial::Dirt);

    assert_eq!(SoundMaterial::from_voxel(Voxel::SandCommon), SoundMaterial::Sand);
    assert_eq!(SoundMaterial::from_voxel(Voxel::SandRed), SoundMaterial::Sand);

    assert_eq!(SoundMaterial::from_voxel(Voxel::GravelAshy), SoundMaterial::Gravel);
    assert_eq!(SoundMaterial::from_voxel(Voxel::GravelDirty), SoundMaterial::Gravel);

    assert_eq!(SoundMaterial::from_voxel(Voxel::SnowBlock), SoundMaterial::Snow);
    assert_eq!(SoundMaterial::from_voxel(Voxel::PowderSnow), SoundMaterial::Snow);
    assert_eq!(SoundMaterial::from_voxel(Voxel::FrostIce), SoundMaterial::Snow);

    assert_eq!(SoundMaterial::from_voxel(Voxel::OakLeaves), SoundMaterial::Leaves);
    assert_eq!(SoundMaterial::from_voxel(Voxel::PineLeaves), SoundMaterial::Leaves);

    assert_eq!(SoundMaterial::from_voxel(Voxel::GlassWhite), SoundMaterial::Glass);
    assert_eq!(SoundMaterial::from_voxel(Voxel::TintedGlass), SoundMaterial::Glass);

    assert_eq!(SoundMaterial::from_voxel(Voxel::LiquidWater), SoundMaterial::Water);
    assert_eq!(SoundMaterial::from_voxel(Voxel::WaterOccupied), SoundMaterial::Water);

    assert_eq!(SoundMaterial::from_voxel(Voxel::CobbledGranite), SoundMaterial::Stone);
    assert_eq!(SoundMaterial::from_voxel(Voxel::Andesite), SoundMaterial::Stone);
    assert_eq!(SoundMaterial::from_voxel(Voxel::Basalt), SoundMaterial::Stone);
}

#[test]
fn test_all_material_audio_assets_exist_on_disk() {
    let materials = [
        SoundMaterial::Grass,
        SoundMaterial::Dirt,
        SoundMaterial::Stone,
        SoundMaterial::Wood,
        SoundMaterial::Sand,
        SoundMaterial::Gravel,
        SoundMaterial::Snow,
        SoundMaterial::Leaves,
        SoundMaterial::Mud,
        SoundMaterial::Slate,
        SoundMaterial::Glass,
        SoundMaterial::Water,
    ];

    for mat in materials {
        for path in mat.break_sounds() {
            let full_path = Path::new("assets").join(path);
            assert!(
                full_path.exists(),
                "Break sound asset does not exist: {:?}",
                full_path
            );
        }
        for path in mat.place_sounds() {
            let full_path = Path::new("assets").join(path);
            assert!(
                full_path.exists(),
                "Place sound asset does not exist: {:?}",
                full_path
            );
        }
        for path in mat.hit_sounds() {
            let full_path = Path::new("assets").join(path);
            assert!(
                full_path.exists(),
                "Hit sound asset does not exist: {:?}",
                full_path
            );
        }
        for path in mat.walk_sounds() {
            let full_path = Path::new("assets").join(path);
            assert!(
                full_path.exists(),
                "Walk sound asset does not exist: {:?}",
                full_path
            );
        }
        for path in mat.run_sounds() {
            let full_path = Path::new("assets").join(path);
            assert!(
                full_path.exists(),
                "Run sound asset does not exist: {:?}",
                full_path
            );
        }
        for path in mat.land_sounds() {
            let full_path = Path::new("assets").join(path);
            assert!(
                full_path.exists(),
                "Land sound asset does not exist: {:?}",
                full_path
            );
        }
    }
}

#[test]
fn test_ui_and_spatial_sfx_builders() {
    assert_eq!(UiSound::Click, UiSound::Click);
    assert_ne!(UiSound::Click, UiSound::Rollover);

    let dummy_handle = bevy::asset::Handle::default();
    let spatial_ev = PlaySpatialSfxEvent::new(
        dummy_handle.clone(),
        bevy::math::Vec3::new(10.0, 5.0, 10.0),
        0.8,
        32.0,
    )
    .with_speed(1.05);

    assert_eq!(spatial_ev.volume, 0.8);
    assert_eq!(spatial_ev.max_distance, 32.0);
    assert_eq!(spatial_ev.speed, 1.05);

    let sfx_ev = PlaySfxEvent::new(dummy_handle, 0.5).with_speed(0.95);
    assert_eq!(sfx_ev.volume, 0.5);
    assert_eq!(sfx_ev.speed, 0.95);
}
