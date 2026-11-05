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
            "null_block"
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
            Self::TreeCharredLog => Some("tree_charred_bark_lit"),
            Self::GeothermalVent => Some("aqua_geothermal_vent_side"),
            Self::Barrel => Some("util_barrel_side"),
            Self::Bookshelf | Self::BookshelfEmpty => Some("util_bookshelf_side"),
            Self::Bag | Self::BagRice | Self::BagSugar => Some("util_bag_side"),
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
            Self::TreeCharredLog => Some("tree_charred_log_lit"),
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
            Self::BagRice => Some("util_bag_top_rice"),
            Self::BagSugar => Some("util_bag_top_suggar"),
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
            Self::TreeCharredLog => Some("tree_charred_log_lit"),
            Self::BeeNest => Some("nest_bee_nest_top"),
            Self::BirdNest => Some("nest_bird_nest_bot"),
            Self::BloodyFlesh | Self::FleshBlock => Some("bio_flesh_top"),
            Self::BloodyBone | Self::BoneBlock => Some("bio_bone_top"),
            Self::HayNest => Some("nest_hay_nest_bot"),
            Self::JellyBlock => Some("organic_jelly_bot"),
            Self::SilkCocoon => Some("nest_silk_cocoon_bot"),
            Self::SlimeBlock => Some("organic_slime_bot"),
            Self::SpiderEgg => Some("nest_spider_egg_bot"),
            Self::WaspNest => Some("nest_wasp_nest_top"),
            Self::Bag | Self::BagRice | Self::BagSugar => Some("util_bag_bot"),
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
            Self::Bag | Self::BagRice | Self::BagSugar => Some("util_bag_side_front"),
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

    #[inline]
    pub const fn has_surface_turf(self) -> bool {
        self.is_grass_soil()
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
            )
    }

    /// Returns the underlying internal base material (dirt or rock) for surface turf outcrop blocks.
    #[inline]
    pub const fn underlying_block(self) -> Self {
        match self {
            Self::CalicheGrass => Self::CalicheDirt,
            Self::LateriteGrass => Self::LateriteDirt,
            Self::LoamGrass | Self::LoamSnow | Self::LoamMulch | Self::LoamTilled => Self::LoamDirt,
            Self::PeatGrass | Self::PeatSnow | Self::PeatMulch | Self::PeatTilled => Self::PeatDirt,
            Self::PermafrostGrass => Self::PermafrostDirt,
            Self::SiltGrass | Self::SiltSnow | Self::SiltMulch | Self::SiltTilled => Self::SiltDirt,
            Self::AndesiteGrass | Self::AndesiteSnow => Self::Andesite,
            Self::ArgilliteGrass | Self::ArgilliteSnow => Self::Argillite,
            Self::BasaltGrass | Self::BasaltSnow => Self::Basalt,
            Self::ChalkGrass | Self::ChalkSnow => Self::Chalk,
            Self::DolomiteGrass | Self::DolomiteSnow => Self::Dolomite,
            Self::GraniteGrass | Self::GraniteSnow => Self::Granite,
            Self::LimestoneGrass | Self::LimestoneSnow => Self::Limestone,
            _ => self,
        }
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
                | Self::TreeCharredLog
                | Self::TreeCharredBark
        )
    }

    #[inline]
    pub const fn is_wood_block(self) -> bool {
        let id = self as u16;
        matches!(id, 139..=202) || matches!(self, Self::TreeCharredLog | Self::TreeCharredBark | Self::TreeCharredPlanks)
    }

    #[inline]
    pub fn transition_priority(self) -> u8 {
        if self.is_empty()
            || self.is_fluid()
            || self.is_transparent()
            || self.is_leaves()
            || self.is_painted()
            || self.is_light()
            || self.has_custom_mesh()
        {
            return 0;
        }
        // Priority 9: Snow Turf & Snow Block (Snow spills over Grass, Dirt, Sand, Stone, Wood)
        if matches!(
            self,
            Self::SnowBlock
                | Self::LoamSnow
                | Self::PeatSnow
                | Self::SiltSnow
                | Self::AndesiteSnow
                | Self::ArgilliteSnow
                | Self::BasaltSnow
                | Self::ChalkSnow
                | Self::DolomiteSnow
                | Self::GraniteSnow
                | Self::LimestoneSnow
        ) {
            return 9;
        }
        // Priority 8: Grass Turf & Mulch (Grass spills over Dirt, Sand, Stone, Wood)
        if self.has_surface_turf() {
            return 8;
        }
        // Priority 7: Dirts / Soils / Mud / Moss (spills over Gravel, Sand, Clay, Rocks, Wood)
        if matches!(
            self,
            Self::CalicheCoarse
                | Self::CalicheCracked
                | Self::CalicheDirt
                | Self::LateriteCoarse
                | Self::LateriteDirt
                | Self::LateriteHardpan
                | Self::LoamCoarse
                | Self::LoamDirt
                | Self::LoamMud
                | Self::LoamMulch
                | Self::LoamRooted
                | Self::LoamTilled
                | Self::MossForest
                | Self::PeatCoarse
                | Self::PeatDirt
                | Self::PeatMud
                | Self::PeatMulch
                | Self::PeatRooted
                | Self::PeatTilled
                | Self::PermafrostCoarse
                | Self::PermafrostDirt
                | Self::PermafrostIcy
                | Self::SiltCoarse
                | Self::SiltDirt
                | Self::SiltMud
                | Self::SiltMulch
                | Self::SiltRooted
                | Self::SiltTilled
        ) {
            return 7;
        }
        // Priority 6: Gravels (Gravel spills over all Sands, Clay, Rocks, Wood)
        if matches!(
            self,
            Self::GravelAshy
                | Self::GravelDirty
                | Self::GravelSandy
                | Self::GravelSnowy
        ) {
            return 6;
        }
        // Priority 5: SandCommon (SandCommon spills over other sands, Clay, Rocks, Wood)
        if self == Self::SandCommon {
            return 5;
        }
        // Priority 4: Other sands (spills over Clay, Rocks, Wood)
        if matches!(
            self,
            Self::SandDune
                | Self::SandRed
                | Self::SandWhite
        ) {
            return 4;
        }
        // Priority 3: Clays (spills over Rocks, Wood)
        if matches!(
            self,
            Self::ClayBlue
                | Self::ClayRed
                | Self::ClayWhite
                | Self::ClayYellow
        ) {
            return 3;
        }
        // Priority 2: Rocks / Stone
        if matches!(
            self,
            Self::Andesite
                | Self::Argillite
                | Self::Basalt
                | Self::Chalk
                | Self::CobbledAndesite
                | Self::CobbledArgillite
                | Self::CobbledBasalt
                | Self::CobbledChalk
                | Self::CobbledDolomite
                | Self::CobbledGabbro
                | Self::CobbledGranite
                | Self::CobbledLimestone
                | Self::CobbledMagma
                | Self::CobbledMantle
                | Self::CobbledMarble
                | Self::CobbledSandstoneCommon
                | Self::CobbledSandstoneDune
                | Self::CobbledSandstoneRed
                | Self::CobbledSandstoneWhite
                | Self::CobbledSlate
                | Self::Dolomite
                | Self::Gabbro
                | Self::Granite
                | Self::Limestone
                | Self::Magma
                | Self::Mantle
                | Self::Marble
                | Self::MossyAndesite
                | Self::MossyArgillite
                | Self::MossyBasalt
                | Self::MossyChalk
                | Self::MossyCobbledAndesite
                | Self::MossyCobbledArgillite
                | Self::MossyCobbledBasalt
                | Self::MossyCobbledChalk
                | Self::MossyCobbledDolomite
                | Self::MossyCobbledGranite
                | Self::MossyCobbledLimestone
                | Self::MossyDolomite
                | Self::MossyGranite
                | Self::MossyLimestone
                | Self::SandstoneCommon
                | Self::SandstoneDune
                | Self::SandstoneRed
                | Self::SandstoneWhite
                | Self::Slate
                | Self::Obsidian
                | Self::CaveCalcite
                | Self::CaveKarst
                | Self::FracturedCaveCalcite
                | Self::FracturedCaveKarst
                | Self::FracturedCaveObsidian
                | Self::BoulderChert
                | Self::BoulderFlint
                | Self::BoulderQuartz
        ) {
            return 2;
        }
        // Priority 1: Wood (Logs, Planks, Woods - Sand, Grass, Snow, Stone all spill over Wood)
        if self.is_wood_block() {
            return 1;
        }
        0
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

            // Rock families (Raw, Cobbled, Bricks, Rubble, and Mossy)
            Self::Andesite | Self::CobbledAndesite | Self::AndesiteBricks | Self::AndesiteRubble => [135, 138, 142, 255],
            Self::MossyAndesite | Self::MossyCobbledAndesite => [118, 135, 110, 255],
            Self::Argillite | Self::CobbledArgillite | Self::ArgilliteBricks | Self::ArgilliteRubble => [125, 78, 68, 255],
            Self::MossyArgillite | Self::MossyCobbledArgillite => [110, 85, 70, 255],
            Self::Basalt | Self::CobbledBasalt | Self::BasaltBricks | Self::BasaltRubble => [50, 48, 52, 255],
            Self::MossyBasalt | Self::MossyCobbledBasalt => [48, 58, 48, 255],
            Self::Chalk | Self::CobbledChalk | Self::ChalkBricks | Self::ChalkRubble => [230, 230, 224, 255],
            Self::MossyChalk | Self::MossyCobbledChalk => [195, 205, 180, 255],
            Self::Dolomite | Self::CobbledDolomite | Self::DolomiteBricks | Self::DolomiteRubble => [195, 182, 172, 255],
            Self::MossyDolomite | Self::MossyCobbledDolomite => [165, 170, 145, 255],
            Self::Gabbro | Self::CobbledGabbro | Self::GabbroBricks | Self::GabbroRubble => [45, 52, 50, 255],
            Self::Granite | Self::CobbledGranite | Self::GraniteBricks | Self::GraniteRubble => [175, 138, 130, 255],
            Self::MossyGranite | Self::MossyCobbledGranite => [145, 135, 110, 255],
            Self::Limestone | Self::CobbledLimestone | Self::LimestoneBricks | Self::LimestoneRubble => [205, 195, 165, 255],
            Self::MossyLimestone | Self::MossyCobbledLimestone => [170, 175, 135, 255],
            Self::Marble | Self::CobbledMarble | Self::MarbleBricks | Self::MarbleRubble => [225, 228, 232, 255],
            Self::Slate | Self::CobbledSlate | Self::SlateBricks | Self::SlateRubble => [75, 82, 92, 255],
            Self::Mantle | Self::CobbledMantle => [22, 18, 28, 255],
            Self::Obsidian | Self::ObsidianSpike | Self::FracturedCaveObsidian => [28, 22, 36, 255],

            // Cave speleothems & karst
            Self::CaveCalcite | Self::FracturedCaveCalcite | Self::SpeleothemCalcite => [210, 215, 220, 255],
            Self::CaveKarst | Self::FracturedCaveKarst | Self::SpeleothemKarst => [130, 125, 120, 255],
            Self::SpeleothemSaltStraw => [235, 230, 225, 255],
            Self::SpeleothemBlueIce => [140, 190, 245, 255],

            // Sandstone Rubble & Bricks
            Self::SandstoneRubble | Self::SandstoneCommonBricks => [212, 190, 138, 255],
            Self::SandstoneDuneRubble | Self::SandstoneDuneBricks => [218, 178, 112, 255],
            Self::SandstoneRedRubble | Self::SandstoneRedBricks => [184, 92, 54, 255],
            Self::SandstoneWhiteRubble | Self::SandstoneWhiteBricks => [232, 228, 212, 255],

            // Outcrop grasses & soils fallback
            Self::AndesiteGrass
            | Self::ArgilliteGrass
            | Self::BasaltGrass
            | Self::ChalkGrass
            | Self::DolomiteGrass
            | Self::GraniteGrass
            | Self::LimestoneGrass
            | Self::CalicheGrass
            | Self::LateriteGrass
            | Self::LoamGrass
            | Self::PeatGrass
            | Self::PermafrostGrass
            | Self::SiltGrass => [110, 160, 65, 255],

            // Wood logs and planks
            Self::AcaciaLog | Self::AcaciaWood => [105, 95, 90, 255],
            Self::AcaciaPlanks => [175, 90, 45, 255],
            Self::BambooStalk => [125, 165, 55, 255],
            Self::BambooPlanks => [190, 175, 105, 255],
            Self::BirchLog | Self::BirchWood => [215, 215, 210, 255],
            Self::BirchPlanks => [200, 185, 140, 255],
            Self::CherryLog | Self::CherryWood => [70, 50, 50, 255],
            Self::CherryPlanks => [215, 160, 170, 255],
            Self::MahoganyLog | Self::MahoganyWood => [95, 55, 45, 255],
            Self::MahoganyPlanks => [140, 75, 60, 255],
            Self::MangroveLog | Self::MangroveWood | Self::MangroveRoots => [80, 50, 40, 255],
            Self::MangrovePlanks => [125, 60, 50, 255],
            Self::MapleLog | Self::MapleWood => [110, 70, 50, 255],
            Self::MaplePlanks => [165, 105, 75, 255],
            Self::OakLog | Self::OakWood => [105, 80, 50, 255],
            Self::OakPlanks => [160, 130, 85, 255],
            Self::PalmLog | Self::PalmWood => [100, 80, 55, 255],
            Self::PalmPlanks => [175, 140, 95, 255],
            Self::PineLog | Self::PineWood => [65, 45, 30, 255],
            Self::PinePlanks => [115, 85, 55, 255],
            Self::RedwoodLog | Self::RedwoodWood => [90, 40, 30, 255],
            Self::RedwoodPlanks => [145, 65, 50, 255],
            Self::WillowLog | Self::WillowWood => [75, 70, 55, 255],
            Self::WillowPlanks => [120, 110, 75, 255],
            Self::YewLog | Self::YewWood => [85, 50, 40, 255],
            Self::YewPlanks => [150, 90, 60, 255],
            Self::TreeCharredLog | Self::TreeCharredBark => [35, 35, 35, 255],
            Self::TreeCharredPlanks => [55, 55, 55, 255],

            // Raw Ores & Minerals
            Self::OreCoalRaw => [35, 35, 38, 255],
            Self::OreCopperRaw => [185, 105, 70, 255],
            Self::OreIronRaw => [165, 135, 115, 255],
            Self::OreGoldRaw => [230, 185, 40, 255],
            Self::OreSilverRaw => [200, 210, 220, 255],
            Self::OreSulfurRaw => [220, 215, 60, 255],
            Self::OreUraniumRaw => [90, 210, 50, 255],
            Self::GemDiamondRough => [90, 220, 225, 255],
            Self::GemEmeraldRough => [40, 190, 80, 255],
            Self::GemJadeRough => [70, 160, 110, 255],
            Self::GemRubyRough => [200, 30, 50, 255],
            Self::GemSapphireRough => [40, 70, 200, 255],
            Self::GemTopazRough => [220, 140, 40, 255],

            // Crystal Geodes
            Self::CrystalAmethystBlock | Self::CrystalAmethystCluster | Self::CrystalAmethystGlinted => [160, 80, 210, 255],
            Self::CrystalAquamarineBlock | Self::CrystalAquamarineCluster | Self::CrystalAquamarineGlinted => [110, 205, 215, 255],
            Self::CrystalCitrineBlock | Self::CrystalCitrineCluster | Self::CrystalCitrineGlinted => [225, 175, 35, 255],
            Self::CrystalFluoriteBlock | Self::CrystalFluoriteCluster | Self::CrystalFluoriteGlinted => [170, 90, 220, 255],
            Self::CrystalPinkSaltBlock | Self::CrystalPinkSaltCluster | Self::CrystalPinkSaltGlinted => [225, 140, 160, 255],
            Self::CrystalSeleniteBlock | Self::CrystalSeleniteCluster | Self::CrystalSeleniteGlinted => [210, 225, 235, 255],

            // Fossils
            Self::FossilAmber => [215, 145, 30, 255],
            Self::FossilBitumen => [40, 38, 42, 255],
            Self::FossilBones | Self::FossilMatrix => [215, 205, 185, 255],

            // Pebbles & Boulders
            Self::PebbleChert | Self::PebbleChertCluster | Self::BoulderChert => [160, 145, 130, 255],
            Self::PebbleFlint | Self::PebbleFlintCluster | Self::BoulderFlint => [65, 70, 75, 255],
            Self::PebbleQuartz | Self::PebbleQuartzCluster | Self::BoulderQuartz => [220, 225, 230, 255],

            // Containers & Storage
            Self::Barrel | Self::Crate | Self::Basket | Self::Bookshelf | Self::BookshelfEmpty => [135, 95, 55, 255],
            Self::Bag | Self::BagRice | Self::BagSugar => [195, 175, 140, 255],

            // Leaves
            Self::AcaciaLeavesDead
            | Self::BirchLeavesDead
            | Self::CherryLeavesDead
            | Self::MahoganyLeavesDead
            | Self::MangroveLeavesDead
            | Self::MapleLeavesDead
            | Self::OakLeavesDead
            | Self::PalmLeavesDead
            | Self::PineLeavesDead
            | Self::RedwoodLeavesDead
            | Self::WillowLeavesDead
            | Self::YewLeavesDead => [125, 95, 65, 255],
            Self::MapleLeavesOrange => [215, 105, 30, 255],
            Self::MapleLeavesRed => [195, 45, 35, 255],
            Self::MapleLeavesYellow => [220, 185, 40, 255],
            Self::CherryLeaves => [235, 165, 185, 255],
            Self::AcaciaLeaves
            | Self::BambooLeaves
            | Self::BirchLeaves
            | Self::MahoganyLeaves
            | Self::MangroveLeaves
            | Self::OakLeaves
            | Self::PalmLeaves
            | Self::PineLeaves
            | Self::RedwoodLeaves
            | Self::WillowLeaves
            | Self::YewLeaves => [75, 140, 45, 255],

            // Corals & Marine Flora
            Self::Coralstone | Self::Reefstone => [165, 130, 120, 255],
            Self::DeadCoral | Self::DeadCoralFan | Self::DyingCoral | Self::DyingCoralFan => [135, 130, 125, 255],
            Self::BrainCoral
            | Self::BrainCoralFan
            | Self::HornCoral
            | Self::HornCoralFan
            | Self::StaghornCoral
            | Self::StaghornCoralFan => [215, 130, 160, 255],
            Self::BubbleCoral | Self::BubbleCoralFan | Self::SunCoral | Self::SunCoralFan => [225, 190, 55, 255],
            Self::FireCoral | Self::FireCoralFan | Self::RedAlgae | Self::Delesseria => [185, 45, 55, 255],
            Self::HeartCoral
            | Self::HeartCoralFan
            | Self::LeafCoral
            | Self::LeafCoralFan
            | Self::ReedCoral
            | Self::ReedCoralFan
            | Self::SpikeCoral
            | Self::SpikeCoralFan
            | Self::ThornCoral
            | Self::ThornCoralFan
            | Self::TubeCoral
            | Self::TubeCoralFan
            | Self::WaxCoral
            | Self::WaxCoralFan
            | Self::ZigzagCoral
            | Self::ZigzagCoralFan => [195, 115, 155, 255],
            Self::BullKelp
            | Self::BullKelpPlant
            | Self::Kelp
            | Self::KelpPlant
            | Self::Seagrass
            | Self::Seaweed
            | Self::Sargassum
            | Self::BoneWorms => [55, 105, 45, 255],

            // Organic, Biological & Nests
            Self::BoneBlock | Self::BloodyBone => [215, 210, 195, 255],
            Self::BrainBlock | Self::BloodyBrain => [200, 120, 140, 255],
            Self::FleshBlock | Self::BloodyFlesh => [165, 55, 55, 255],
            Self::HoneycombBlock | Self::BeeNest | Self::WaspNest => [225, 160, 45, 255],
            Self::SlimeBlock | Self::JellyBlock => [115, 195, 75, 255],
            Self::AntHill | Self::AntHillOpen | Self::TermiteMound | Self::TermiteMoundOpen => [155, 105, 70, 255],
            Self::BirdNest | Self::HayNest => [180, 150, 85, 255],
            Self::SpiderEgg => [210, 205, 190, 255],
            Self::SilkCocoon => [230, 225, 215, 255],
            Self::Cobweb | Self::CobwebArch | Self::CobwebHanging | Self::CobwebStretched => [225, 230, 235, 200],
            Self::PoopBlock => [85, 55, 30, 255],
            Self::DustBlock => [160, 155, 145, 255],

            // Bricks (Colored & Standard)
            Self::CobbledMagma => [140, 50, 25, 255],
            Self::Bricks | Self::BrickRed => [165, 65, 50, 255],
            Self::BrickBlack => [35, 35, 38, 255],
            Self::BrickBlue => [45, 95, 185, 255],
            Self::BrickBrown => [100, 65, 45, 255],
            Self::BrickDarkBlue => [25, 45, 115, 255],
            Self::BrickDarkGrey => [75, 75, 80, 255],
            Self::BrickGreen => [45, 115, 50, 255],
            Self::BrickLightGrey => [170, 175, 180, 255],
            Self::BrickLightPink => [235, 175, 190, 255],
            Self::BrickLime => [115, 190, 45, 255],
            Self::BrickOrange => [220, 115, 35, 255],
            Self::BrickPink => [215, 115, 155, 255],
            Self::BrickPurple => [135, 65, 175, 255],
            Self::BrickTurquoise => [45, 165, 170, 255],
            Self::BrickWhite => [235, 238, 240, 255],
            Self::BrickYellow => [230, 195, 45, 255],

            // Plaster
            Self::Plaster | Self::PlasterWhite => [235, 235, 230, 255],
            Self::PlasterBlack => [30, 30, 32, 255],
            Self::PlasterBlue => [55, 105, 195, 255],
            Self::PlasterBrown => [105, 70, 50, 255],
            Self::PlasterDarkBlue => [30, 50, 125, 255],
            Self::PlasterDarkGrey => [80, 80, 85, 255],
            Self::PlasterGreen => [55, 125, 60, 255],
            Self::PlasterLightGrey => [175, 180, 185, 255],
            Self::PlasterLightPink => [240, 185, 200, 255],
            Self::PlasterLime => [125, 200, 55, 255],
            Self::PlasterOrange => [230, 125, 45, 255],
            Self::PlasterPink => [225, 125, 165, 255],
            Self::PlasterPurple => [145, 75, 185, 255],
            Self::PlasterRed => [185, 55, 45, 255],
            Self::PlasterTurquoise => [55, 175, 180, 255],
            Self::PlasterYellow => [240, 205, 55, 255],

            // Fabric & Wool
            Self::Fabric | Self::Wool | Self::FabricWhite | Self::WoolWhite => [230, 230, 230, 255],
            Self::FabricBlack | Self::WoolBlack => [28, 28, 30, 255],
            Self::FabricBlue | Self::WoolBlue => [50, 90, 180, 255],
            Self::FabricBrown | Self::WoolBrown => [95, 60, 40, 255],
            Self::FabricDarkBlue | Self::WoolDarkBlue => [25, 40, 110, 255],
            Self::FabricDarkGrey | Self::WoolDarkGrey => [70, 70, 75, 255],
            Self::FabricGreen | Self::WoolGreen => [50, 110, 50, 255],
            Self::FabricLightGrey | Self::WoolLightGrey => [165, 170, 175, 255],
            Self::FabricLightPink | Self::WoolLightPink => [235, 170, 185, 255],
            Self::FabricLime | Self::WoolLime => [110, 185, 40, 255],
            Self::FabricOrange | Self::WoolOrange => [215, 110, 30, 255],
            Self::FabricPink | Self::WoolPink => [210, 110, 150, 255],
            Self::FabricPurple | Self::WoolPurple => [130, 60, 170, 255],
            Self::FabricRed | Self::WoolRed => [175, 45, 40, 255],
            Self::FabricTurquoise | Self::WoolTurquoise => [45, 160, 165, 255],
            Self::FabricYellow | Self::WoolYellow => [225, 190, 40, 255],

            // Shingles
            Self::Shingles | Self::ShinglesBlack => [40, 40, 45, 255],
            Self::ShinglesBlue => [50, 85, 160, 255],
            Self::ShinglesBrown => [110, 75, 55, 255],
            Self::ShinglesDarkBlue => [30, 45, 95, 255],
            Self::ShinglesDarkGrey => [65, 65, 70, 255],
            Self::ShinglesGreen => [55, 100, 60, 255],
            Self::ShinglesLightGrey => [150, 155, 160, 255],
            Self::ShinglesLightPink => [215, 165, 175, 255],
            Self::ShinglesLime => [115, 165, 55, 255],
            Self::ShinglesOrange => [195, 100, 40, 255],
            Self::ShinglesPink => [190, 105, 140, 255],
            Self::ShinglesPurple => [120, 65, 145, 255],
            Self::ShinglesRed => [160, 55, 45, 255],
            Self::ShinglesTurquoise => [50, 145, 150, 255],
            Self::ShinglesWhite => [220, 220, 220, 255],
            Self::ShinglesYellow => [210, 175, 50, 255],

            // Lighting & Fire
            Self::ElementFire => [245, 125, 25, 255],
            Self::EmitLampWarm | Self::EmitTorchWarm | Self::EmitTorchFire => [255, 205, 80, 255],
            Self::EmitLampBlue | Self::EmitTorchBlue => [75, 145, 255, 255],
            Self::EmitLampGreen | Self::EmitTorchGreen => [75, 235, 75, 255],
            Self::EmitLampRed | Self::EmitTorchRed => [245, 60, 60, 255],
            Self::GeothermalVent => [85, 70, 65, 255],

            // Fungi & Mushrooms
            Self::FungiAmanita | Self::FungiAmanitaCap => [200, 40, 40, 255],
            Self::FungiPorcini | Self::FungiPorciniCap => [140, 95, 60, 255],
            Self::FungiRussula | Self::FungiRussulaCap => [185, 65, 80, 255],
            Self::FungiShiitake | Self::FungiShiitakeCap => [120, 80, 55, 255],
            Self::FungiVerdigris | Self::FungiVerdigrisCap => [65, 160, 150, 255],
            Self::FungiStem | Self::FungiInside => [225, 220, 205, 255],

            // Wild Vegetation & Grasses
            Self::VegeWildgrass
            | Self::VegeTallWildgrassBottom
            | Self::VegeTallWildgrassTop
            | Self::VegeBluffGrass
            | Self::VegeMossgrass
            | Self::VegeFern
            | Self::VegeBracken
            | Self::VegeTallFernBottom
            | Self::VegeTallFernTop
            | Self::VegeTallJungleFernBottom
            | Self::VegeTallJungleFernTop
            | Self::VegeBush
            | Self::FlowerClover => [95, 170, 55, 255],
            Self::VegeDrygrass
            | Self::VegeSeaOatsBottom
            | Self::VegeSeaOatsTop
            | Self::VegeDeadBush => [185, 165, 110, 255],
            Self::VegeBarrelCactus
            | Self::VegePricklyPearCactus
            | Self::VegeCactus
            | Self::FlowerCactusFlower => [90, 145, 60, 255],
            Self::VegeRedShrub => [175, 60, 45, 255],
            Self::VegeLilyPad => [55, 130, 45, 255],
            Self::VegeCattailBottom | Self::VegeCattailTop => [120, 85, 50, 255],
            Self::VegeVines | Self::VegeHangingMoss | Self::VegeHangingRoots | Self::VegeLichen => [80, 135, 55, 255],
            Self::VegeCaveGrowths
            | Self::VegeGrainyCaveGrowths
            | Self::VegeCaveHyssop
            | Self::VegeCaveVines
            | Self::VegeCaveVinesPlant => [90, 160, 140, 255],

            // Flowers
            Self::FlowerAllium | Self::FlowerLavender => [165, 110, 215, 255],
            Self::FlowerBlueHydrangeaBottom
            | Self::FlowerBlueHydrangeaTop
            | Self::FlowerBlueOrchid
            | Self::FlowerCornflower
            | Self::FlowerIris => [65, 125, 225, 255],
            Self::FlowerDandelion | Self::FlowerWildflowers => [235, 205, 50, 255],
            Self::FlowerFoxgloveBottom
            | Self::FlowerFoxgloveTop
            | Self::FlowerLilacBottom
            | Self::FlowerLilacTop
            | Self::FlowerPeonyBottom
            | Self::FlowerPeonyTop => [210, 115, 185, 255],
            Self::FlowerGardeniaBottom
            | Self::FlowerGardeniaTop
            | Self::FlowerLilyOfTheValley
            | Self::FlowerOxeyeDaisy
            | Self::FlowerWhitePetals => [240, 245, 250, 255],
            Self::FlowerGlobeMallow | Self::FlowerMarigold | Self::FlowerOrangeTulip => [230, 125, 40, 255],
            Self::FlowerHibiscus
            | Self::FlowerPoppy
            | Self::FlowerRoseBushBottom
            | Self::FlowerRoseBushTop => [220, 45, 50, 255],
            Self::FlowerOrchid | Self::FlowerPinkPetals | Self::FlowerPinkTulip => [235, 135, 175, 255],
            Self::FlowerSnapdragonBottom | Self::FlowerSnapdragonTop => [225, 165, 60, 255],
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
