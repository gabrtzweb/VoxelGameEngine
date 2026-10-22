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
            Self::LoamBurrow => Some("soil_loam_grass_side"),
            Self::PeatBurrow => Some("soil_peat_grass_side"),
            Self::SiltBurrow => Some("soil_silt_grass_side"),

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
            Self::AspenLog => Some("tree_aspen_bark"),
            Self::BaobabLog => Some("tree_baobab_bark"),
            Self::BirchLog => Some("tree_birch_bark"),
            Self::CherryLog => Some("tree_cherry_bark"),
            Self::CypressLog => Some("tree_cypress_bark"),
            Self::FirLog => Some("tree_fir_bark"),
            Self::KapokLog => Some("tree_kapok_bark"),
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
            Self::BoneBlock => Some("bio_bone_side"),
            Self::CalcinedBone => Some("bio_calcined_bone_side"),
            Self::BloodyFlesh | Self::FleshBlock => Some("bio_flesh_side"),
            Self::CornBale => Some("deco_corn_bale_side"),
            Self::HayBale => Some("deco_hay_bale_side"),
            Self::RiceBale => Some("deco_rice_bale_side"),
            Self::StrawBale => Some("deco_straw_bale_side"),
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
            Self::AspenLog => Some("tree_aspen_log"),
            Self::BambooStalk => Some("tree_bamboo_stalk_top"),
            Self::BaobabLog => Some("tree_baobab_log"),
            Self::BirchLog => Some("tree_birch_log"),
            Self::CherryLog => Some("tree_cherry_log"),
            Self::CypressLog => Some("tree_cypress_log"),
            Self::FirLog => Some("tree_fir_log"),
            Self::KapokLog => Some("tree_kapok_log"),
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
            Self::PearlBlock => Some("aqua_pearl_top"),
            Self::ScuteBlock => Some("aqua_scute_top"),
            Self::SeaAnemone => Some("aqua_sea_anemone_top"),
            Self::BeeNest => Some("nest_bee_nest_top"),
            Self::BirdNest => Some("nest_bird_nest_top"),
            Self::BloodyFlesh | Self::FleshBlock => Some("bio_flesh_top"),
            Self::BoneBlock => Some("bio_bone_top"),
            Self::CalcinedBone => Some("bio_calcined_bone_top"),
            Self::FurBlock => Some("bio_fur_top"),
            Self::HayNest => Some("nest_hay_nest_top"),
            Self::HoneyBlock => Some("organic_honey_block_top"),
            Self::JellyBlock => Some("organic_jelly_top"),
            Self::LoamBurrow | Self::PeatBurrow | Self::SiltBurrow => {
                Some("nest_burrow_top_colored")
            }
            Self::SilkCocoon => Some("nest_silk_cocoon_top"),
            Self::SlimeBlock => Some("organic_slime_top"),
            Self::SpiderEgg => Some("nest_spider_egg_top"),
            Self::WaspNest => Some("nest_wasp_nest_top"),
            Self::Bag => Some("util_bag_top"),
            Self::Barrel => Some("util_barrel_top"),
            Self::Basket => Some("util_basket_top"),
            Self::Bookshelf | Self::BookshelfEmpty => Some("util_bookshelf_top"),
            Self::CornBale => Some("deco_corn_bale_top"),
            Self::Crate => Some("util_crate_top"),
            Self::EmitLamp => Some("emit_lamp_top"),
            Self::HayBale => Some("deco_hay_bale_top"),
            Self::RiceBale => Some("deco_rice_bale_top"),
            Self::StrawBale => Some("deco_straw_bale_top"),
            _ => None,
        }
    }

    pub fn bottom_texture_name(self) -> Option<&'static str> {
        match self {
            Self::AcaciaLog => Some("tree_acacia_log"),
            Self::AspenLog => Some("tree_aspen_log"),
            Self::BambooStalk => Some("tree_bamboo_stalk_top"),
            Self::BaobabLog => Some("tree_baobab_log"),
            Self::BirchLog => Some("tree_birch_log"),
            Self::CherryLog => Some("tree_cherry_log"),
            Self::CypressLog => Some("tree_cypress_log"),
            Self::FirLog => Some("tree_fir_log"),
            Self::KapokLog => Some("tree_kapok_log"),
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
            Self::PearlBlock => Some("aqua_pearl_bot"),
            Self::ScuteBlock => Some("aqua_scute_bot"),
            Self::SeaAnemone => Some("aqua_sea_anemone_bot"),
            Self::BeeNest => Some("nest_bee_nest_bot"),
            Self::BirdNest => Some("nest_bird_nest_bot"),
            Self::BloodyFlesh | Self::FleshBlock => Some("bio_flesh_top"),
            Self::BoneBlock => Some("bio_bone_top"),
            Self::CalcinedBone => Some("bio_calcined_bone_top"),
            Self::FurBlock => Some("bio_fur_bot"),
            Self::HayNest => Some("nest_hay_nest_bot"),
            Self::HoneyBlock => Some("organic_honey_block_bot"),
            Self::JellyBlock => Some("organic_jelly_bot"),
            Self::SilkCocoon => Some("nest_silk_cocoon_bot"),
            Self::SlimeBlock => Some("organic_slime_bot"),
            Self::SpiderEgg => Some("nest_spider_egg_bot"),
            Self::WaspNest => Some("nest_wasp_nest_bot"),
            Self::Bag => Some("util_bag_bot"),
            Self::Barrel => Some("util_barrel_bot"),
            Self::Basket => Some("util_basket_bot"),
            Self::Bookshelf | Self::BookshelfEmpty => Some("util_bookshelf_top"),
            Self::CornBale => Some("deco_corn_bale_bottom"),
            Self::Crate => Some("util_crate_bot"),
            Self::HayBale | Self::StrawBale => Some("deco_hay_bale_top"),
            Self::RiceBale => Some("deco_rice_bale_bottom"),

            Self::CalicheGrass => Some("soil_caliche_dirt"),
            Self::LateriteGrass => Some("soil_laterite_dirt"),
            Self::LoamGrass
            | Self::LoamSnow
            | Self::LoamMulch
            | Self::LoamTilled
            | Self::LoamBurrow => Some("soil_loam_dirt"),
            Self::PeatGrass
            | Self::PeatSnow
            | Self::PeatMulch
            | Self::PeatTilled
            | Self::PeatBurrow => Some("soil_peat_dirt"),
            Self::PermafrostGrass => Some("soil_permafrost_dirt"),
            Self::SiltGrass
            | Self::SiltSnow
            | Self::SiltMulch
            | Self::SiltTilled
            | Self::SiltBurrow => Some("soil_silt_dirt"),

            Self::AndesiteGrass | Self::AndesiteSnow => Some("rock_andesite"),
            Self::ArgilliteGrass | Self::ArgilliteSnow => Some("rock_argillite"),
            Self::BasaltGrass | Self::BasaltSnow => Some("rock_basalt"),
            Self::ChalkGrass | Self::ChalkSnow => Some("rock_chalk"),
            Self::DolomiteGrass | Self::DolomiteSnow => Some("rock_dolomite"),
            Self::GraniteGrass | Self::GraniteSnow => Some("rock_granite"),
            Self::LimestoneGrass | Self::LimestoneSnow => Some("rock_limestone"),

            Self::FungiAmanitaCap
            | Self::FungiChanterelleCap
            | Self::FungiIndigoCap
            | Self::FungiMycenaCap
            | Self::FungiPorciniCap
            | Self::FungiRussulaCap
            | Self::FungiShiitakeCap
            | Self::FungiVerdigrisCap => Some("fungi_inside"),
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

            Self::GemDiamondBasalt | Self::GemDiamondGabbro => Some("gem_diamond_overlay"),
            Self::GemEmeraldMarble | Self::GemEmeraldSlate => Some("gem_emerald_overlay"),
            Self::GemOnyxSlate => Some("gem_onyx_overlay"),
            Self::GemOpalSandstoneWhite => Some("gem_opal_overlay"),

            Self::GemVeinAndesite
            | Self::GemVeinBasalt
            | Self::GemVeinDolomite
            | Self::GemVeinGabbro
            | Self::GemVeinGranite
            | Self::GemVeinMarble
            | Self::GemVeinSlate => Some("gem_vein_overlay"),

            Self::OreCinnabarBasalt => Some("ore_cinnabar_overlay"),

            Self::OreCoalArgillite
            | Self::OreCoalChalk
            | Self::OreCoalDolomite
            | Self::OreCoalGranite
            | Self::OreCoalLimestone
            | Self::OreCoalSandstoneCommon => Some("ore_coal_overlay"),

            Self::OreCopperAndesite
            | Self::OreCopperDolomite
            | Self::OreCopperGranite
            | Self::OreCopperSandstoneCommon
            | Self::OreCopperSandstoneRed
            | Self::OreCopperSandstoneWhite => Some("ore_copper_overlay"),

            Self::OreGoldGabbro
            | Self::OreGoldGranite
            | Self::OreGoldSandstoneRed
            | Self::OreGoldSlate => Some("ore_gold_overlay"),

            Self::OreIronAndesite
            | Self::OreIronBasalt
            | Self::OreIronDolomite
            | Self::OreIronGabbro
            | Self::OreIronGranite
            | Self::OreIronSlate => Some("ore_iron_overlay"),

            Self::OreLazuriteLimestone | Self::OreLazuriteMarble => Some("ore_lazurite_overlay"),

            Self::OrePlatinumBasalt | Self::OrePlatinumGabbro => Some("ore_platinum_overlay"),

            Self::OreSilverAndesite | Self::OreSilverMarble | Self::OreSilverSlate => {
                Some("ore_silver_overlay")
            }

            Self::OreSulfurBasalt => Some("ore_sulfur_overlay"),

            Self::OreTinAndesite
            | Self::OreTinChalk
            | Self::OreTinDolomite
            | Self::OreTinGranite => Some("ore_tin_overlay"),

            Self::OreUraniumGabbro | Self::OreUraniumSlate => Some("ore_uranium_overlay"),

            Self::OreZincAndesite
            | Self::OreZincArgillite
            | Self::OreZincDolomite
            | Self::OreZincSlate => Some("ore_zinc_overlay"),

            Self::AlgaeReefstone => Some("algae_dead_overlay"),
            Self::BarnaclesReefstone => Some("barnacles_overlay"),
            Self::BloodyBrain | Self::BloodyFlesh => Some("bloody_overlay"),
            _ => None,
        }
    }

    pub fn default_shape(self) -> BlockShape {
        if self.is_cross_billboard_const() {
            BlockShape::CrossBillboard
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
        } else if matches!(self, Self::BambooStalk | Self::BambooLeaves) {
            BlockShape::Bamboo
        } else {
            BlockShape::Full
        }
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
                | Self::LoamBurrow
                | Self::PeatBurrow
                | Self::SiltBurrow
        )
    }

    pub fn is_tinted(self) -> bool {
        matches!(
            self,
            Self::OakLeaves
                | Self::BirchLeaves
                | Self::PalmLeaves
                | Self::AcaciaLeaves
                | Self::BaobabLeaves
                | Self::KapokLeaves
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
                | Self::LoamBurrow
                | Self::PeatBurrow
                | Self::SiltBurrow
        )
    }

    #[inline]
    pub fn is_axis_aligned(self) -> bool {
        matches!(
            self,
            Self::AcaciaLog
                | Self::AspenLog
                | Self::BaobabLog
                | Self::BirchLog
                | Self::CherryLog
                | Self::CypressLog
                | Self::FirLog
                | Self::KapokLog
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
                | Self::AspenWood
                | Self::BaobabWood
                | Self::BirchWood
                | Self::CherryWood
                | Self::CypressWood
                | Self::FirWood
                | Self::KapokWood
                | Self::MahoganyWood
                | Self::MangroveWood
                | Self::MapleWood
                | Self::OakWood
                | Self::PalmWood
                | Self::PineWood
                | Self::RedwoodWood
                | Self::WillowWood
                | Self::YewWood
                | Self::BoneBlock
                | Self::CalcinedBone
                | Self::BloodyFlesh
                | Self::FleshBlock
                | Self::CornBale
                | Self::HayBale
                | Self::RiceBale
                | Self::StrawBale
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

        // Trees & Wood (logs, planks, wood, bamboo): 151..=241
        if matches!(id, 151..=241) {
            return NaturalRotationMode::None;
        }

        // Architecture, furniture, utilities, lighting, fibers: 432..=593
        if matches!(id, 432..=593) {
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

        // Natural isotropic: Soils, sands, gravels, clays, ashes (29..=84)
        if matches!(id, 29..=84) {
            return NaturalRotationMode::Isotropic;
        }

        // Geology & Rocks (85..=152)
        if matches!(id, 85..=152) {
            return NaturalRotationMode::Isotropic;
        }

        // Ores, Minerals & Gemology (244..=317)
        if matches!(id, 244..=317) {
            return NaturalRotationMode::Isotropic;
        }

        // Crystals, Speleothems, Volcanism, Fossils (318..=347)
        if matches!(id, 318..=347) {
            return NaturalRotationMode::Isotropic;
        }

        NaturalRotationMode::None
    }

    pub const PAINT_PALETTE: [[f32; 4]; 16] = [
        [0.102, 0.102, 0.102, 1.0], // 0: Black (#1A1A1A)
        [0.145, 0.267, 0.549, 1.0], // 1: Blue (#25448C)
        [0.361, 0.220, 0.118, 1.0], // 2: Brown (#5C381E)
        [0.078, 0.137, 0.294, 1.0], // 3: Dark Blue (#14234B)
        [0.247, 0.267, 0.282, 1.0], // 4: Dark Grey (#3F4448)
        [0.231, 0.416, 0.149, 1.0], // 5: Green (#3B6A26)
        [0.604, 0.631, 0.651, 1.0], // 6: Light Grey (#9AA1A6)
        [0.957, 0.710, 0.784, 1.0], // 7: Light Pink (#F4B5C8)
        [0.416, 0.745, 0.188, 1.0], // 8: Lime (#6ABE30)
        [0.851, 0.431, 0.078, 1.0], // 9: Orange (#D96E14)
        [0.910, 0.416, 0.573, 1.0], // 10: Pink (#E86A92)
        [0.455, 0.196, 0.557, 1.0], // 11: Purple (#74328E)
        [0.698, 0.133, 0.133, 1.0], // 12: Red (#B22222)
        [0.157, 0.627, 0.627, 1.0], // 13: Turquoise (#28A0A0)
        [0.941, 0.941, 0.941, 1.0], // 14: White (#F0F0F0)
        [0.902, 0.722, 0.000, 1.0], // 15: Yellow (#E6B800)
    ];

    #[inline]
    pub const fn is_painted(self) -> bool {
        self.paint_color().is_some()
    }

    pub const fn paint_color(self) -> Option<[f32; 4]> {
        let id = self as u16;
        let color_idx = match id {
            439..=454 => id - 439, // Brick
            471..=486 => id - 471, // Fabric
            488..=503 => id - 488, // Glass
            506..=521 => id - 506, // Linen
            527..=542 => id - 527, // Plaster
            545..=560 => id - 545, // Shingles
            563..=578 => id - 563, // TintedGlass
            580..=595 => id - 580, // Wool
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
            Self::BaobabLeaves => [0.65, 0.95, 0.32, 1.0],
            Self::KapokLeaves => [0.46, 0.98, 0.36, 1.0],
            Self::MahoganyLeaves => [0.45, 1.00, 0.40, 1.0],
            Self::MangroveLeaves => [0.40, 0.95, 0.42, 1.0],
            Self::PalmLeaves => [0.55, 1.05, 0.30, 1.0],
            Self::WillowLeaves => [0.52, 0.88, 0.48, 1.0],
            Self::YewLeaves => [0.28, 0.70, 0.38, 1.0],
            Self::LoamBurrow | Self::PeatBurrow | Self::SiltBurrow => [0.55, 0.94, 0.42, 1.0],
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
            Self::NullBlock | Self::NullLiquid => [255, 0, 255, 255],
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
            | Self::PowderSnow
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
            Self::LoamDirt | Self::LoamCoarse | Self::LoamRooted | Self::LoamBurrow => {
                [110, 80, 55, 255]
            }
            Self::LoamTilled => [95, 68, 45, 255],
            Self::PeatDirt | Self::PeatCoarse | Self::PeatRooted | Self::PeatBurrow => {
                [70, 52, 40, 255]
            }
            Self::PeatTilled => [60, 45, 35, 255],
            Self::SiltDirt | Self::SiltCoarse | Self::SiltRooted | Self::SiltBurrow => {
                [135, 110, 80, 255]
            }
            Self::SiltTilled => [115, 95, 70, 255],
            Self::CalicheDirt | Self::CalicheCoarse | Self::CalicheCracked => [200, 185, 160, 255],
            Self::LateriteDirt | Self::LateriteCoarse | Self::LateriteHardpan => [150, 60, 45, 255],
            Self::PermafrostDirt | Self::PermafrostCoarse => [105, 95, 85, 255],

            // Ash and gravels
            Self::AshPumice => [160, 160, 155, 255],
            Self::AshVolcanic => [70, 70, 75, 255],
            Self::GravelDirty | Self::PathDirty => [120, 100, 80, 255],
            Self::GravelAshy => [105, 105, 105, 255],
            Self::PathStony => [145, 140, 135, 255],

            // Moss
            Self::MossAmber => [180, 150, 40, 255],
            Self::MossCave => [40, 100, 70, 255],
            Self::MossCrimson => [140, 30, 45, 255],
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
            Self::EmitLightWarm | Self::EmitTorchWarm | Self::EmitLamp => {
                Color::srgb(1.0, 0.82, 0.42)
            }
            Self::EmitTorchFire => Color::srgb(1.0, 0.65, 0.25),
            Self::EmitLightCold | Self::EmitTorchCold => Color::srgb(0.80, 0.90, 1.0),
            Self::EmitLightRed | Self::EmitTorchRed => Color::srgb(1.0, 0.25, 0.25),
            Self::EmitLightGreen | Self::EmitTorchGreen => Color::srgb(0.25, 1.0, 0.25),
            Self::EmitLightBlue | Self::EmitTorchBlue => Color::srgb(0.25, 0.50, 1.0),
            Self::LiquidLava
            | Self::LiquidCoreMagma
            | Self::LiquidMolten
            | Self::Magma => Color::srgb(1.0, 0.45, 0.15),
            Self::GeothermalVent => Color::srgb(1.0, 0.55, 0.20),
            Self::FungiMycena | Self::FungiMycenaCap => Color::srgb(0.20, 0.90, 0.70),
            Self::CrystalCitrineGlinted => Color::srgb(1.0, 0.85, 0.30),
            Self::CrystalFluoriteGlinted => Color::srgb(0.70, 0.40, 0.95),
            Self::CrystalPinkSaltGlinted => Color::srgb(1.0, 0.60, 0.75),
            Self::CrystalSeleniteGlinted => Color::srgb(0.85, 0.92, 1.0),
            _ => Color::WHITE,
        }
    }
}
