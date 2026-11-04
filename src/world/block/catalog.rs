use super::data::VOXEL_DATA;
use super::shape::{BlockShape, NaturalRotationMode};
use super::voxel::Voxel;
use bevy::prelude::*;

impl Voxel {
    #[inline(always)]
    pub fn id(self) -> &'static str {
        VOXEL_DATA[self as usize].0
    }

    #[inline(always)]
    pub fn label(self) -> &'static str {
        VOXEL_DATA[self as usize].1
    }

    pub fn from_id(id: &str) -> Option<Self> {
        VOXEL_DATA
            .iter()
            .position(|&(i, _, _)| i == id)
            .and_then(|idx| Self::from_u16(idx as u16))
    }

    #[inline(always)]
    pub fn texture_name(self) -> Option<&'static str> {
        let t = VOXEL_DATA[self as usize].2;
        if t.is_empty() { None } else { Some(t) }
    }

    #[inline(always)]
    pub fn backup_texture_name(self) -> &'static str {
        if self.is_fluid() {
            "null_liquid"
        } else if self.is_cross_billboard() {
            "null_crossmesh"
        } else if self.has_custom_mesh() || self.is_flat_pad_const() || self.is_wall_decal_const() {
            "null_generic"
        } else {
            "null_block"
        }
    }

    pub fn side_texture_name(self) -> Option<&'static str> {
        match self {
            Self::LiquidAcid => Some("liquid_acid_flow"),
            Self::LiquidBlood => Some("liquid_blood_flow"),
            Self::LiquidCoreMagma => Some("liquid_core_magma_flow"),
            Self::LiquidLava => Some("liquid_lava_flow"),
            Self::LiquidMolten => Some("liquid_molten_flow"),
            Self::LiquidOoze => Some("liquid_ooze_flow"),
            Self::LiquidSludge => Some("liquid_sludge_flow"),
            Self::LiquidSpringWater => Some("liquid_spring_water_flow"),
            Self::LiquidTar => Some("liquid_tar_flow"),
            Self::LiquidWater => Some("liquid_water_flow"),

            Self::CalicheGrass => Some("soil_caliche_grass_side"),
            Self::LateriteGrass => Some("soil_laterite_grass_side"),
            Self::LoamGrass => Some("soil_loam_grass_side"),
            Self::LoamMulch => Some("soil_loam_mulch_side"),
            Self::LoamSnow => Some("soil_loam_snow_side"),
            Self::LoamTilled => Some("soil_loam_tilled_side"),
            Self::PeatGrass => Some("soil_peat_grass_side"),
            Self::PeatMulch => Some("soil_peat_mulch_side"),
            Self::PeatSnow => Some("soil_peat_snow_side"),
            Self::PeatTilled => Some("soil_peat_tilled_side"),
            Self::PermafrostGrass => Some("soil_permafrost_grass_side"),
            Self::SiltGrass => Some("soil_silt_grass_side"),
            Self::SiltMulch => Some("soil_silt_mulch_side"),
            Self::SiltSnow => Some("soil_silt_snow_side"),
            Self::SiltTilled => Some("soil_silt_tilled_side"),

            Self::AndesiteGrass => Some("rock_andesite_grass_side"),
            Self::AndesiteSnow => Some("rock_andesite_snow_side"),
            Self::ArgilliteGrass => Some("rock_argillite_grass_side"),
            Self::ArgilliteSnow => Some("rock_argillite_snow_side"),
            Self::BasaltGrass => Some("rock_basalt_grass_side"),
            Self::BasaltSnow => Some("rock_basalt_snow_side"),
            Self::ChalkGrass => Some("rock_chalk_grass_side"),
            Self::ChalkSnow => Some("rock_chalk_snow_side"),
            Self::DolomiteGrass => Some("rock_dolomite_grass_side"),
            Self::DolomiteSnow => Some("rock_dolomite_snow_side"),
            Self::GraniteGrass => Some("rock_granite_grass_side"),
            Self::GraniteSnow => Some("rock_granite_snow_side"),
            Self::LimestoneGrass => Some("rock_limestone_grass_side"),
            Self::LimestoneSnow => Some("rock_limestone_snow_side"),

            Self::AcaciaLog => Some("tree_acacia_bark"),
            Self::BirchLog => Some("tree_birch_bark"),
            Self::CherryLog => Some("tree_cherry_bark"),
            Self::MahoganyLog => Some("tree_mahogany_bark"),
            Self::MangroveLog => Some("tree_mangrove_bark"),
            Self::MapleLog => Some("tree_maple_bark"),
            Self::OakLog => Some("tree_oak_bark"),
            Self::PalmLog => Some("tree_palm_bark"),
            Self::PineLog => Some("tree_pine_bark"),
            Self::RedwoodLog => Some("tree_redwood_bark"),
            Self::WillowLog => Some("tree_willow_bark"),
            Self::YewLog => Some("tree_yew_bark"),
            Self::BambooStalk => Some("tree_bamboo_stalk"),
            Self::BloodyBone | Self::BoneBlock => Some("bio_bone_side"),
            Self::BloodyFlesh | Self::FleshBlock => Some("bio_flesh_side"),
            Self::SeaAnemone => Some("aqua_sea_anemone_side"),
            Self::GeothermalVent => Some("aqua_geothermal_vent_side"),
            Self::Barrel => Some("util_barrel_side"),
            Self::Bookshelf | Self::BookshelfEmpty => Some("util_bookshelf_side"),
            Self::Bag => Some("util_bag_side"),
            Self::Crate => Some("util_crate_side"),
            Self::EmitLampWarm => Some("emit_lamp_warm_side"),
            Self::EmitLampBlue => Some("emit_lamp_blue_side"),
            Self::EmitLampGreen => Some("emit_lamp_green_side"),
            Self::EmitLampRed => Some("emit_lamp_red_side"),
            Self::Basket => Some("util_basket_side"),
            Self::SpiderEgg => Some("nest_spider_egg_side"),
            Self::SilkCocoon => Some("nest_silk_cocoon_side"),
            _ => None,
        }
    }

    pub fn top_texture_name(self) -> Option<&'static str> {
        match self {
            Self::CalicheGrass
            | Self::LateriteGrass
            | Self::LoamGrass
            | Self::PeatGrass
            | Self::PermafrostGrass
            | Self::SiltGrass
            | Self::AndesiteGrass
            | Self::ArgilliteGrass
            | Self::BasaltGrass
            | Self::ChalkGrass
            | Self::DolomiteGrass
            | Self::GraniteGrass
            | Self::LimestoneGrass => Some("soil_grass"),

            Self::LoamSnow
            | Self::PeatSnow
            | Self::SiltSnow
            | Self::AndesiteSnow
            | Self::ArgilliteSnow
            | Self::BasaltSnow
            | Self::ChalkSnow
            | Self::DolomiteSnow
            | Self::GraniteSnow
            | Self::LimestoneSnow => Some("soil_snow"),

            Self::LoamMulch => Some("soil_loam_mulch"),
            Self::PeatMulch => Some("soil_peat_mulch"),
            Self::SiltMulch => Some("soil_silt_mulch"),
            Self::LoamTilled => Some("soil_loam_tilled_top"),
            Self::PeatTilled => Some("soil_peat_tilled_top"),
            Self::SiltTilled => Some("soil_silt_tilled_top"),

            Self::AcaciaLog => Some("tree_acacia_log"),
            Self::BambooStalk => Some("tree_bamboo_stalk_top"),
            Self::BirchLog => Some("tree_birch_log"),
            Self::CherryLog => Some("tree_cherry_log"),
            Self::MahoganyLog => Some("tree_mahogany_log"),
            Self::MangroveLog => Some("tree_mangrove_log"),
            Self::MangroveRoots => Some("tree_mangrove_roots_top"),
            Self::MapleLog => Some("tree_maple_log"),
            Self::OakLog => Some("tree_oak_log"),
            Self::PalmLog => Some("tree_palm_log"),
            Self::PineLog => Some("tree_pine_log"),
            Self::RedwoodLog => Some("tree_redwood_log"),
            Self::WillowLog => Some("tree_willow_log"),
            Self::YewLog => Some("tree_yew_log"),

            Self::GeothermalVent => Some("aqua_geothermal_vent_top"),
            Self::SeaAnemone => Some("aqua_sea_anemone_top"),
            Self::BeeNest => Some("nest_bee_nest_top"),
            Self::BirdNest => Some("nest_bird_nest_top"),
            Self::BloodyFlesh | Self::FleshBlock => Some("bio_flesh_top"),
            Self::BloodyBone | Self::BoneBlock => Some("bio_bone_top"),
            Self::HayNest => Some("nest_hay_nest_top"),
            Self::JellyBlock => Some("organic_jelly_top"),
            Self::SilkCocoon => Some("nest_silk_cocoon_top"),
            Self::SlimeBlock => Some("organic_slime_top"),
            Self::SpiderEgg => Some("nest_spider_egg_top"),
            Self::WaspNest => Some("nest_wasp_nest_top"),
            Self::Bag => Some("util_bag_top"),
            Self::Barrel => Some("util_barrel_top"),
            Self::Basket => Some("util_basket_top"),
            Self::Bookshelf | Self::BookshelfEmpty => Some("util_bookshelf_top"),
            Self::Crate => Some("util_crate_top"),
            Self::EmitLampWarm => Some("emit_lamp_warm_top"),
            Self::EmitLampBlue => Some("emit_lamp_blue_top"),
            Self::EmitLampGreen => Some("emit_lamp_green_top"),
            Self::EmitLampRed => Some("emit_lamp_red_top"),
            Self::VegeCactus => Some("vege_cactus_top"),
            _ => None,
        }
    }

    pub fn bottom_texture_name(self) -> Option<&'static str> {
        match self {
            Self::AcaciaLog => Some("tree_acacia_log"),
            Self::BambooStalk => Some("tree_bamboo_stalk_top"),
            Self::BirchLog => Some("tree_birch_log"),
            Self::CherryLog => Some("tree_cherry_log"),
            Self::MahoganyLog => Some("tree_mahogany_log"),
            Self::MangroveLog => Some("tree_mangrove_log"),
            Self::MangroveRoots => Some("tree_mangrove_roots_top"),
            Self::MapleLog => Some("tree_maple_log"),
            Self::OakLog => Some("tree_oak_log"),
            Self::PalmLog => Some("tree_palm_log"),
            Self::PineLog => Some("tree_pine_log"),
            Self::RedwoodLog => Some("tree_redwood_log"),
            Self::WillowLog => Some("tree_willow_log"),
            Self::YewLog => Some("tree_yew_log"),

            Self::GeothermalVent => Some("aqua_geothermal_vent_bot"),
            Self::VegeCactus => Some("vege_cactus_bottom"),
            Self::SeaAnemone => Some("aqua_sea_anemone_bot"),
            Self::BeeNest => Some("nest_bee_nest_bot"),
            Self::BirdNest => Some("nest_bird_nest_bot"),
            Self::BloodyFlesh | Self::FleshBlock => Some("bio_flesh_top"),
            Self::BloodyBone | Self::BoneBlock => Some("bio_bone_top"),
            Self::HayNest => Some("nest_hay_nest_bot"),
            Self::JellyBlock => Some("organic_jelly_bot"),
            Self::SilkCocoon => Some("nest_silk_cocoon_bot"),
            Self::SlimeBlock => Some("organic_slime_bot"),
            Self::SpiderEgg => Some("nest_spider_egg_bot"),
            Self::WaspNest => Some("nest_wasp_nest_bot"),
            Self::Bag => Some("util_bag_bot"),
            Self::Barrel => Some("util_barrel_bot"),
            Self::Basket => Some("util_basket_bot"),
            Self::Bookshelf | Self::BookshelfEmpty => Some("util_bookshelf_top"),
            Self::Crate => Some("util_crate_bot"),
            Self::EmitLampWarm => Some("emit_lamp_warm_bot"),
            Self::EmitLampBlue => Some("emit_lamp_blue_bot"),
            Self::EmitLampGreen => Some("emit_lamp_green_bot"),
            Self::EmitLampRed => Some("emit_lamp_red_bot"),

            Self::CalicheGrass => Some("soil_caliche_dirt"),
            Self::LateriteGrass => Some("soil_laterite_dirt"),
            Self::LoamGrass
            | Self::LoamSnow
            | Self::LoamMulch
            | Self::LoamTilled => Some("soil_loam_dirt"),
            Self::PeatGrass
            | Self::PeatSnow
            | Self::PeatMulch
            | Self::PeatTilled => Some("soil_peat_dirt"),
            Self::PermafrostGrass => Some("soil_permafrost_dirt"),
            Self::SiltGrass
            | Self::SiltSnow
            | Self::SiltMulch
            | Self::SiltTilled => Some("soil_silt_dirt"),

            Self::AndesiteGrass | Self::AndesiteSnow => Some("rock_andesite"),
            Self::ArgilliteGrass | Self::ArgilliteSnow => Some("rock_argillite"),
            Self::BasaltGrass | Self::BasaltSnow => Some("rock_basalt"),
            Self::ChalkGrass | Self::ChalkSnow => Some("rock_chalk"),
            Self::DolomiteGrass | Self::DolomiteSnow => Some("rock_dolomite"),
            Self::GraniteGrass | Self::GraniteSnow => Some("rock_granite"),
            Self::LimestoneGrass | Self::LimestoneSnow => Some("rock_limestone"),

            Self::FungiAmanitaCap
            | Self::FungiPorciniCap
            | Self::FungiRussulaCap
            | Self::FungiShiitakeCap
            | Self::FungiVerdigrisCap => Some("fungi_inside"),
            _ => None,
        }
    }

    pub fn front_texture_name(self) -> Option<&'static str> {
        match self {
            Self::BeeNest => Some("nest_bee_nest_front"),
            Self::WaspNest => Some("nest_wasp_nest_front"),
            Self::Bookshelf => Some("util_bookshelf_front"),
            Self::BookshelfEmpty => Some("util_bookshelf_front_empty"),
            Self::Bag => Some("util_bag_side_front"),
            _ => None,
        }
    }

    pub fn overlay_texture_name(&self) -> Option<&'static str> {
        match *self {
            Self::CalicheGrass
            | Self::LateriteGrass
            | Self::LoamGrass
            | Self::PeatGrass
            | Self::PermafrostGrass
            | Self::SiltGrass
            | Self::AndesiteGrass
            | Self::ArgilliteGrass
            | Self::BasaltGrass
            | Self::ChalkGrass
            | Self::DolomiteGrass
            | Self::GraniteGrass
            | Self::LimestoneGrass => Some("soil_grass_side_overlay"),

            Self::MossyAndesite
            | Self::MossyArgillite
            | Self::MossyBasalt
            | Self::MossyChalk
            | Self::MossyDolomite
            | Self::MossyGranite
            | Self::MossyLimestone => Some("mossy_rock_overlay"),

            Self::MossyCobbledAndesite
            | Self::MossyCobbledArgillite
            | Self::MossyCobbledBasalt
            | Self::MossyCobbledChalk
            | Self::MossyCobbledDolomite
            | Self::MossyCobbledGranite
            | Self::MossyCobbledLimestone => Some("mossy_cobbled_overlay"),

            Self::BloodyBone | Self::BloodyBrain | Self::BloodyFlesh => Some("bloody_overlay"),
            _ => None,
        }
    }

    pub fn default_shape(self) -> BlockShape {
        if self.is_cross_billboard_const() {
            BlockShape::CrossBillboard
        } else if self.is_flat_pad_const() {
            BlockShape::FlatPad
        } else if self.is_wall_decal_const() {
            BlockShape::WallDecal
        } else if self == Self::VegeCactus {
            BlockShape::Cactus
        } else if self.is_torch_const() {
            BlockShape::Torch
        } else if self.is_basket_const() {
            BlockShape::Basket
        } else if matches!(self, Self::PebbleChert | Self::PebbleFlint | Self::PebbleQuartz) {
            BlockShape::Pebble
        } else if matches!(
            self,
            Self::PebbleChertCluster | Self::PebbleFlintCluster | Self::PebbleQuartzCluster
        ) {
            BlockShape::PebbleCluster
        } else if self.is_rubble_const() {
            BlockShape::Rubble
        } else if self.is_speleothem_const() {
            BlockShape::Speleothem
        } else if matches!(self, Self::BirdNest | Self::HayNest) {
            BlockShape::Slab
        } else if matches!(self, Self::BambooStalk | Self::BambooLeaves) {
            BlockShape::Bamboo
        } else if matches!(self, Self::SpiderEgg | Self::SilkCocoon) {
            BlockShape::Cocoon
        } else {
            BlockShape::Full
        }
    }

    pub fn is_shape_locked(self) -> bool {
        self.has_custom_mesh()
            || self.is_speleothem_const()
            || self.is_flat_pad_const()
            || self.is_wall_decal_const()
            || self == Self::VegeCactus
            || matches!(
                self,
                Self::BirdNest
                    | Self::HayNest
                    | Self::BeeNest
                    | Self::WaspNest
                    | Self::SpiderEgg
                    | Self::SilkCocoon
            )
    }

    #[inline]
    pub const fn is_grass_soil(self) -> bool {
        matches!(
            self,
            Self::CalicheGrass
                | Self::LateriteGrass
                | Self::LoamGrass
                | Self::PeatGrass
                | Self::PermafrostGrass
                | Self::SiltGrass
                | Self::AndesiteGrass
                | Self::ArgilliteGrass
                | Self::BasaltGrass
                | Self::ChalkGrass
                | Self::DolomiteGrass
                | Self::GraniteGrass
                | Self::LimestoneGrass
        )
    }

    #[inline(always)]
    pub fn is_tinted_flora(self) -> bool {
        matches!(
            self,
            Self::VegeWildgrass
                | Self::VegeTallWildgrassBottom
                | Self::VegeTallWildgrassTop
                | Self::VegeFern
                | Self::VegeBracken
                | Self::VegeTallFernBottom
                | Self::VegeTallFernTop
                | Self::VegeTallJungleFernBottom
                | Self::VegeTallJungleFernTop
                | Self::VegeBush
                | Self::VegeVines
                | Self::FlowerClover
        )
    }

    pub fn is_tinted(self) -> bool {
        self.is_tinted_flora()
            || matches!(
                self,
                Self::OakLeaves
                    | Self::BirchLeaves
                    | Self::PalmLeaves
                    | Self::AcaciaLeaves
                    | Self::MahoganyLeaves
                    | Self::MangroveLeaves
                    | Self::WillowLeaves
                    | Self::PineLeaves
                    | Self::YewLeaves
                    | Self::WaterOccupied
                    | Self::LiquidWater
                    | Self::CalicheGrass
                    | Self::LateriteGrass
                    | Self::LoamGrass
                    | Self::PeatGrass
                    | Self::PermafrostGrass
                    | Self::SiltGrass
                    | Self::AndesiteGrass
                    | Self::ArgilliteGrass
                    | Self::BasaltGrass
                    | Self::ChalkGrass
                    | Self::DolomiteGrass
                    | Self::GraniteGrass
                    | Self::LimestoneGrass
            )
    }

    #[inline]
    pub fn is_axis_aligned(self) -> bool {
        matches!(
            self,
            Self::AcaciaLog
                | Self::BirchLog
                | Self::CherryLog
                | Self::MahoganyLog
                | Self::MangroveLog
                | Self::MapleLog
                | Self::OakLog
                | Self::PalmLog
                | Self::PineLog
                | Self::RedwoodLog
                | Self::WillowLog
                | Self::YewLog
                | Self::AcaciaWood
                | Self::BirchWood
                | Self::CherryWood
                | Self::MahoganyWood
                | Self::MangroveWood
                | Self::MapleWood
                | Self::OakWood
                | Self::PalmWood
                | Self::PineWood
                | Self::RedwoodWood
                | Self::WillowWood
                | Self::YewWood
                | Self::BloodyBone
                | Self::BoneBlock
                | Self::BloodyFlesh
                | Self::FleshBlock
        )
    }

    #[inline]
    pub fn is_pillar_block(self) -> bool {
        self.is_axis_aligned()
    }

    pub fn natural_rotation_mode(self) -> NaturalRotationMode {
        if self.is_empty()
            || self.is_fluid()
            || self.is_transparent()
            || self.is_leaves()
            || self.is_painted()
            || self.is_light()
            || self.is_axis_aligned()
            || self.has_custom_mesh()
        {
            return NaturalRotationMode::None;
        }

        let id = self as u16;

        const TREES_START: u16 = Voxel::AcaciaLeaves as u16;
        const TREES_END: u16 = Voxel::YewWood as u16;
        if matches!(id, TREES_START..=TREES_END) {
            return NaturalRotationMode::None;
        }

        const ARCH_START: u16 = Voxel::AndesiteBricks as u16;
        const ARCH_END: u16 = Voxel::WoolYellow as u16;
        if matches!(id, ARCH_START..=ARCH_END) {
            return NaturalRotationMode::None;
        }

        // Soils with grass/snow/mulch/tilled overlays on side (keep vertical side upright, mirror U only)
        if self.is_grass_soil()
            || matches!(
                self,
                Self::LoamSnow
                    | Self::PeatSnow
                    | Self::SiltSnow
                    | Self::AndesiteSnow
                    | Self::ArgilliteSnow
                    | Self::BasaltSnow
                    | Self::ChalkSnow
                    | Self::DolomiteSnow
                    | Self::GraniteSnow
                    | Self::LimestoneSnow
                    | Self::LoamMulch
                    | Self::PeatMulch
                    | Self::SiltMulch
                    | Self::LoamTilled
                    | Self::PeatTilled
                    | Self::SiltTilled
            )
        {
            return NaturalRotationMode::SoilSide;
        }

        const SOILS_START: u16 = Voxel::CalicheCoarse as u16;
        const SOILS_END: u16 = Voxel::SnowBlock as u16;
        const ROCKS_START: u16 = Voxel::Andesite as u16;
        const ROCKS_END: u16 = Voxel::Slate as u16;
        const ORES_START: u16 = Voxel::GemDiamondRough as u16;
        const ORES_END: u16 = Voxel::OreUraniumRaw as u16;
        const CAVERNS_START: u16 = Voxel::CaveCalcite as u16;
        const CAVERNS_END: u16 = Voxel::SpeleothemSaltStraw as u16;

        if matches!(
            id,
            SOILS_START..=SOILS_END
                | ROCKS_START..=ROCKS_END
                | ORES_START..=ORES_END
                | CAVERNS_START..=CAVERNS_END
        ) {
            return NaturalRotationMode::Isotropic;
        }

        NaturalRotationMode::None
    }

    pub const PAINT_PALETTE: [[f32; 4]; 16] = [
        [0.220, 0.224, 0.235, 1.0], // 0: Black (#38393C)
        [0.275, 0.447, 0.722, 1.0], // 1: Blue (#4672B8)
        [0.510, 0.357, 0.247, 1.0], // 2: Brown (#825B3F)
        [0.196, 0.278, 0.478, 1.0], // 3: Dark Blue (#32477A)
        [0.365, 0.384, 0.404, 1.0], // 4: Dark Grey (#5D6267)
        [0.310, 0.518, 0.267, 1.0], // 5: Green (#4F8444)
        [0.686, 0.706, 0.722, 1.0], // 6: Light Grey (#AFB4B8)
        [0.933, 0.725, 0.776, 1.0], // 7: Light Pink (#EEB9C6)
        [0.502, 0.737, 0.318, 1.0], // 8: Lime (#80BC51)
        [0.863, 0.506, 0.224, 1.0], // 9: Orange (#DC8139)
        [0.855, 0.467, 0.584, 1.0], // 10: Pink (#DA7795)
        [0.525, 0.341, 0.635, 1.0], // 11: Purple (#8657A2)
        [0.753, 0.267, 0.247, 1.0], // 12: Red (#C0443F)
        [0.259, 0.635, 0.624, 1.0], // 13: Turquoise (#42A29F)
        [0.957, 0.953, 0.937, 1.0], // 14: White (#F4F3EF)
        [0.910, 0.757, 0.275, 1.0], // 15: Yellow (#E8C146)
    ];

    #[inline]
    pub const fn is_painted(self) -> bool {
        self.paint_color().is_some()
    }

    pub const fn paint_color(self) -> Option<[f32; 4]> {
        const BRICK_START: u16 = Voxel::BrickBlack as u16;
        const BRICK_END: u16 = Voxel::BrickYellow as u16;
        const FABRIC_START: u16 = Voxel::FabricBlack as u16;
        const FABRIC_END: u16 = Voxel::FabricYellow as u16;
        const PLASTER_START: u16 = Voxel::PlasterBlack as u16;
        const PLASTER_END: u16 = Voxel::PlasterYellow as u16;
        const SHINGLES_START: u16 = Voxel::ShinglesBlack as u16;
        const SHINGLES_END: u16 = Voxel::ShinglesYellow as u16;
        const WOOL_START: u16 = Voxel::WoolBlack as u16;
        const WOOL_END: u16 = Voxel::WoolYellow as u16;

        let id = self as u16;
        let color_idx = match id {
            BRICK_START..=BRICK_END => id - BRICK_START,
            FABRIC_START..=FABRIC_END => id - FABRIC_START,
            PLASTER_START..=PLASTER_END => id - PLASTER_START,
            SHINGLES_START..=SHINGLES_END => id - SHINGLES_START,
            WOOL_START..=WOOL_END => id - WOOL_START,
            _ => return None,
        };
        Some(Self::PAINT_PALETTE[color_idx as usize])
    }
    pub fn tint_color(self) -> [f32; 4] {
        if let Some(paint) = self.paint_color() {
            return paint;
        }
        match self {
            Self::LiquidWater | Self::WaterOccupied => [0.35, 0.65, 0.92, 1.0],
            Self::OakLeaves => [0.60, 1.15, 0.35, 1.0],
            Self::BirchLeaves => [0.85, 1.25, 0.40, 1.0],
            Self::PineLeaves => [0.40, 0.90, 0.55, 1.0],
            Self::AcaciaLeaves => [0.72, 0.92, 0.28, 1.0],
            Self::MahoganyLeaves => [0.45, 1.00, 0.40, 1.0],
            Self::MangroveLeaves => [0.40, 0.95, 0.42, 1.0],
            Self::PalmLeaves => [0.55, 1.05, 0.30, 1.0],
            Self::WillowLeaves => [0.52, 0.88, 0.48, 1.0],
            Self::YewLeaves => [0.28, 0.70, 0.38, 1.0],
            Self::CalicheGrass | Self::LateriteGrass => [0.72, 0.85, 0.30, 1.0],
            Self::LoamGrass
            | Self::PeatGrass
            | Self::PermafrostGrass
            | Self::SiltGrass
            | Self::AndesiteGrass
            | Self::ArgilliteGrass
            | Self::BasaltGrass
            | Self::ChalkGrass
            | Self::DolomiteGrass
            | Self::GraniteGrass
            | Self::LimestoneGrass => [0.55, 0.94, 0.42, 1.0],
            s if s.is_tinted_flora() => [0.55, 0.94, 0.42, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn tint_color_at(self, world_voxel: IVec3) -> [f32; 4] {
        if let Some(paint) = self.paint_color() {
            return paint;
        }
        if !self.is_tinted() {
            return [1.0, 1.0, 1.0, 1.0];
        }
        crate::generation::sample_blended_biome_color(
            self,
            world_voxel.x as f32,
            world_voxel.z as f32,
            1337,
        )
    }

    pub fn fallback_color(self) -> [u8; 4] {
        if let Some(paint) = self.paint_color() {
            return [
                (paint[0] * 255.0).round().clamp(0.0, 255.0) as u8,
                (paint[1] * 255.0).round().clamp(0.0, 255.0) as u8,
                (paint[2] * 255.0).round().clamp(0.0, 255.0) as u8,
                255,
            ];
        }
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => [0, 0, 0, 0],
            Self::LiquidWater | Self::LiquidSpringWater => [60, 140, 220, 255],
            Self::LiquidLava | Self::LiquidCoreMagma | Self::LiquidMolten | Self::Magma => {
                [230, 100, 20, 255]
            }
            Self::LiquidAcid => [120, 230, 40, 255],
            Self::LiquidBlood => [180, 20, 20, 255],
            Self::LiquidOoze => [130, 50, 160, 255],
            Self::LiquidSludge => [70, 80, 50, 255],
            Self::LiquidTar => [25, 25, 30, 255],
            // Snow & Cryology variants
            Self::SnowBlock
            | Self::LoamSnow
            | Self::PeatSnow
            | Self::SiltSnow
            | Self::PathSnowy
            | Self::GravelSnowy
            | Self::AndesiteSnow
            | Self::ArgilliteSnow
            | Self::BasaltSnow
            | Self::ChalkSnow
            | Self::DolomiteSnow
            | Self::GraniteSnow
            | Self::LimestoneSnow => [245, 248, 252, 255],
            Self::PermafrostIcy => [210, 225, 240, 255],
            Self::FrostBlueIce | Self::FrostBlueIceCracked => [120, 180, 240, 255],
            Self::FrostIce | Self::FrostIceFragile | Self::FrostIcePacked => [180, 215, 245, 255],
            Self::FrostIceBlack | Self::FrostIceBlackFragile | Self::FrostIceBlackPacked => {
                [60, 70, 85, 255]
            }

            // Sand variants
            Self::SandCommon => [222, 202, 148, 255],
            Self::SandDune => [228, 188, 120, 255],
            Self::SandRed => [194, 102, 62, 255],
            Self::SandWhite => [242, 238, 222, 255],
            Self::GravelSandy | Self::PathSandy => [204, 184, 138, 255],

            // Sandstones
            Self::SandstoneCommon | Self::CobbledSandstoneCommon => [212, 190, 138, 255],
            Self::SandstoneDune | Self::CobbledSandstoneDune => [218, 178, 112, 255],
            Self::SandstoneRed | Self::CobbledSandstoneRed => [184, 92, 54, 255],
            Self::SandstoneWhite | Self::CobbledSandstoneWhite => [232, 228, 212, 255],

            // Clay variants
            Self::ClayBlue => [120, 140, 155, 255],
            Self::ClayRed => [170, 85, 65, 255],
            Self::ClayWhite => [220, 220, 215, 255],
            Self::ClayYellow => [195, 160, 90, 255],

            // Mulch variants
            Self::LoamMulch => [65, 45, 30, 255],
            Self::PeatMulch => [45, 35, 28, 255],
            Self::SiltMulch => [80, 65, 50, 255],

            // Mud variants
            Self::LoamMud => [75, 55, 40, 255],
            Self::PeatMud => [50, 38, 30, 255],
            Self::SiltMud => [90, 75, 60, 255],

            // Dirt and soil variants
            Self::LoamDirt | Self::LoamCoarse | Self::LoamRooted => [110, 80, 55, 255],
            Self::LoamTilled => [95, 68, 45, 255],
            Self::PeatDirt | Self::PeatCoarse | Self::PeatRooted => [70, 52, 40, 255],
            Self::PeatTilled => [60, 45, 35, 255],
            Self::SiltDirt | Self::SiltCoarse | Self::SiltRooted => [135, 110, 80, 255],
            Self::SiltTilled => [115, 95, 70, 255],
            Self::CalicheDirt | Self::CalicheCoarse | Self::CalicheCracked => [200, 185, 160, 255],
            Self::LateriteDirt | Self::LateriteCoarse | Self::LateriteHardpan => [150, 60, 45, 255],
            Self::PermafrostDirt | Self::PermafrostCoarse => [150, 152, 151, 255],

            // Gravels
            Self::GravelDirty | Self::PathDirty => [120, 100, 80, 255],
            Self::GravelAshy => [105, 105, 105, 255],
            Self::PathStony => [145, 140, 135, 255],

            // Moss
            Self::MossForest => [60, 120, 45, 255],

            s if s.is_water() => [60, 140, 220, 255],
            s if s.is_leaves() => [80, 150, 50, 255],
            s if s.is_transparent() => [200, 225, 235, 160],
            s if s.is_light() => [255, 220, 100, 255],
            _ => [128, 128, 128, 255],
        }
    }

    pub fn light_color(self) -> Color {
        match self {
            Self::EmitLampWarm | Self::EmitTorchWarm => {
                Color::srgb(1.0, 0.82, 0.42)
            }
            Self::EmitTorchFire => Color::srgb(1.0, 0.65, 0.25),
            Self::EmitLampRed | Self::EmitTorchRed => Color::srgb(1.0, 0.25, 0.25),
            Self::EmitLampGreen | Self::EmitTorchGreen => Color::srgb(0.25, 1.0, 0.25),
            Self::EmitLampBlue | Self::EmitTorchBlue => Color::srgb(0.25, 0.50, 1.0),
            Self::LiquidLava
            | Self::LiquidCoreMagma
            | Self::LiquidMolten
            | Self::Magma => Color::srgb(1.0, 0.45, 0.15),
            Self::GeothermalVent => Color::srgb(1.0, 0.55, 0.20),
            Self::CrystalCitrineGlinted => Color::srgb(1.0, 0.85, 0.30),
            Self::CrystalFluoriteGlinted => Color::srgb(0.70, 0.40, 0.95),
            Self::CrystalPinkSaltGlinted => Color::srgb(1.0, 0.60, 0.75),
            Self::CrystalSeleniteGlinted => Color::srgb(0.85, 0.92, 1.0),
            _ => Color::WHITE,
        }
    }
}
