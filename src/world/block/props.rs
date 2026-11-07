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

    if matches!(v, Voxel::WaterOccupied | Voxel::LiquidWater | Voxel::LiquidSpringWater) {
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
        && !v.is_wall_decal_const()
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
    let is_solid_opaque =
        (matches!(id, ROCKS_START..=ROCKS_END | ORES_START..=ORES_END) && !v.is_pebble_const() && !v.is_rubble_const())
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
        )
    }

    #[inline(always)]
    pub const fn emission_level(self) -> u8 {
        match self {
            Self::ElementFire => 15,
            Self::TreeCharredLog
            | Self::TreeCharredBark
            | Self::TreeCharredPlanks => 6,

            Self::EmitLampWarm
            | Self::EmitLampBlue
            | Self::EmitLampGreen
            | Self::EmitLampRed
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
    pub const fn has_grass_turf(self) -> bool {
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
    pub const fn is_flat_pad_const(self) -> bool {
        matches!(
            self,
            Self::VegeLilyPad
                | Self::FlowerClover
                | Self::FlowerPinkPetals
                | Self::FlowerWhitePetals
                | Self::FlowerWildflowers
        )
    }

    #[inline(always)]
    pub const fn is_wall_decal_const(self) -> bool {
        matches!(
            self,
            Self::VegeVines
                | Self::VegeLichen
                | Self::VegeHangingMoss
                | Self::VegeHangingRoots
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
    pub const fn is_flora_billboard(self) -> bool {
        const FLORA_START: u16 = Voxel::VegeWildgrass as u16;
        const FLORA_END: u16 = Voxel::FlowerWildflowers as u16;
        let id = self as u16;
        (id >= FLORA_START && id <= FLORA_END)
            && id != (Self::VegeCactus as u16)
            && !self.is_flat_pad_const()
            && !self.is_wall_decal_const()
    }

    #[inline(always)]
    pub const fn is_cross_billboard_const(self) -> bool {
        self.is_flora_billboard()
            || matches!(
                self,
                Self::ElementFire
                |                Self::CrystalAmethystCluster
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
                |             Self::LiquidLava
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
            || self.is_mushroom()
            || self.is_coral_fan()
            || self.is_marine_flora()
            || (self.is_flora_billboard() && !self.is_hanging_flora())
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
