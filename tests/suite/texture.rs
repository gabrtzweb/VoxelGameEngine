use bevy::prelude::*;
use voxel_game_engine::{
    meshing::{
        VoxelTextureRegistry,
        greedy::FaceDirection,
        textures::{FaceTextureInfo, build_voxel_texture_array},
    },
    world::Voxel,
};

#[test]
fn test_build_array() {
    let (img, _reg) = build_voxel_texture_array();
    let depth = img.texture_descriptor.size.depth_or_array_layers;
    let data_len = img.data.as_ref().map_or(0, |d| d.len());
    println!("TOTAL LAYERS: {}", depth);
    println!("DATA LEN: {}", data_len);
    println!("EXPECTED DATA LEN: {}", depth as usize * 16 * 16 * 4);
    assert!(
        depth <= 2048,
        "Texture array depth must be <= 2048 to fit GPU limits, got {}",
        depth
    );
}

#[test]
fn test_oriented_pillar_textures() {
    let mut registry = VoxelTextureRegistry::default();
    let side = FaceTextureInfo {
        start_layer: 10,
        variant_count: 1,
        frame_count: 1,
    };
    let top = FaceTextureInfo {
        start_layer: 20,
        variant_count: 1,
        frame_count: 1,
    };
    let bottom = FaceTextureInfo {
        start_layer: 21,
        variant_count: 1,
        frame_count: 1,
    };

    registry.register_multi_face(Voxel::OakLog, side, top, bottom);

    // Axis Y (orientation 0): Top/Bottom get caps, lateral get side
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveY, 0)
            .0,
        20
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeY, 0)
            .0,
        21
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveX, 0)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveZ, 0)
            .0,
        10
    );

    // Axis X (orientation 1): PositiveX/NegativeX get caps, Y/Z get side
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveX, 1)
            .0,
        20
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeX, 1)
            .0,
        21
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveY, 1)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeY, 1)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveZ, 1)
            .0,
        10
    );

    // Axis Z (orientation 2): PositiveZ/NegativeZ get caps, X/Y get side
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveZ, 2)
            .0,
        20
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeZ, 2)
            .0,
        21
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveX, 2)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::NegativeX, 2)
            .0,
        10
    );
    assert_eq!(
        registry
            .get_face_texture_info_oriented(Voxel::OakLog, IVec3::ZERO, FaceDirection::PositiveY, 2)
            .0,
        10
    );
}

#[test]
fn test_pillar_and_leaves_textures_and_tinting() {
    // 1. Pillar blocks must have distinct bark/side textures, never top rings on sides!
    assert_eq!(Voxel::OakLog.side_texture_name(), Some("tree_oak_bark"));
    assert_eq!(Voxel::OakLog.top_texture_name(), Some("tree_oak_log"));
    assert_eq!(Voxel::BoneBlock.side_texture_name(), Some("bio_bone_side"));
    assert_eq!(Voxel::BoneBlock.top_texture_name(), Some("bio_bone_top"));
    assert_eq!(Voxel::FleshBlock.side_texture_name(), Some("bio_flesh_side"));
    assert_eq!(Voxel::FleshBlock.top_texture_name(), Some("bio_flesh_top"));
    assert_eq!(Voxel::BambooStalk.side_texture_name(), Some("tree_bamboo_stalk"));
    assert_eq!(Voxel::BambooStalk.top_texture_name(), Some("tree_bamboo_stalk_top"));

    // 2. Dead leaves, seasonal maple leaves, cherry, and aspen must NOT be tinted!
    assert!(!Voxel::OakLeavesDead.is_tinted());
    assert!(!Voxel::BirchLeavesDead.is_tinted());
    assert!(!Voxel::PineLeavesDead.is_tinted());
    assert!(!Voxel::MapleLeavesOrange.is_tinted());
    assert!(!Voxel::MapleLeavesRed.is_tinted());
    assert!(!Voxel::MapleLeavesYellow.is_tinted());
    assert!(!Voxel::MapleLeavesDead.is_tinted());
    assert!(!Voxel::CherryLeaves.is_tinted());
    assert!(!Voxel::RedwoodLeaves.is_tinted());

    // 3. Living grayscale canopy species MUST be tinted
    assert!(Voxel::OakLeaves.is_tinted());
    assert!(Voxel::BirchLeaves.is_tinted());
    assert!(Voxel::PalmLeaves.is_tinted());
    assert!(Voxel::AcaciaLeaves.is_tinted());

    // 4. Bloody blocks must have the bloody overlay
    assert_eq!(Voxel::BloodyBone.overlay_texture_name(), Some("bloody_overlay"));
    assert_eq!(Voxel::BloodyBone.side_texture_name(), Some("bio_bone_side"));
    assert_eq!(Voxel::BloodyBone.top_texture_name(), Some("bio_bone_top"));
    assert_eq!(Voxel::BloodyBrain.overlay_texture_name(), Some("bloody_overlay"));
    assert_eq!(Voxel::BloodyFlesh.overlay_texture_name(), Some("bloody_overlay"));
}



#[test]
fn test_build_mask_texture_array() {
    use voxel_game_engine::meshing::textures::build_mask_texture_array;
    let img = build_mask_texture_array();
    let depth = img.texture_descriptor.size.depth_or_array_layers;
    assert_eq!(depth, 85, "Mask texture array must have 85 layers (5 categories x 17 masks)");
    assert_eq!(img.texture_descriptor.size.width, 16);
    assert_eq!(img.texture_descriptor.size.height, 16);
}

#[test]
fn test_connected_textures_priority_and_transition() {
    use voxel_game_engine::{
        meshing::{
            greedy::compute_block_transition,
            textures::build_voxel_texture_array,
        },
        world::{Chunk, VoxelWorld},
    };

    // 1. Verify priority levels:
    // Snow(9) > Grass(8) > Dirt(7) > Gravel(6) > SandCommon(5) > OtherSands(4) > Clay(3) > Rock(2) > Wood(1) > None(0)
    assert_eq!(Voxel::LoamSnow.transition_priority(), 9);
    assert_eq!(Voxel::SnowBlock.transition_priority(), 9);
    assert_eq!(Voxel::LoamGrass.transition_priority(), 8);
    assert_eq!(Voxel::LoamDirt.transition_priority(), 7);
    assert_eq!(Voxel::GravelSandy.transition_priority(), 6);
    assert_eq!(Voxel::SandCommon.transition_priority(), 5);
    assert_eq!(Voxel::SandDune.transition_priority(), 4);
    assert_eq!(Voxel::ClayBlue.transition_priority(), 3);
    assert_eq!(Voxel::Granite.transition_priority(), 2);
    assert_eq!(Voxel::OakPlanks.transition_priority(), 1);
    assert_eq!(Voxel::LiquidWater.transition_priority(), 0);
    assert_eq!(Voxel::Air.transition_priority(), 0);

    // 2. Setup a test world: Sand with Grass neighbor
    let mut world = VoxelWorld::default();
    let mut chunk = Chunk::new();

    // Center sand block at (5, 5, 5)
    chunk.set(5, 5, 5, Voxel::SandCommon);

    // North neighbor at (5, 5, 4) is LoamGrass (priority 5 > sand 3)
    chunk.set(5, 5, 4, Voxel::LoamGrass);

    world.insert_chunk(IVec3::ZERO, chunk);
    world.insert_chunk(IVec3::new(1, 0, 0), Chunk::new());

    let (_, reg) = build_voxel_texture_array();
    use voxel_game_engine::meshing::greedy::FaceDirection;

    let (ov_layer, mask_idx, packed_tint) = compute_block_transition(&world, IVec3::new(5, 5, 5), Voxel::SandCommon, FaceDirection::PositiveY, &reg);

    // North only: bitmask 1 -> mask file 15 -> mask index 16
    assert_eq!(mask_idx, 16, "North neighbor must map to mask index 16 (mask 15)");
    assert!(ov_layer > 0);
    assert!(packed_tint > 0, "Grass overlay must carry packed biome grass tint");

    // 3. Snow next to Grass: Snow (9) must spill onto Grass (8)
    world.set_voxel(IVec3::new(6, 5, 4), Voxel::LoamSnow);
    let (snow_ov, grass_snow_mask, snow_tint) = compute_block_transition(&world, IVec3::new(5, 5, 4), Voxel::LoamGrass, FaceDirection::PositiveY, &reg);
    assert!(grass_snow_mask >= 52 && grass_snow_mask <= 68, "Snow mask must be in snow category (52..=68)");
    assert_eq!(snow_ov, 0, "Snow is pre-colored in mask, does not sample voxel texture");
    assert_eq!(snow_tint, 0);

    // 4. Priority hierarchy: Gravel (6) > SandCommon (5) > SandDune (4) > ClayBlue (3) > Granite (2)
    world.set_voxel(IVec3::new(5, 5, 6), Voxel::GravelSandy);
    world.set_voxel(IVec3::new(5, 5, 7), Voxel::SandCommon);
    let (_, sand_mask, _) = compute_block_transition(&world, IVec3::new(5, 5, 7), Voxel::SandCommon, FaceDirection::PositiveY, &reg);
    assert!(sand_mask >= 18 && sand_mask <= 34, "Gravel (6) must spill onto SandCommon (5)");

    world.set_voxel(IVec3::new(5, 5, 8), Voxel::SandDune);
    let (_, dune_mask, _) = compute_block_transition(&world, IVec3::new(5, 5, 8), Voxel::SandDune, FaceDirection::PositiveY, &reg);
    assert!(dune_mask >= 18 && dune_mask <= 34, "SandCommon (5) must spill onto SandDune (4)");

    world.set_voxel(IVec3::new(5, 5, 9), Voxel::ClayBlue);
    let (_, clay_mask, _) = compute_block_transition(&world, IVec3::new(5, 5, 9), Voxel::ClayBlue, FaceDirection::PositiveY, &reg);
    assert!(clay_mask >= 18 && clay_mask <= 34, "SandDune (4) must spill onto ClayBlue (3)");

    // 5. Mossy rocks use Rocks mask category (35..=51), NOT Moss mask (which is only for MossForest)
    world.set_voxel(IVec3::new(5, 5, 10), Voxel::MossyGranite);
    world.set_voxel(IVec3::new(5, 5, 11), Voxel::OakPlanks);
    let (rock_ov, wood_rock_mask, _) = compute_block_transition(&world, IVec3::new(5, 5, 11), Voxel::OakPlanks, FaceDirection::PositiveY, &reg);
    assert!(wood_rock_mask >= 35 && wood_rock_mask <= 51, "Mossy rock must use Rocks mask category (35..=51)");
    assert!(rock_ov > 0, "Mossy rock receives rock texture via code");

    // 6. Lateral wall transitions (e.g. PositiveX face on a vertical cave wall)
    // Granite next to MossyGranite on wall: MossyGranite has higher rock rank and spills over Granite
    world.set_voxel(IVec3::new(10, 5, 5), Voxel::Granite);
    world.set_voxel(IVec3::new(10, 6, 5), Voxel::MossyGranite); // Block above (+Y) on the wall
    let (lat_ov, lat_mask, _) = compute_block_transition(&world, IVec3::new(10, 5, 5), Voxel::Granite, FaceDirection::PositiveX, &reg);
    assert!(lat_mask >= 35 && lat_mask <= 51, "MossyGranite must spill onto Granite on lateral wall");
    assert!(lat_ov > 0, "Overlay texture for lateral face must be valid");

    // Sand on wall next to Granite: Sand (5) spills over Granite (2)
    world.set_voxel(IVec3::new(10, 5, 6), Voxel::SandCommon);
    let (sand_lat_ov, sand_lat_mask, _) = compute_block_transition(&world, IVec3::new(10, 5, 5), Voxel::Granite, FaceDirection::PositiveX, &reg);
    assert!(sand_lat_mask >= 18 && sand_lat_mask <= 34, "SandCommon must spill onto Granite on lateral wall");
    assert!(sand_lat_ov > 0);

    // 7. Cave blocks participate in connected textures with rock mask category
    assert_eq!(Voxel::CaveCalcite.transition_priority(), 2);
    assert_eq!(Voxel::FracturedCaveCalcite.transition_priority(), 2);
    world.set_voxel(IVec3::new(20, 5, 5), Voxel::CaveCalcite);
    world.set_voxel(IVec3::new(20, 6, 5), Voxel::FracturedCaveCalcite); // Cobbled cave rock has higher sub-tier
    let (cave_ov, cave_mask, _) = compute_block_transition(&world, IVec3::new(20, 5, 5), Voxel::CaveCalcite, FaceDirection::PositiveX, &reg);
    println!("CAVE TEST: cave_mask = {cave_mask}, cave_ov = {cave_ov}");
    assert!(cave_mask >= 35 && cave_mask <= 51, "FracturedCaveCalcite must spill onto CaveCalcite");
    assert!(cave_ov > 0);

    // 8. Outcrop turf blocks on lateral walls act as their underlying block (no snow/grass band on vertical rock/dirt wall)
    world.set_voxel(IVec3::new(25, 5, 5), Voxel::Granite);
    world.set_voxel(IVec3::new(25, 6, 5), Voxel::GraniteSnow); // Snow rock above raw granite
    let (_, snow_wall_mask, _) = compute_block_transition(&world, IVec3::new(25, 5, 5), Voxel::Granite, FaceDirection::PositiveX, &reg);
    assert_eq!(snow_wall_mask, 0, "GraniteSnow above Granite on vertical wall must not spill snow (treated as Granite on Granite)");

    world.set_voxel(IVec3::new(26, 5, 5), Voxel::LoamDirt);
    world.set_voxel(IVec3::new(26, 6, 5), Voxel::LoamGrass); // Grass turf above raw dirt
    let (_, grass_wall_mask, _) = compute_block_transition(&world, IVec3::new(26, 5, 5), Voxel::LoamDirt, FaceDirection::PositiveX, &reg);
    assert_eq!(grass_wall_mask, 0, "LoamGrass above LoamDirt on vertical wall must not spill grass (treated as LoamDirt on LoamDirt)");

    // 9. Floor corner transition on vertical walls:
    // When a wall block touches a floor transition (Sand) at Down, and meets a different solid rock at Right,
    // it includes the corner bit so the floor transition curves up into the corner (Mask 3: Bottom + Right).
    world.set_voxel(IVec3::new(30, 5, 5), Voxel::Granite);
    world.set_voxel(IVec3::new(30, 4, 5), Voxel::SandCommon); // Floor (Down) is Sand
    world.set_voxel(IVec3::new(30, 5, 6), Voxel::Basalt);     // Right is Basalt
    let (_, corner_mask, _) = compute_block_transition(&world, IVec3::new(30, 5, 5), Voxel::Granite, FaceDirection::PositiveX, &reg);
    // Mask 3 is in sand category (18..=34). Specifically: category.base_layer() + 3 + 1 = 17 + 3 + 1 = 21
    assert_eq!(corner_mask, 21, "Floor transition on wall must curve into the corner (Mask 3 = index 21) when meeting a different solid block");
}


#[test]
fn test_slab_and_stair_connected_textures() {
    use voxel_game_engine::{
        meshing::{
            greedy::ChunkMesher,
            textures::build_voxel_texture_array,
        },
        world::{BlockShape, Chunk, VoxelWorld},
    };

    let mut world = VoxelWorld::default();
    let mut chunk = Chunk::new();

    // Place a Sand slab at (5, 5, 5)
    chunk.set(5, 5, 5, Voxel::SandCommon);
    chunk.set_shape(5, 5, 5, BlockShape::Slab, 0);

    // Place a LoamGrass block to the North at (5, 5, 4)
    chunk.set(5, 5, 4, Voxel::LoamGrass);

    // Place a Granite stair at (8, 5, 5) with a LoamGrass block to North
    chunk.set(8, 5, 5, Voxel::Granite);
    chunk.set_shape(8, 5, 5, BlockShape::Stair, 0);
    chunk.set(8, 5, 4, Voxel::LoamGrass);

    world.insert_chunk(IVec3::ZERO, chunk);
    world.insert_chunk(IVec3::new(1, 0, 0), Chunk::new());

    let (_, reg) = build_voxel_texture_array();
    let meshes = ChunkMesher::build_meshes(&world, IVec3::ZERO, &reg);
    let opaque_mesh = meshes.opaque.expect("Opaque mesh must be present");

    // Verify that vertices in the opaque mesh contain connected-texture mask metadata
    use bevy::render::mesh::VertexAttributeValues;
    let uv_b_attr = opaque_mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_UV_1).unwrap();
    let VertexAttributeValues::Float32x2(uv_bs) = uv_b_attr else {
        panic!("UV_1 must be Float32x2");
    };

    let color_attr = opaque_mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_COLOR).unwrap();
    let VertexAttributeValues::Float32x4(colors) = color_attr else {
        panic!("COLOR must be Float32x4");
    };

    // At least one vertex must have a mask applied (uv_b.y >= 16.0) and negative packed alpha (color.a < -0.5)
    let found_mask_face = uv_bs.iter().zip(colors.iter()).any(|(uv_b, color)| {
        uv_b[1] >= 16.0 && color[3] < -0.5
    });

    assert!(found_mask_face, "Slab / Stair top faces connecting to higher priority neighbor must have connected-texture mask vertices");
}
