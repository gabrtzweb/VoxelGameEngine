use super::{data::VOXEL_DATA, voxel::Voxel};

pub const FLAG_EMPTY: u16 = 1 << 0;
pub const FLAG_WATER: u16 = 1 << 1;
pub const FLAG_FLUID: u16 = 1 << 2;
pub const FLAG_COLLIDABLE: u16 = 1 << 3;
pub const FLAG_TORCH: u16 = 1 << 4;
pub const FLAG_BASKET: u16 = 1 << 5;
pub const FLAG_TRANSPARENT: u16 = 1 << 6;
pub const FLAG_LEAVES: u16 = 1 << 7;
pub const FLAG_SOLID_OPAQUE: u16 = 1 << 8;
pub const FLAG_LIGHT: u16 = 1 << 9;
pub const FLAG_POINT_LIGHT: u16 = 1 << 10;
pub const FLAG_CROSS_BILLBOARD: u16 = 1 << 11;

#[derive(Clone, Copy)]
struct VoxelProps {
    flags: u16,
    max_fluid_spread: u8,
}

const fn compute_voxel_props(v: Voxel) -> VoxelProps {
    let mut flags = 0u16;
    let id = v as u16;

    if id == 0 {
        flags |= FLAG_EMPTY;
    }

    if matches!(
        v,
        Voxel::WaterOccupied | Voxel::LiquidWater | Voxel::LiquidSpringWater
    ) {
        flags |= FLAG_WATER;
    }

    if matches!(
        v,
        Voxel::WaterOccupied
            | Voxel::LiquidAcid
            | Voxel::LiquidBlood
            | Voxel::LiquidCoreMagma
            | Voxel::LiquidLava
            | Voxel::LiquidMolten
            | Voxel::LiquidOoze
            | Voxel::LiquidSludge
            | Voxel::LiquidSpringWater
            | Voxel::LiquidTar
            | Voxel::LiquidWater
    ) {
        flags |= FLAG_FLUID;
    }

    if v.is_torch_const() {
        flags |= FLAG_TORCH;
    }

    if v.is_basket_const() {
        flags |= FLAG_BASKET;
    }

    if v.is_cross_billboard_const() {
        flags |= FLAG_CROSS_BILLBOARD;
    }

    if v.is_transparent_const() {
        flags |= FLAG_TRANSPARENT;
    }

    if v.is_leaves_const() {
        flags |= FLAG_LEAVES;
    }

    let is_empty = (flags & FLAG_EMPTY) != 0;
    let is_fluid = (flags & FLAG_FLUID) != 0;
    let is_torch = (flags & FLAG_TORCH) != 0;
    let is_occupied = matches!(id, 1 | 2);

    let is_collidable = (!is_empty
        && !is_fluid
        && !is_torch
        && (flags & FLAG_CROSS_BILLBOARD) == 0
        && !v.is_flat_pad_const()
        && !v.is_carpet()
        && !v.is_wall_decal_const()
        && !v.is_stick_pile()
        && !is_occupied)
        || v.is_crystal_cluster_const()
        || v.is_rubble_const()
        || matches!(v, Voxel::VegeLilyPad);
    if is_collidable {
        flags |= FLAG_COLLIDABLE;
    }

    const ROCKS_START: u16 = Voxel::Andesite as u16;
    const ROCKS_END: u16 = Voxel::Slate as u16;
    const ORES_START: u16 = Voxel::GemDiamondRough as u16;
    const ORES_END: u16 = Voxel::OreUraniumRaw as u16;
    let is_solid_opaque = (matches!(id, ROCKS_START..=ROCKS_END | ORES_START..=ORES_END)
        && !v.is_pebble_const()
        && !v.is_rubble_const())
        || (!is_empty
            && !is_fluid
            && (flags & FLAG_LEAVES) == 0
            && (flags & FLAG_TRANSPARENT) == 0
            && !is_torch
            && (flags & FLAG_BASKET) == 0
            && (flags & FLAG_CROSS_BILLBOARD) == 0
            && !v.is_pebble_const()
            && !v.is_rubble_const()
            && !v.is_speleothem_const()
            && !v.is_stick_pile()
            && !v.is_cocoa_pod()
            && !v.is_hanging_coconut()
            && !v.is_campfire()
            && !v.is_carpet()
            && !v.is_flat_pad_const()
            && !v.is_wall_decal_const()
            && !matches!(
                v,
                Voxel::BambooStalk | Voxel::BambooLeaves | Voxel::VegeCactus
            )
            && !is_occupied);

    if is_solid_opaque {
        flags |= FLAG_SOLID_OPAQUE;
    }

    if v.is_point_light_const() {
        flags |= FLAG_POINT_LIGHT;
    }

    if v.is_light_const() {
        flags |= FLAG_LIGHT;
    }

    let max_fluid_spread = match v {
        Voxel::WaterOccupied | Voxel::LiquidWater | Voxel::LiquidSpringWater => 8,
        Voxel::LiquidAcid => 5,
        Voxel::LiquidBlood => 4,
        Voxel::LiquidCoreMagma
        | Voxel::LiquidLava
        | Voxel::LiquidMolten
        | Voxel::LiquidOoze
        | Voxel::LiquidSludge => 3,
        Voxel::LiquidTar => 2,
        _ => 0,
    };

    VoxelProps {
        flags,
        max_fluid_spread,
    }
}

pub const MAX_VOXEL_COUNT: usize = 1024;

const fn init_voxel_properties() -> [VoxelProps; MAX_VOXEL_COUNT] {
    let mut table = [VoxelProps {
        flags: 0,
        max_fluid_spread: 0,
    }; MAX_VOXEL_COUNT];

    const MAX_ID: usize = Voxel::ALL[Voxel::ALL.len() - 1] as usize;
    let mut idx = 0;
    while idx <= MAX_ID {
        if let Some(v) = Voxel::from_u16(idx as u16) {
            table[idx] = compute_voxel_props(v);
        }
        idx += 1;
    }

    table
}

static VOXEL_PROPS: [VoxelProps; MAX_VOXEL_COUNT] = init_voxel_properties();

impl Voxel {
    #[inline(always)]
    pub const fn is_torch_const(self) -> bool {
        matches!(
            self,
            Self::EmitTorchBlue
                | Self::EmitTorchFire
                | Self::EmitTorchGreen
                | Self::EmitTorchRed
                | Self::EmitTorchWarm
        )
    }

    #[inline(always)]
    pub const fn is_basket_const(self) -> bool {
        matches!(self, Self::Basket)
    }

    #[inline(always)]
    pub const fn is_pebble_const(self) -> bool {
        matches!(
            self,
            Self::PebbleChert
                | Self::PebbleChertCluster
                | Self::PebbleFlint
                | Self::PebbleFlintCluster
                | Self::PebbleQuartz
                | Self::PebbleQuartzCluster
        )
    }

    #[inline(always)]
    pub const fn is_speleothem_const(self) -> bool {
        matches!(
            self,
            Self::SpeleothemBlueIce
                | Self::SpeleothemCalcite
                | Self::SpeleothemKarst
                | Self::SpeleothemSaltStraw
                | Self::ObsidianSpike
        )
    }

    pub const fn is_rubble_const(self) -> bool {
        matches!(
            self,
            Self::AndesiteRubble
                | Self::ArgilliteRubble
                | Self::BasaltRubble
                | Self::ChalkRubble
                | Self::DolomiteRubble
                | Self::GabbroRubble
                | Self::GraniteRubble
                | Self::LimestoneRubble
                | Self::MarbleRubble
                | Self::SandstoneRubble
                | Self::SandstoneDuneRubble
                | Self::SandstoneRedRubble
                | Self::SandstoneWhiteRubble
                | Self::SandstonePinkRubble
                | Self::SandstoneBlackRubble
                | Self::SlateRubble
        )
    }

    #[inline(always)]
    pub const fn is_point_light_const(self) -> bool {
        matches!(
            self,
            Self::Magma
                | Self::CrystalCitrineGlinted
                | Self::CrystalFluoriteGlinted
                | Self::CrystalPinkSaltGlinted
                | Self::CrystalSeleniteGlinted
                | Self::GeothermalVent
                | Self::EmitLampWarm
                | Self::EmitLampBlue
                | Self::EmitLampGreen
                | Self::EmitLampRed
                | Self::EmitTorchBlue
                | Self::EmitTorchFire
                | Self::EmitTorchGreen
                | Self::EmitTorchRed
                | Self::EmitTorchWarm
                | Self::Campfire
        )
    }

    #[inline(always)]
    pub const fn emission_level(self) -> u8 {
        match self {
            Self::ElementFire => 15,
            Self::TreeCharredLogLit | Self::TreeCharredBarkLit | Self::TreeCharredPlanksLit => 6,

            Self::EmitLampWarm
            | Self::EmitLampBlue
            | Self::EmitLampGreen
            | Self::EmitLampRed
            | Self::Campfire
            | Self::LiquidLava
            | Self::LiquidCoreMagma
            | Self::LiquidMolten
            | Self::Magma => 15,

            Self::EmitTorchBlue
            | Self::EmitTorchFire
            | Self::EmitTorchGreen
            | Self::EmitTorchRed
            | Self::EmitTorchWarm => 14,

            Self::GeothermalVent => 12,

            Self::CrystalCitrineGlinted
            | Self::CrystalFluoriteGlinted
            | Self::CrystalPinkSaltGlinted
            | Self::CrystalSeleniteGlinted
            | Self::CrystalAmethystGlinted
            | Self::CrystalAquamarineGlinted => 8,

            _ => 0,
        }
    }

    #[inline(always)]
    pub const fn is_leaves_const(self) -> bool {
        matches!(
            self,
            Self::AcaciaLeaves
                | Self::AcaciaLeavesDead
                | Self::BambooLeaves
                | Self::BirchLeaves
                | Self::BirchLeavesDead
                | Self::CherryLeaves
                | Self::CherryLeavesDead
                | Self::MahoganyLeaves
                | Self::MahoganyLeavesDead
                | Self::MangroveLeaves
                | Self::MangroveLeavesDead
                | Self::MapleLeavesDead
                | Self::MapleLeavesOrange
                | Self::MapleLeavesRed
                | Self::MapleLeavesYellow
                | Self::OakLeaves
                | Self::OakLeavesDead
                | Self::PalmLeaves
                | Self::PalmLeavesDead
                | Self::PineLeaves
                | Self::PineLeavesDead
                | Self::RedwoodLeaves
                | Self::RedwoodLeavesDead
                | Self::WillowLeaves
                | Self::WillowLeavesDead
                | Self::YewLeaves
                | Self::YewLeavesDead
        )
    }

    #[inline(always)]
    pub const fn is_tundra_grass(self) -> bool {
        matches!(
            self,
            Self::SiltTundraGrass
                | Self::LoamTundraGrass
                | Self::PeatTundraGrass
                | Self::PermafrostTundraGrass
        )
    }

    #[inline(always)]
    pub const fn has_grass_turf(self) -> bool {
        self.is_tundra_grass()
            || matches!(
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
    pub const fn is_flat_pad_const(self) -> bool {
        matches!(
            self,
            Self::VegeLilyPad
                | Self::FlowerClover
                | Self::FlowerPinkPetals
                | Self::FlowerWhitePetals
                | Self::FlowerWildflowers
                | Self::VegeLeafLitter
        )
    }

    #[inline(always)]
    pub const fn is_wall_decal_const(self) -> bool {
        matches!(
            self,
            Self::VegeVines | Self::VegeLichen | Self::VegeHangingMoss | Self::VegeHangingRoots
        )
    }

    #[inline(always)]
    pub const fn is_petal_carpet(self) -> bool {
        matches!(
            self,
            Self::FlowerPinkPetals
                | Self::FlowerWhitePetals
                | Self::FlowerWildflowers
                | Self::FlowerClover
        )
    }

    #[inline(always)]
    pub const fn is_crystal_solid_block_const(self) -> bool {
        matches!(
            self,
            Self::CrystalAmethystBlock
                | Self::CrystalAquamarineBlock
                | Self::CrystalCitrineBlock
                | Self::CrystalFluoriteBlock
                | Self::CrystalPinkSaltBlock
                | Self::CrystalSeleniteBlock
        )
    }

    #[inline(always)]
    pub const fn is_fluid_opaque_const(self) -> bool {
        matches!(
            self,
            Self::LiquidLava
                | Self::LiquidCoreMagma
                | Self::LiquidMolten
                | Self::LiquidTar
                | Self::LiquidSludge
                | Self::LiquidBlood
                | Self::LiquidOoze
        )
    }

    #[inline(always)]
    pub const fn is_fluid_transparent_const(self) -> bool {
        matches!(
            self,
            Self::LiquidWater | Self::WaterOccupied | Self::LiquidSpringWater | Self::LiquidAcid
        )
    }

    #[inline(always)]
    pub const fn is_crystal_cluster_const(self) -> bool {
        matches!(
            self,
            Self::CrystalAmethystCluster
                | Self::CrystalAmethystGlinted
                | Self::CrystalAquamarineCluster
                | Self::CrystalAquamarineGlinted
                | Self::CrystalCitrineCluster
                | Self::CrystalCitrineGlinted
                | Self::CrystalFluoriteCluster
                | Self::CrystalFluoriteGlinted
                | Self::CrystalPinkSaltCluster
                | Self::CrystalPinkSaltGlinted
                | Self::CrystalSeleniteCluster
                | Self::CrystalSeleniteGlinted
        )
    }

    #[inline(always)]
    pub const fn is_crop_billboard(self) -> bool {
        let id = self as u16;
        // 538..=628: Domestic crop stages and stems
        // 631..=650: Wild crops (except melon/pumpkin solid blocks 651, 652)
        // 653..=681: Agricultural items and seeds
        // 700..=710: Crafting materials & tools
        (id >= 538 && id <= 628)
            || (id >= 631 && id <= 650)
            || (id >= 653 && id <= 681)
            || (id >= 700 && id <= 710)
            || id == Voxel::ItemStick as u16
            || id == Voxel::ItemCocoaBeans as u16
            || id == Voxel::ItemCoconut as u16
    }

    #[inline(always)]
    pub const fn is_crop_stage(self) -> bool {
        let id = self as u16;
        (id >= 538 && id <= 628) || (id >= 684 && id <= 689)
    }

    /// Returns true for inventory-only items (drops, seeds, food slices, sticks, crafting materials, tools)
    /// that must never be placed directly as world blocks.
    #[inline(always)]
    pub const fn is_inventory_only_item(self) -> bool {
        let id = self as u16;
        (id >= 653 && id <= 681)
            || (id >= 700 && id <= 710)
            || id == Voxel::ItemStick as u16
            || id == Voxel::ItemCocoaBeans as u16
            || id == Voxel::ItemCoconut as u16
    }

    #[inline(always)]
    pub const fn is_tool(self) -> bool {
        let id = self as u16;
        id >= Voxel::ToolFireStarter as u16 && id <= Voxel::ToolFlintShovel as u16
    }

    #[inline(always)]
    pub const fn max_stack_size(self) -> u16 {
        if self.is_tool() { 1 } else { 64 }
    }

    /// Returns true for ground vegetation (wildgrass, ferns, bushes, etc.) that drops Plant Fiber
    /// unless harvested directly with Shears.
    pub fn is_shearable_vegetation(self) -> bool {
        let id = self.id();
        (id.starts_with("vege_")
            && self != Voxel::VegeCactus
            && self != Voxel::VegeLeafLitter
            && self != Voxel::VegeLilyPad)
            || self == Voxel::MossCarpet
    }

    /// Base time in seconds to break this block in Survival mode with bare hands / neutral tool.
    pub fn base_break_time_secs(self) -> f32 {
        if self == Voxel::Mantle {
            return f32::INFINITY;
        }
        let id = self.id();
        let tex = self.texture_name().unwrap_or("");

        if self.is_torch()
            || self.is_flora_billboard()
            || self.is_crop_billboard()
            || self.is_carpet()
            || self == Voxel::ElementFire
            || self == Voxel::VegeLeafLitter
            || self == Voxel::StickPile
            || id.starts_with("fungi_")
            || id.starts_with("pebble_")
            || id.starts_with("flower_")
        {
            return 0.15;
        }

        if self.is_leaves_const() || tex.starts_with("frost_snow") || id.contains("snow") {
            return 0.45;
        }

        if tex.starts_with("soil_") || id.starts_with("path_") || id.starts_with("moss_") {
            return 1.05;
        }

        if tex.starts_with("tree_")
            || self.is_campfire()
            || matches!(
                self,
                Voxel::Barrel
                    | Voxel::Crate
                    | Voxel::Basket
                    | Voxel::Bookshelf
                    | Voxel::BookshelfEmpty
            )
        {
            return 1.65;
        }

        if id.starts_with("ore_")
            || id.starts_with("gem_")
            || id.starts_with("crystal_")
            || id.contains("obsidian")
        {
            return 3.40;
        }

        if tex.starts_with("rock_")
            || tex.starts_with("cave_")
            || tex.starts_with("deco_")
            || tex.starts_with("frost_")
            || id.starts_with("dripstone_")
            || id.starts_with("speleothem_")
            || id.starts_with("fossil_")
        {
            return 2.40;
        }

        1.0
    }

    /// Mining speed multiplier granted by the held tool against this target block.
    pub fn tool_speed_multiplier(self, held: Option<Voxel>) -> f32 {
        let id = self.id();
        let tex = self.texture_name().unwrap_or("");

        let is_rock_like = tex.starts_with("rock_")
            || tex.starts_with("cave_")
            || id.starts_with("ore_")
            || id.starts_with("gem_")
            || id.starts_with("crystal_")
            || id.starts_with("dripstone_")
            || id.starts_with("speleothem_")
            || id.starts_with("fossil_")
            || tex.starts_with("frost_");
        let is_wood_like = tex.starts_with("tree_")
            || self.is_campfire()
            || matches!(
                self,
                Voxel::Barrel
                    | Voxel::Crate
                    | Voxel::Basket
                    | Voxel::Bookshelf
                    | Voxel::BookshelfEmpty
            );
        let is_soil_like = tex.starts_with("soil_")
            || id.starts_with("path_")
            || id.starts_with("moss_")
            || id.contains("snow");
        let is_crop_or_leaf = self.is_crop_billboard()
            || self.is_leaves_const()
            || id.starts_with("wild_")
            || id.contains("melon")
            || id.contains("pumpkin");
        let is_organic =
            tex.starts_with("bio_") || tex.starts_with("organic_") || tex.starts_with("nest_");

        match held {
            Some(Voxel::ToolFlintPickaxe) if is_rock_like => 3.4,
            Some(Voxel::ToolFlintAxe) if is_wood_like => 3.4,
            Some(Voxel::ToolFlintShovel) if is_soil_like => 2.2,
            Some(Voxel::ToolFlintHoe) if is_crop_or_leaf => 3.4,
            Some(Voxel::ToolFlintShears)
                if self.is_shearable_vegetation() || self.is_leaves_const() =>
            {
                4.0
            }
            Some(Voxel::ToolFlintKnife) if is_organic || self.is_shearable_vegetation() => 3.0,
            Some(Voxel::ToolFlintHammer) if is_rock_like || tex.starts_with("deco_") => 2.6,
            _ => {
                if is_rock_like {
                    0.45
                } else {
                    1.0
                }
            }
        }
    }

    #[inline(always)]
    pub const fn is_flora_billboard(self) -> bool {
        const FLORA_START: u16 = Voxel::VegeWildgrass as u16;
        const FLORA_END: u16 = Voxel::FlowerWildflowers as u16;
        let id = self as u16;
        ((id >= FLORA_START && id <= FLORA_END)
            && id != (Self::VegeCactus as u16)
            && !self.is_flat_pad_const()
            && !self.is_wall_decal_const())
            || matches!(self, Self::VegeArcticMoss | Self::VegeArcticGrass)
    }

    #[inline(always)]
    pub const fn is_cross_billboard_const(self) -> bool {
        self.is_flora_billboard()
            || self.is_crop_billboard()
            || matches!(
                self,
                Self::ElementFire
                    | Self::CrystalAmethystCluster
                    | Self::CrystalAmethystGlinted
                    | Self::CrystalAquamarineCluster
                    | Self::CrystalAquamarineGlinted
                    | Self::CrystalCitrineCluster
                    | Self::CrystalCitrineGlinted
                    | Self::CrystalFluoriteCluster
                    | Self::CrystalFluoriteGlinted
                    | Self::CrystalPinkSaltCluster
                    | Self::CrystalPinkSaltGlinted
                    | Self::CrystalSeleniteCluster
                    | Self::CrystalSeleniteGlinted
                    | Self::BoneWorms
                    | Self::BrainCoralFan
                    | Self::BubbleCoralFan
                    | Self::DeadCoralFan
                    | Self::DyingCoralFan
                    | Self::FireCoralFan
                    | Self::HeartCoralFan
                    | Self::HornCoralFan
                    | Self::LeafCoralFan
                    | Self::ReedCoralFan
                    | Self::Seagrass
                    | Self::TallSeagrassBottom
                    | Self::TallSeagrassTop
                    | Self::BullKelp
                    | Self::BullKelpPlant
                    | Self::Delesseria
                    | Self::Kelp
                    | Self::KelpPlant
                    | Self::RedAlgae
                    | Self::Sargassum
                    | Self::Seaweed
                    | Self::SpikeCoralFan
                    | Self::StaghornCoralFan
                    | Self::SunCoralFan
                    | Self::ThornCoralFan
                    | Self::TubeCoralFan
                    | Self::WaxCoralFan
                    | Self::ZigzagCoralFan
                    | Self::Cobweb
                    | Self::CobwebArch
                    | Self::CobwebHanging
                    | Self::CobwebStretched
                    | Self::FungiAmanita
                    | Self::FungiPorcini
                    | Self::FungiRussula
                    | Self::FungiShiitake
                    | Self::FungiVerdigris
            )
    }

    #[inline(always)]
    pub const fn is_transparent_const(self) -> bool {
        matches!(
            self,
            Self::WaterOccupied
                | Self::LiquidSpringWater
                | Self::LiquidWater
                | Self::FrostBlueIce
                | Self::FrostBlueIceCracked
                | Self::FrostIce
                | Self::FrostIceBlack
                | Self::FrostIceBlackFragile
                | Self::FrostIceFragile
                | Self::JellyBlock
                | Self::SlimeBlock
        )
    }

    #[inline(always)]
    pub const fn is_light_const(self) -> bool {
        matches!(
            self,
            Self::ElementFire
                | Self::TreeCharredLog
                | Self::TreeCharredBark
                | Self::TreeCharredPlanks
                | Self::TreeCharredLogLit
                | Self::TreeCharredBarkLit
                | Self::TreeCharredPlanksLit
                | Self::LiquidLava
                | Self::LiquidCoreMagma
                | Self::LiquidMolten
                | Self::Magma
                | Self::CrystalAmethystGlinted
                | Self::CrystalAquamarineGlinted
                | Self::CrystalCitrineGlinted
                | Self::CrystalFluoriteGlinted
                | Self::CrystalPinkSaltGlinted
                | Self::CrystalSeleniteGlinted
                | Self::GeothermalVent
                | Self::EmitLampWarm
                | Self::EmitLampBlue
                | Self::EmitLampGreen
                | Self::EmitLampRed
                | Self::EmitTorchBlue
                | Self::EmitTorchFire
                | Self::EmitTorchGreen
                | Self::EmitTorchRed
                | Self::EmitTorchWarm
                | Self::Campfire
        )
    }
}

impl Voxel {
    #[inline(always)]
    pub fn is_empty(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_EMPTY) != 0
    }

    #[inline(always)]
    pub fn is_water(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_WATER) != 0
    }

    #[inline(always)]
    pub fn is_fluid(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_FLUID) != 0
    }

    #[inline(always)]
    pub fn is_fluid_opaque(self) -> bool {
        self.is_fluid_opaque_const()
    }

    #[inline(always)]
    pub fn is_fluid_transparent(self) -> bool {
        self.is_fluid_transparent_const()
    }

    #[inline(always)]
    pub fn max_fluid_spread(self) -> u8 {
        VOXEL_PROPS[self as usize].max_fluid_spread
    }

    #[inline(always)]
    pub fn is_collidable(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_COLLIDABLE) != 0
    }

    #[inline(always)]
    pub fn is_torch(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_TORCH) != 0
    }

    #[inline(always)]
    pub fn is_basket(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_BASKET) != 0
    }

    #[inline(always)]
    pub fn is_cross_billboard(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_CROSS_BILLBOARD) != 0
    }

    #[inline(always)]
    pub fn is_pebble(self) -> bool {
        self.is_pebble_const()
    }

    #[inline(always)]
    pub fn is_bamboo(self) -> bool {
        matches!(self, Self::BambooStalk | Self::BambooLeaves)
    }

    #[inline(always)]
    pub fn is_rubble(self) -> bool {
        self.is_rubble_const()
    }

    #[inline(always)]
    pub fn is_speleothem(self) -> bool {
        self.is_speleothem_const()
    }

    #[inline(always)]
    pub fn is_crystal_solid_block(self) -> bool {
        self.is_crystal_solid_block_const()
    }

    #[inline(always)]
    pub fn is_crystal_cluster(self) -> bool {
        self.is_crystal_cluster_const()
    }

    #[inline(always)]
    pub fn has_custom_mesh(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & (FLAG_TORCH | FLAG_BASKET | FLAG_CROSS_BILLBOARD)) != 0
            || self.is_pebble_const()
            || self.is_rubble_const()
            || self.is_speleothem_const()
            || self.is_bamboo()
            || self.is_stick_pile()
            || self.is_cocoa_pod()
            || self.is_hanging_coconut()
            || self.is_campfire()
    }

    #[inline(always)]
    pub fn is_mushroom(self) -> bool {
        matches!(
            self,
            Self::FungiAmanita
                | Self::FungiPorcini
                | Self::FungiRussula
                | Self::FungiShiitake
                | Self::FungiVerdigris
        )
    }

    #[inline(always)]
    pub fn is_coral_fan(self) -> bool {
        matches!(
            self,
            Self::BrainCoralFan
                | Self::BubbleCoralFan
                | Self::DeadCoralFan
                | Self::DyingCoralFan
                | Self::FireCoralFan
                | Self::HeartCoralFan
                | Self::HornCoralFan
                | Self::LeafCoralFan
                | Self::ReedCoralFan
                | Self::SpikeCoralFan
                | Self::StaghornCoralFan
                | Self::SunCoralFan
                | Self::ThornCoralFan
                | Self::TubeCoralFan
                | Self::WaxCoralFan
                | Self::ZigzagCoralFan
        )
    }

    #[inline(always)]
    pub fn is_marine_flora(self) -> bool {
        matches!(
            self,
            Self::Seagrass
                | Self::TallSeagrassBottom
                | Self::TallSeagrassTop
                | Self::BullKelp
                | Self::BullKelpPlant
                | Self::Delesseria
                | Self::Kelp
                | Self::KelpPlant
                | Self::RedAlgae
                | Self::Sargassum
                | Self::Seaweed
                | Self::BoneWorms
        )
    }

    #[inline(always)]
    pub fn is_hanging_flora(self) -> bool {
        matches!(
            self,
            Self::VegeCaveVines
                | Self::VegeCaveVinesPlant
                | Self::VegeHangingMoss
                | Self::VegeHangingRoots
                | Self::VegeVines
        )
    }

    #[inline(always)]
    pub fn requires_floor_support(self) -> bool {
        self.is_pebble()
            || self.is_rubble()
            || self.is_stick_pile()
            || self.is_mushroom()
            || self.is_marine_flora()
            || self.is_crop_billboard()
            || self.is_leaf_litter()
            || self.is_carpet()
            || self.is_petal_carpet()
            || (self.is_flora_billboard() && !self.is_hanging_flora())
    }

    /// Returns true for sandy, coastal, or arid mineral substrates (sands, sandy gravel, caliche).
    #[inline(always)]
    pub fn is_sandy_soil(self) -> bool {
        matches!(
            self,
            Self::SandCommon
                | Self::SeagrassSand
                | Self::SandDune
                | Self::SandRed
                | Self::SandWhite
                | Self::SandPink
                | Self::SandBlack
                | Self::GravelSandy
                | Self::PathSandy
                | Self::CalicheCracked
                | Self::CalicheCoarse
                | Self::CalicheDirt
                | Self::LateriteHardpan
        )
    }

    /// Returns true for fertile soil substrates capable of supporting lush temperate vegetation,
    /// flowers, and wild crops (grasses, dirts, mulch, mud, tilled soils, moss).
    #[inline(always)]
    pub fn is_fertile_soil(self) -> bool {
        self.is_grass_soil()
            || matches!(
                self,
                Self::LoamDirt
                    | Self::LoamCoarse
                    | Self::LoamMud
                    | Self::LoamMulch
                    | Self::LoamRooted
                    | Self::LoamTilled
                    | Self::LoamTundraGrass
                    | Self::PeatDirt
                    | Self::PeatCoarse
                    | Self::PeatMud
                    | Self::PeatMulch
                    | Self::PeatRooted
                    | Self::PeatTilled
                    | Self::PeatTundraGrass
                    | Self::SiltDirt
                    | Self::SiltCoarse
                    | Self::SiltMud
                    | Self::SiltMulch
                    | Self::SiltRooted
                    | Self::SiltTilled
                    | Self::SiltTundraGrass
                    | Self::LateriteDirt
                    | Self::LateriteCoarse
                    | Self::CalicheDirt
                    | Self::CalicheCoarse
                    | Self::PermafrostDirt
                    | Self::PermafrostCoarse
                    | Self::PermafrostTundraGrass
                    | Self::PathDirty
                    | Self::GravelDirty
                    | Self::GravelAshy
                    | Self::ClayBlue
                    | Self::ClayRed
                    | Self::ClayWhite
                    | Self::ClayYellow
                    | Self::MossForest
                    | Self::MossyAndesite
                    | Self::MossyArgillite
                    | Self::MossyBasalt
                    | Self::MossyChalk
                    | Self::MossyDolomite
                    | Self::MossyGranite
                    | Self::MossyLimestone
            )
    }

    /// Validates whether a floor-standing vegetation, flower, or wild crop block can be placed
    /// on top of `ground`. Non-flora blocks always return true.
    pub fn can_be_placed_on_ground(self, ground: Voxel) -> bool {
        // Cave flora & lichen can grow on any solid opaque stone/earth surface
        if matches!(
            self,
            Self::VegeCaveGrowths
                | Self::VegeGrainyCaveGrowths
                | Self::VegeCaveHyssop
                | Self::VegeLichen
        ) {
            return ground.is_solid_opaque();
        }

        // Cacti & desert succulents: sand or arid soils (and cactus stacking)
        if matches!(
            self,
            Self::VegeCactus
                | Self::VegeBarrelCactus
                | Self::VegePricklyPearCactus
                | Self::FlowerCactusFlower
        ) {
            return ground.is_sandy_soil() || ground == Self::VegeCactus;
        }

        // Coastal, arid, hardy grasses, rustic flowers, and adaptable wild crops:
        // allowed on both sandy/arid soils and fertile soils/dirt/gravel
        if matches!(
            self,
            Self::VegeSeaOatsBottom
                | Self::VegeDeadBush
                | Self::VegeDrygrass
                | Self::VegeRedShrub
                | Self::VegeBluffGrass
                | Self::VegeBush
                | Self::VegeBracken
                | Self::VegeWildgrass
                | Self::VegeTallWildgrassBottom
                | Self::VegeCattailBottom
                | Self::VegeArcticMoss
                | Self::VegeArcticGrass
                | Self::FlowerGlobeMallow
                | Self::FlowerDandelion
                | Self::FlowerPoppy
                | Self::FlowerMarigold
                | Self::FlowerWildflowers
                | Self::FlowerLavender
                | Self::FlowerOxeyeDaisy
                | Self::FlowerAllium
                | Self::WildSugarCaneBottom
                | Self::WildWheatBottom
                | Self::WildBarleyBottom
                | Self::WildCarrots
                | Self::WildOnions
                | Self::WildGarlic
                | Self::WildPepper
                | Self::WildRiceBottom
        ) {
            return ground.is_sandy_soil() || ground.is_fertile_soil();
        }

        // Lush woodland / shade-loving flora, delicate flowers, and rich-soil wild crops
        // require fertile soil (grass, dirt, dirty gravel, clay, mud, moss, tilled soil)
        let is_ground_flora = (self.is_flora_billboard() && !self.is_hanging_flora())
            || matches!(
                self,
                Self::FlowerPinkPetals
                    | Self::FlowerWhitePetals
                    | Self::WildPotatoes
                    | Self::WildBeetroots
                    | Self::WildCabbages
                    | Self::WildLettuce
                    | Self::WildTomatoes
                    | Self::WildCoffee
                    | Self::WildCornBottom
                    | Self::WildMelon
                    | Self::WildPumpkin
            );

        if is_ground_flora {
            return ground.is_fertile_soil();
        }

        true
    }

    #[inline(always)]
    pub fn is_transparent(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_TRANSPARENT) != 0
    }

    #[inline(always)]
    pub fn is_leaves(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_LEAVES) != 0
    }

    #[inline(always)]
    pub fn is_solid_opaque(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_SOLID_OPAQUE) != 0
    }

    #[inline(always)]
    pub fn is_light(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_LIGHT) != 0
    }

    #[inline(always)]
    pub fn is_point_light_fixture(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & FLAG_POINT_LIGHT) != 0
    }

    #[inline(always)]
    pub fn is_unbreakable(self) -> bool {
        matches!(self, Self::Mantle | Self::CobbledMantle)
    }

    #[inline(always)]
    pub fn id_str(self) -> &'static str {
        let idx = self as usize;
        if idx < VOXEL_DATA.len() {
            VOXEL_DATA[idx].0
        } else {
            ""
        }
    }

    #[inline(always)]
    pub fn label_str(self) -> &'static str {
        let idx = self as usize;
        if idx < VOXEL_DATA.len() {
            VOXEL_DATA[idx].1
        } else {
            ""
        }
    }
}
