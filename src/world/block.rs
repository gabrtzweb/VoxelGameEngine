#![allow(dead_code)]

use bevy::prelude::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum BlockShape {
    #[default]
    Full = 0,
    Slab = 1,
    Stair = 2,
    Column = 3,
    Torch = 4,
    Basket = 5,
    PebbleOne = 6,
    PebbleTwo = 7,
    PebbleThree = 8,
    CrossBillboard = 9,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum NaturalRotationMode {
    #[default]
    None = 0,
    Isotropic = 1,
    SoilSide = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum PillarAxis {
    #[default]
    Y = 0,
    X = 1,
    Z = 2,
}

impl PillarAxis {
    #[inline]
    pub fn from_normal(normal: IVec3) -> Self {
        if normal.x != 0 {
            Self::X
        } else if normal.z != 0 {
            Self::Z
        } else {
            Self::Y
        }
    }
}

impl BlockShape {
    /// All 10 registered block shapes in the engine.
    pub const fn all() -> [BlockShape; 10] {
        [
            BlockShape::Full,
            BlockShape::Slab,
            BlockShape::Stair,
            BlockShape::Column,
            BlockShape::Torch,
            BlockShape::Basket,
            BlockShape::PebbleOne,
            BlockShape::PebbleTwo,
            BlockShape::PebbleThree,
            BlockShape::CrossBillboard,
        ]
    }

    /// The 4 geometric shapes selectable via the player radial wheel.
    pub const fn shapable() -> [BlockShape; 4] {
        [
            BlockShape::Full,
            BlockShape::Slab,
            BlockShape::Stair,
            BlockShape::Column,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "Full Block",
            Self::Slab => "Slab",
            Self::Stair => "Stairs",
            Self::Column => "Column",
            Self::Torch => "Torch",
            Self::Basket => "Basket",
            Self::PebbleOne => "Pebble (1)",
            Self::PebbleTwo => "Pebble (2)",
            Self::PebbleThree => "Pebble (3)",
            Self::CrossBillboard => "Cross-Billboard",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Full => "FULL",
            Self::Slab => "SLAB",
            Self::Stair => "STAIR",
            Self::Column => "COLUMN",
            Self::Torch => "TORCH",
            Self::Basket => "BASKET",
            Self::PebbleOne => "PEBBLE1",
            Self::PebbleTwo => "PEBBLE2",
            Self::PebbleThree => "PEBBLE3",
            Self::CrossBillboard => "CROSS",
        }
    }

    pub const fn orientation_count(self) -> u8 {
        match self {
            Self::Full => 3,
            Self::Basket | Self::CrossBillboard => 1,
            Self::Slab => 6,
            Self::Stair => 8,
            Self::Column => 6,
            Self::Torch => 5,
            Self::PebbleOne | Self::PebbleTwo | Self::PebbleThree => 4,
        }
    }

    pub fn orientation_name(self, orientation: u8) -> &'static str {
        match self {
            Self::Full => match orientation % 3 {
                0 => "Vertical (Y)",
                1 => "Horizontal (X)",
                2 => "Horizontal (Z)",
                _ => "Standard",
            },
            Self::Basket | Self::CrossBillboard => "Standard",
            Self::Slab => match orientation % 6 {
                0 => "Bottom (Floor)",
                1 => "Top (Ceiling)",
                2 => "North Wall (-Z)",
                3 => "South Wall (+Z)",
                4 => "West Wall (-X)",
                5 => "East Wall (+X)",
                _ => "Unknown",
            },
            Self::Stair => match orientation % 8 {
                0 => "Upright (+X)",
                1 => "Upright (-X)",
                2 => "Upright (+Z)",
                3 => "Upright (-Z)",
                4 => "Inverted (+X)",
                5 => "Inverted (-X)",
                6 => "Inverted (+Z)",
                7 => "Inverted (-Z)",
                _ => "Unknown",
            },
            Self::Column => match orientation % 6 {
                0 => "Centered Vertical",
                1 => "Corner (Min X, Min Z)",
                2 => "Corner (Max X, Min Z)",
                3 => "Corner (Min X, Max Z)",
                4 => "Corner (Max X, Max Z)",
                5 => "Centered Horizontal",
                _ => "Unknown",
            },
            Self::Torch => match orientation % 5 {
                0 => "Floor (Standing)",
                1 => "Wall North (-Z)",
                2 => "Wall South (+Z)",
                3 => "Wall West (-X)",
                4 => "Wall East (+X)",
                _ => "Unknown",
            },
            Self::PebbleOne | Self::PebbleTwo | Self::PebbleThree => match orientation % 4 {
                0 => "North (0°)",
                1 => "East (90°)",
                2 => "South (180°)",
                3 => "West (270°)",
                _ => "Unknown",
            },
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::Slab,
            Self::Slab => Self::Stair,
            Self::Stair => Self::Column,
            Self::Column => Self::CrossBillboard,
            Self::CrossBillboard => Self::Full,
            other => other,
        }
    }

    pub fn local_boxes(self, orientation: u8) -> ([Vec3; 2], Option<[Vec3; 2]>) {
        match self {
            Self::Full | Self::Basket | Self::CrossBillboard => ([Vec3::ZERO, Vec3::ONE], None),
            Self::Slab => {
                let bbox = match orientation % 6 {
                    0 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.5, 1.0)],
                    1 => [Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0)],
                    2 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.5)],
                    3 => [Vec3::new(0.0, 0.0, 0.5), Vec3::new(1.0, 1.0, 1.0)],
                    4 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 1.0)],
                    _ => [Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)],
                };
                (bbox, None)
            }
            Self::Column => {
                let bbox = match orientation % 6 {
                    0 => [Vec3::new(0.25, 0.0, 0.25), Vec3::new(0.75, 1.0, 0.75)],
                    1 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5)],
                    2 => [Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.5)],
                    3 => [Vec3::new(0.0, 0.0, 0.5), Vec3::new(0.5, 1.0, 1.0)],
                    4 => [Vec3::new(0.5, 0.0, 0.5), Vec3::new(1.0, 1.0, 1.0)],
                    _ => [Vec3::new(0.0, 0.25, 0.25), Vec3::new(1.0, 0.75, 0.75)],
                };
                (bbox, None)
            }
            Self::Stair => {
                let is_inverted = orientation >= 4;
                let step_dir = orientation % 4;

                let base = if !is_inverted {
                    [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.5, 1.0)]
                } else {
                    [Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0)]
                };

                let step = match (is_inverted, step_dir) {
                    (false, 0) => [Vec3::new(0.5, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0)],
                    (false, 1) => [Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.5, 1.0, 1.0)],
                    (false, 2) => [Vec3::new(0.0, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0)],
                    (false, 3) => [Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 0.5)],

                    (true, 0) => [Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.0, 0.5, 1.0)],
                    (true, 1) => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 1.0)],
                    (true, 2) => [Vec3::new(0.0, 0.0, 0.5), Vec3::new(1.0, 0.5, 1.0)],
                    (true, 3) => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.5, 0.5)],
                    _ => unreachable!(),
                };

                (base, Some(step))
            }
            Self::Torch => {
                let bbox = match orientation % 5 {
                    0 => [Vec3::new(0.40, 0.0, 0.40), Vec3::new(0.60, 0.65, 0.60)],
                    1 => [Vec3::new(0.40, 0.12, 0.0), Vec3::new(0.60, 0.78, 0.45)],
                    2 => [Vec3::new(0.40, 0.12, 0.55), Vec3::new(0.60, 0.78, 1.0)],
                    3 => [Vec3::new(0.0, 0.12, 0.40), Vec3::new(0.45, 0.78, 0.60)],
                    _ => [Vec3::new(0.55, 0.12, 0.40), Vec3::new(1.0, 0.78, 0.60)],
                };
                (bbox, None)
            }
            Self::PebbleOne => {
                let b = rotate_aabb_y(
                    Vec3::new(7.0 / 16.0, 0.0, 5.0 / 16.0),
                    Vec3::new(11.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0),
                    orientation,
                );
                (b, None)
            }
            Self::PebbleTwo => {
                let b = rotate_aabb_y(
                    Vec3::new(3.0 / 16.0, 0.0, 10.0 / 16.0),
                    Vec3::new(5.0 / 16.0, 1.0 / 16.0, 12.0 / 16.0),
                    orientation,
                );
                (b, None)
            }
            Self::PebbleThree => {
                let b1 = rotate_aabb_y(
                    Vec3::new(7.0 / 16.0, 0.0, 8.0 / 16.0),
                    Vec3::new(12.0 / 16.0, 2.0 / 16.0, 13.0 / 16.0),
                    orientation,
                );
                let b2 = rotate_aabb_y(
                    Vec3::new(5.0 / 16.0, 0.0, 6.0 / 16.0),
                    Vec3::new(8.0 / 16.0, 1.0 / 16.0, 10.0 / 16.0),
                    orientation,
                );
                (b1, Some(b2))
            }
        }
    }
}

fn rotate_aabb_y(min: Vec3, max: Vec3, orientation: u8) -> [Vec3; 2] {
    match orientation % 4 {
        0 => [min, max],
        1 => [
            Vec3::new(1.0 - max.z, min.y, min.x),
            Vec3::new(1.0 - min.z, max.y, max.x),
        ],
        2 => [
            Vec3::new(1.0 - max.x, min.y, 1.0 - max.z),
            Vec3::new(1.0 - min.x, max.y, 1.0 - min.z),
        ],
        _ => [
            Vec3::new(min.z, min.y, 1.0 - max.x),
            Vec3::new(max.z, max.y, 1.0 - min.x),
        ],
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[allow(non_camel_case_types)]
pub enum Voxel {
    #[default]
    Air = 0,
    Occupied = 1,
    WaterOccupied = 2,

    // --- Category 01: Technical & System Blocks ---
    DebugAccept = 3,
    DebugFail = 4,
    DebugInstance = 5,
    DebugStart = 6,
    NullBlock = 7,
    NullCrossmesh = 8,
    NullGeneric = 9,
    NullLiquid = 10,

    // --- Category 02: Hydrosphere & Cryology ---
    FrostBlueIce = 11,
    FrostBlueIceCracked = 12,
    FrostIce = 13,
    FrostIceBlack = 14,
    FrostIceBlackFragile = 15,
    FrostIceBlackPacked = 16,
    FrostIceFragile = 17,
    FrostIcePacked = 18,
    LiquidAcid = 19,
    LiquidBlood = 20,
    LiquidCoreMagma = 21,
    LiquidLava = 22,
    LiquidMolten = 23,
    LiquidOoze = 24,
    LiquidSludge = 25,
    LiquidSpringWater = 26,
    LiquidTar = 27,
    LiquidWater = 28,

    // --- Category 03: Pedology & Soils ---
    AshPumice = 29,
    AshVolcanic = 30,
    CalicheCoarse = 31,
    CalicheCracked = 32,
    CalicheDirt = 33,
    CalicheGrass = 34,
    ClayBlue = 35,
    ClayRed = 36,
    ClayWhite = 37,
    ClayYellow = 38,
    GravelAshy = 39,
    GravelDirty = 40,
    GravelSandy = 41,
    GravelSnowy = 42,
    LateriteCoarse = 43,
    LateriteDirt = 44,
    LateriteGrass = 45,
    LateriteHardpan = 46,
    LoamCoarse = 47,
    LoamDirt = 48,
    LoamGrass = 49,
    LoamMud = 50,
    LoamMulch = 51,
    LoamRooted = 52,
    LoamSnow = 53,
    LoamTilled = 54,
    MossAmber = 55,
    MossCave = 56,
    MossCrimson = 57,
    MossForest = 58,
    PeatCoarse = 59,
    PeatDirt = 60,
    PeatGrass = 61,
    PeatMud = 62,
    PeatMulch = 63,
    PeatRooted = 64,
    PeatSnow = 65,
    PeatTilled = 66,
    PermafrostCoarse = 67,
    PermafrostDirt = 68,
    PermafrostGrass = 69,
    PermafrostIcy = 70,
    PowderSnow = 71,
    SandBlack = 72,
    SandCommon = 73,
    SandDune = 74,
    SandPink = 75,
    SandRed = 76,
    SandWhite = 77,
    SiltCoarse = 78,
    SiltDirt = 79,
    SiltGrass = 80,
    SiltMud = 81,
    SiltMulch = 82,
    SiltRooted = 83,
    SiltSnow = 84,
    SiltTilled = 85,
    SnowBlock = 86,

    // --- Category 04: Geology & Rocks ---
    Andesite = 87,
    AndesiteGrass = 88,
    AndesiteSnow = 89,
    Argillite = 90,
    ArgilliteGrass = 91,
    ArgilliteSnow = 92,
    Basalt = 93,
    BasaltGrass = 94,
    BasaltSnow = 95,
    Brimstone = 96,
    Calcite = 97,
    Chalk = 98,
    ChalkGrass = 99,
    ChalkSnow = 100,
    CobbledAndesite = 101,
    CobbledArgillite = 102,
    CobbledBasalt = 103,
    CobbledBrimstone = 104,
    CobbledCalcite = 105,
    CobbledChalk = 106,
    CobbledDiorite = 107,
    CobbledDolomite = 108,
    CobbledGabbro = 109,
    CobbledGneiss = 110,
    CobbledGranite = 111,
    CobbledKarst = 112,
    CobbledLimestone = 113,
    CobbledMagma = 114,
    CobbledMarble = 115,
    CobbledObsidian = 116,
    CobbledSandstoneBlack = 117,
    CobbledSandstoneCommon = 118,
    CobbledSandstoneDune = 119,
    CobbledSandstonePink = 120,
    CobbledSandstoneRed = 121,
    CobbledSandstoneWhite = 122,
    CobbledScoria = 123,
    CobbledSerpentinite = 124,
    CobbledSlate = 125,
    CobbledTuff = 126,
    Diorite = 127,
    Dolomite = 128,
    DolomiteGrass = 129,
    DolomiteSnow = 130,
    Gabbro = 131,
    Gneiss = 132,
    GneissGrass = 133,
    GneissSnow = 134,
    Granite = 135,
    GraniteGrass = 136,
    GraniteSnow = 137,
    Karst = 138,
    Limestone = 139,
    LimestoneGrass = 140,
    LimestoneSnow = 141,
    Magma = 142,
    Mantle = 143,
    MantlePlume = 144,
    Marble = 145,
    MossyAndesite = 146,
    MossyArgillite = 147,
    MossyBasalt = 148,
    MossyCobbledAndesite = 149,
    MossyCobbledArgillite = 150,
    MossyCobbledBasalt = 151,
    MossyCobbledDolomite = 152,
    MossyCobbledGneiss = 153,
    MossyCobbledGranite = 154,
    MossyCobbledLimestone = 155,
    MossyCobbledSerpentinite = 156,
    MossyDolomite = 157,
    MossyGneiss = 158,
    MossyGranite = 159,
    MossyLimestone = 160,
    MossySerpentinite = 161,
    Obsidian = 162,
    PebbleFlintOne = 163,
    PebbleFlintThree = 164,
    PebbleFlintTwo = 165,
    PebbleQuartzOne = 166,
    PebbleQuartzThree = 167,
    PebbleQuartzTwo = 168,
    SandstoneBlack = 169,
    SandstoneCommon = 170,
    SandstoneDune = 171,
    SandstonePink = 172,
    SandstoneRed = 173,
    SandstoneWhite = 174,
    Scoria = 175,
    Serpentinite = 176,
    Slate = 177,
    Tuff = 178,

    // --- Category 05: Trees & Forestry ---
    AcaciaLeaves = 179,
    AcaciaLeavesDead = 180,
    AcaciaLog = 181,
    AcaciaPlanks = 182,
    AcaciaWood = 183,
    AspenLeaves = 184,
    AspenLeavesDead = 185,
    AspenLog = 186,
    AspenPlanks = 187,
    AspenWood = 188,
    BambooLargeLeaves = 189,
    BambooPlanks = 190,
    BambooSmallLeaves = 191,
    BambooStalk = 192,
    BaobabLeaves = 193,
    BaobabLeavesDead = 194,
    BaobabLog = 195,
    BaobabPlanks = 196,
    BaobabWood = 197,
    BirchLeaves = 198,
    BirchLeavesDead = 199,
    BirchLog = 200,
    BirchPlanks = 201,
    BirchWood = 202,
    CherryLeaves = 203,
    CherryLeavesDead = 204,
    CherryLog = 205,
    CherryPlanks = 206,
    CherryWood = 207,
    CypressLeaves = 208,
    CypressLeavesDead = 209,
    CypressLog = 210,
    CypressPlanks = 211,
    CypressWood = 212,
    FirLeaves = 213,
    FirLeavesDead = 214,
    FirLog = 215,
    FirPlanks = 216,
    FirWood = 217,
    KapokLeaves = 218,
    KapokLeavesDead = 219,
    KapokLog = 220,
    KapokPlanks = 221,
    KapokWood = 222,
    MahoganyLeaves = 223,
    MahoganyLeavesDead = 224,
    MahoganyLog = 225,
    MahoganyPlanks = 226,
    MahoganyWood = 227,
    MangroveLeaves = 228,
    MangroveLeavesDead = 229,
    MangroveLog = 230,
    MangrovePlanks = 231,
    MangroveRoots = 232,
    MangroveWood = 233,
    MapleLeavesDead = 234,
    MapleLeavesOrange = 235,
    MapleLeavesRed = 236,
    MapleLeavesYellow = 237,
    MapleLog = 238,
    MaplePlanks = 239,
    MapleWood = 240,
    OakLeaves = 241,
    OakLeavesDead = 242,
    OakLog = 243,
    OakPlanks = 244,
    OakWood = 245,
    PalmLeaves = 246,
    PalmLeavesDead = 247,
    PalmLog = 248,
    PalmPlanks = 249,
    PalmWood = 250,
    PineLeaves = 251,
    PineLeavesDead = 252,
    PineLog = 253,
    PinePlanks = 254,
    PineWood = 255,
    RedwoodLeaves = 256,
    RedwoodLeavesDead = 257,
    RedwoodLog = 258,
    RedwoodPlanks = 259,
    RedwoodWood = 260,
    WillowLeaves = 261,
    WillowLeavesDead = 262,
    WillowLog = 263,
    WillowPlanks = 264,
    WillowWood = 265,
    YewLeaves = 266,
    YewLeavesDead = 267,
    YewLog = 268,
    YewPlanks = 269,
    YewWood = 270,

    // --- Category 06: Ores, Minerals & Gemology ---
    GemDiamondGabbro = 271,
    GemDiamondRough = 272,
    GemDiamondSerpentinite = 273,
    GemEmeraldMarble = 274,
    GemEmeraldRough = 275,
    GemEmeraldSlate = 276,
    GemJadeMarble = 277,
    GemJadeRough = 278,
    GemJadeSerpentinite = 279,
    GemOnyxCalcite = 280,
    GemOnyxMarble = 281,
    GemOnyxRough = 282,
    GemOnyxSandstoneBlack = 283,
    GemOnyxSlate = 284,
    GemOpalBasalt = 285,
    GemOpalRough = 286,
    GemOpalSandstonePink = 287,
    GemOpalSandstoneWhite = 288,
    GemOpalTuff = 289,
    GemRubyBasalt = 290,
    GemRubyMarble = 291,
    GemRubyRough = 292,
    GemSapphireBasalt = 293,
    GemSapphireMarble = 294,
    GemSapphireRough = 295,
    GemTopazGranite = 296,
    GemTopazRough = 297,
    GemTopazSandstonePink = 298,
    OreAluminumArgillite = 299,
    OreAluminumChalk = 300,
    OreAluminumLimestone = 301,
    OreAluminumRaw = 302,
    OreAluminumSandstoneWhite = 303,
    OreCinnabarBasalt = 304,
    OreCinnabarBrimstone = 305,
    OreCinnabarKarst = 306,
    OreCinnabarRaw = 307,
    OreCinnabarTuff = 308,
    OreCoalAndesite = 309,
    OreCoalArgillite = 310,
    OreCoalChalk = 311,
    OreCoalDiorite = 312,
    OreCoalDolomite = 313,
    OreCoalGneiss = 314,
    OreCoalGranite = 315,
    OreCoalLimestone = 316,
    OreCoalRaw = 317,
    OreCoalSandstoneBlack = 318,
    OreCoalSandstoneCommon = 319,
    OreCoalSandstoneDune = 320,
    OreCoalSlate = 321,
    OreCopperAndesite = 322,
    OreCopperArgillite = 323,
    OreCopperChalk = 324,
    OreCopperDiorite = 325,
    OreCopperDolomite = 326,
    OreCopperGabbro = 327,
    OreCopperGneiss = 328,
    OreCopperGranite = 329,
    OreCopperLimestone = 330,
    OreCopperRaw = 331,
    OreCopperSandstoneCommon = 332,
    OreCopperSandstoneRed = 333,
    OreCopperSerpentinite = 334,
    OreCopperSlate = 335,
    OreCopperTuff = 336,
    OreGoldGranite = 337,
    OreGoldMarble = 338,
    OreGoldRaw = 339,
    OreGoldSandstoneRed = 340,
    OreGoldSlate = 341,
    OreIronAndesite = 342,
    OreIronBasalt = 343,
    OreIronDiorite = 344,
    OreIronDolomite = 345,
    OreIronGabbro = 346,
    OreIronGneiss = 347,
    OreIronGranite = 348,
    OreIronRaw = 349,
    OreIronSandstoneBlack = 350,
    OreIronSandstoneCommon = 351,
    OreIronSandstoneRed = 352,
    OreIronScoria = 353,
    OreIronSerpentinite = 354,
    OreLazuriteCalcite = 355,
    OreLazuriteLimestone = 356,
    OreLazuriteMarble = 357,
    OreLazuriteRaw = 358,
    OreLazuriteSandstoneBlack = 359,
    OreLeadAndesite = 360,
    OreLeadArgillite = 361,
    OreLeadChalk = 362,
    OreLeadDiorite = 363,
    OreLeadKarst = 364,
    OreLeadLimestone = 365,
    OreLeadRaw = 366,
    OreLeadSandstoneCommon = 367,
    OreLeadSlate = 368,
    OreManganeseArgillite = 369,
    OreManganeseChalk = 370,
    OreManganeseDolomite = 371,
    OreManganeseGneiss = 372,
    OreManganeseLimestone = 373,
    OreManganeseRaw = 374,
    OreManganeseSlate = 375,
    OreNickelBasalt = 376,
    OreNickelGabbro = 377,
    OreNickelRaw = 378,
    OreNickelScoria = 379,
    OreNickelSerpentinite = 380,
    OrePlatinumBasalt = 381,
    OrePlatinumGabbro = 382,
    OrePlatinumRaw = 383,
    OrePlatinumScoria = 384,
    OrePlatinumSerpentinite = 385,
    OreQuartzAndesite = 386,
    OreQuartzGneiss = 387,
    OreQuartzGranite = 388,
    OreQuartzRaw = 389,
    OreQuartzSandstoneCommon = 390,
    OreQuartzSandstoneRed = 391,
    OreQuartzSandstoneWhite = 392,
    OreSaltpeterKarst = 393,
    OreSaltpeterLimestone = 394,
    OreSaltpeterRaw = 395,
    OreSaltpeterSandstoneDune = 396,
    OreSaltpeterSandstoneWhite = 397,
    OreSilverAndesite = 398,
    OreSilverDiorite = 399,
    OreSilverGneiss = 400,
    OreSilverMarble = 401,
    OreSilverRaw = 402,
    OreSulfurBasalt = 403,
    OreSulfurBrimstone = 404,
    OreSulfurRaw = 405,
    OreSulfurScoria = 406,
    OreSulfurTuff = 407,
    OreTinAndesite = 408,
    OreTinDiorite = 409,
    OreTinDolomite = 410,
    OreTinGneiss = 411,
    OreTinGranite = 412,
    OreTinRaw = 413,
    OreTinSandstoneCommon = 414,
    OreUraniumDiorite = 415,
    OreUraniumGabbro = 416,
    OreUraniumGranite = 417,
    OreUraniumRaw = 418,
    OreUraniumSlate = 419,
    OreZincAndesite = 420,
    OreZincArgillite = 421,
    OreZincChalk = 422,
    OreZincDiorite = 423,
    OreZincDolomite = 424,
    OreZincGneiss = 425,
    OreZincKarst = 426,
    OreZincLimestone = 427,
    OreZincRaw = 428,

    // --- Category 07: Speleology & Crystals ---
    CrystalAmethystBlock = 429,
    CrystalAmethystCluster = 430,
    CrystalAmethystGlinted = 431,
    CrystalAquamarineBlock = 432,
    CrystalAquamarineCluster = 433,
    CrystalAquamarineGlinted = 434,
    CrystalCitrineBlock = 435,
    CrystalCitrineCluster = 436,
    CrystalCitrineGlinted = 437,
    CrystalFluoriteBlock = 438,
    CrystalFluoriteCluster = 439,
    CrystalFluoriteGlinted = 440,
    CrystalPinkSaltBlock = 441,
    CrystalPinkSaltCluster = 442,
    CrystalPinkSaltGlinted = 443,
    CrystalSeleniteBlock = 444,
    CrystalSeleniteCluster = 445,
    CrystalSeleniteGlinted = 446,
    DripstoneCalcite = 447,
    DripstoneKarst = 448,
    FossilAmber = 449,
    FossilBitumen = 450,
    FossilBones = 451,
    FossilMatrix = 452,
    SpeleothemBasalt = 453,
    SpeleothemBrimstone = 454,
    SpeleothemIce = 455,
    SpeleothemSaltStraw = 456,

    // --- Category 08: Ocean & Benthic Ecosystems ---
    AlgaeMat = 457,
    AlgaeReefstone = 458,
    BarnaclesReefstone = 459,
    BoneWorms = 460,
    BrainCoral = 461,
    BrainCoralFan = 462,
    BubbleCoral = 463,
    BubbleCoralFan = 464,
    BullKelp = 465,
    Coralstone = 466,
    DeadCoral = 467,
    DeadCoralFan = 468,
    Delesseria = 469,
    DyingCoral = 470,
    DyingCoralFan = 471,
    FireCoral = 472,
    FireCoralFan = 473,
    GeothermalVent = 474,
    HeartCoral = 475,
    HeartCoralFan = 476,
    HornCoral = 477,
    HornCoralFan = 478,
    Kelp = 479,
    LeafCoral = 480,
    LeafCoralFan = 481,
    PearlBlock = 482,
    RedAlgae = 483,
    ReedCoral = 484,
    ReedCoralFan = 485,
    Reefstone = 486,
    Sargassum = 487,
    ScuteBlock = 488,
    SeaAnemone = 489,
    Seagrass = 490,
    Seaweed = 491,
    SpikeCoral = 492,
    SpikeCoralFan = 493,
    Sponge = 494,
    StaghornCoral = 495,
    StaghornCoralFan = 496,
    SunCoral = 497,
    SunCoralFan = 498,
    ThornCoral = 499,
    ThornCoralFan = 500,
    TubeCoral = 501,
    TubeCoralFan = 502,
    WaxCoral = 503,
    WaxCoralFan = 504,
    ZigzagCoral = 505,
    ZigzagCoralFan = 506,

    // --- Category 09: Biomaterials, Fauna & Organic Matter ---
    AntHill = 507,
    AntHillOpen = 508,
    BeaverDam = 509,
    BeeNest = 510,
    BirdNest = 511,
    BloodyBrain = 512,
    BloodyFlesh = 513,
    BlubberBlock = 514,
    BoneBlock = 515,
    BrainBlock = 516,
    CalcinedBone = 517,
    Cobweb = 518,
    CobwebArch = 519,
    CobwebHanging = 520,
    CobwebStretched = 521,
    CoproliteBlock = 522,
    DustBlock = 523,
    FleshBlock = 524,
    FurBlock = 525,
    GuanoBlock = 526,
    HayNest = 527,
    HoneyBlock = 528,
    HoneycombBlock = 529,
    JellyBlock = 530,
    LoamBurrow = 531,
    PeatBurrow = 532,
    PoopBlock = 533,
    SeashellBlock = 534,
    SilkCocoon = 535,
    SiltBurrow = 536,
    SlimeBlock = 537,
    SpiderEgg = 538,
    TermiteMound = 539,
    TermiteMoundOpen = 540,
    WaspNest = 541,
    WaxBlock = 542,

    // --- Category 10: Decorations, Lighting & Architecture ---
    Bag = 543,
    Barrel = 544,
    Basket = 545,
    Bookshelf = 546,
    BookshelfEmpty = 547,
    BrickBlack = 548,
    BrickBlue = 549,
    BrickBrown = 550,
    BrickDarkBlue = 551,
    BrickDarkGrey = 552,
    BrickGreen = 553,
    BrickLightGrey = 554,
    BrickLightPink = 555,
    BrickLime = 556,
    BrickOrange = 557,
    BrickPink = 558,
    BrickPurple = 559,
    BrickRed = 560,
    BrickTurquoise = 561,
    BrickWhite = 562,
    BrickYellow = 563,
    Bricks = 564,
    CornBale = 565,
    Crate = 566,
    EmitLamp = 567,
    EmitLightBlue = 568,
    EmitLightCold = 569,
    EmitLightGreen = 570,
    EmitLightRed = 571,
    EmitLightWarm = 572,
    EmitTorchBlue = 573,
    EmitTorchCold = 574,
    EmitTorchFire = 575,
    EmitTorchGreen = 576,
    EmitTorchRed = 577,
    EmitTorchWarm = 578,
    Fabric = 579,
    FabricBlack = 580,
    FabricBlue = 581,
    FabricBrown = 582,
    FabricDarkBlue = 583,
    FabricDarkGrey = 584,
    FabricGreen = 585,
    FabricLightGrey = 586,
    FabricLightPink = 587,
    FabricLime = 588,
    FabricOrange = 589,
    FabricPink = 590,
    FabricPurple = 591,
    FabricRed = 592,
    FabricTurquoise = 593,
    FabricWhite = 594,
    FabricYellow = 595,
    Glass = 596,
    GlassBlack = 597,
    GlassBlue = 598,
    GlassBrown = 599,
    GlassDarkBlue = 600,
    GlassDarkGrey = 601,
    GlassGreen = 602,
    GlassLightGrey = 603,
    GlassLightPink = 604,
    GlassLime = 605,
    GlassOrange = 606,
    GlassPink = 607,
    GlassPurple = 608,
    GlassRed = 609,
    GlassTurquoise = 610,
    GlassWhite = 611,
    GlassYellow = 612,
    HayBale = 613,
    Linen = 614,
    LinenBlack = 615,
    LinenBlue = 616,
    LinenBrown = 617,
    LinenDarkBlue = 618,
    LinenDarkGrey = 619,
    LinenGreen = 620,
    LinenLightGrey = 621,
    LinenLightPink = 622,
    LinenLime = 623,
    LinenOrange = 624,
    LinenPink = 625,
    LinenPurple = 626,
    LinenRed = 627,
    LinenTurquoise = 628,
    LinenWhite = 629,
    LinenYellow = 630,
    PathDirty = 631,
    PathSandy = 632,
    PathSnowy = 633,
    PathStony = 634,
    Plaster = 635,
    PlasterBlack = 636,
    PlasterBlue = 637,
    PlasterBrown = 638,
    PlasterDarkBlue = 639,
    PlasterDarkGrey = 640,
    PlasterGreen = 641,
    PlasterLightGrey = 642,
    PlasterLightPink = 643,
    PlasterLime = 644,
    PlasterOrange = 645,
    PlasterPink = 646,
    PlasterPurple = 647,
    PlasterRed = 648,
    PlasterTurquoise = 649,
    PlasterWhite = 650,
    PlasterYellow = 651,
    RiceBale = 652,
    Shingles = 653,
    ShinglesBlack = 654,
    ShinglesBlue = 655,
    ShinglesBrown = 656,
    ShinglesDarkBlue = 657,
    ShinglesDarkGrey = 658,
    ShinglesGreen = 659,
    ShinglesLightGrey = 660,
    ShinglesLightPink = 661,
    ShinglesLime = 662,
    ShinglesOrange = 663,
    ShinglesPink = 664,
    ShinglesPurple = 665,
    ShinglesRed = 666,
    ShinglesTurquoise = 667,
    ShinglesWhite = 668,
    ShinglesYellow = 669,
    StrawBale = 670,
    TintedGlass = 671,
    TintedGlassBlack = 672,
    TintedGlassBlue = 673,
    TintedGlassBrown = 674,
    TintedGlassDarkBlue = 675,
    TintedGlassDarkGrey = 676,
    TintedGlassGreen = 677,
    TintedGlassLightGrey = 678,
    TintedGlassLightPink = 679,
    TintedGlassLime = 680,
    TintedGlassOrange = 681,
    TintedGlassPink = 682,
    TintedGlassPurple = 683,
    TintedGlassRed = 684,
    TintedGlassTurquoise = 685,
    TintedGlassWhite = 686,
    TintedGlassYellow = 687,
    Wool = 688,
    WoolBlack = 689,
    WoolBlue = 690,
    WoolBrown = 691,
    WoolDarkBlue = 692,
    WoolDarkGrey = 693,
    WoolGreen = 694,
    WoolLightGrey = 695,
    WoolLightPink = 696,
    WoolLime = 697,
    WoolOrange = 698,
    WoolPink = 699,
    WoolPurple = 700,
    WoolRed = 701,
    WoolTurquoise = 702,
    WoolWhite = 703,
    WoolYellow = 704,

    // --- Category 11: Mycology: Mushrooms ---
    FungiAmanita = 705,
    FungiAmanitaCap = 706,
    FungiChanterelle = 707,
    FungiChanterelleCap = 708,
    FungiIndigo = 709,
    FungiIndigoCap = 710,
    FungiInside = 711,
    FungiMycena = 712,
    FungiMycenaCap = 713,
    FungiPorcini = 714,
    FungiPorciniCap = 715,
    FungiRussula = 716,
    FungiRussulaCap = 717,
    FungiShiitake = 718,
    FungiShiitakeCap = 719,
    FungiStem = 720,
    FungiVerdigris = 721,
    FungiVerdigrisCap = 722,
}

impl Voxel {
    #[inline(always)]
    pub const fn from_u16(val: u16) -> Option<Self> {
        if val <= 722 {
            Some(unsafe { core::mem::transmute::<u16, Self>(val) })
        } else {
            None
        }
    }
}

const fn generate_all_voxels() -> [Voxel; 720] {
    let mut arr = [Voxel::Air; 720];
    let mut i = 0;
    while i < 720 {
        arr[i] = unsafe { core::mem::transmute::<u16, Voxel>((i as u16) + 3) };
        i += 1;
    }
    arr
}

impl Voxel {
    pub const ALL: [Voxel; 720] = generate_all_voxels();
}

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

    if matches!(id, 2 | 26 | 28) {
        flags |= FLAG_WATER;
    }

    if matches!(id, 2 | 10 | 19..=28) {
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

    let is_collidable = !is_empty && !is_fluid && !is_torch && !is_occupied;
    if is_collidable {
        flags |= FLAG_COLLIDABLE;
    }

    // Rocks (87..=178 without pebbles 163..=168) + Ores (271..=428): 250 blocks resolved in 1 single line
    let is_solid_opaque = matches!(id, 87..=162 | 169..=178 | 271..=428)
        || (!is_empty
            && !is_fluid
            && (flags & FLAG_LEAVES) == 0
            && (flags & FLAG_TRANSPARENT) == 0
            && !is_torch
            && (flags & FLAG_BASKET) == 0
            && (flags & FLAG_CROSS_BILLBOARD) == 0
            && !v.is_pebble_const()
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

    let max_fluid_spread = match id {
        2 | 26 | 28 => 8,
        19 => 5,
        10 | 20 => 4,
        21..=25 => 3,
        27 => 2,
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

    let mut idx = 0;
    while idx <= 722 {
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
        matches!(self as u16, 573..=578)
    }

    #[inline(always)]
    pub const fn is_basket_const(self) -> bool {
        matches!(self as u16, 511 | 527 | 545)
    }

    #[inline(always)]
    pub const fn is_pebble_const(self) -> bool {
        matches!(self as u16, 163..=168)
    }

    #[inline(always)]
    pub const fn is_point_light_const(self) -> bool {
        matches!(
            self as u16,
            437 | 440 | 443 | 446 | 474 | 567..=578 | 712..=713
        )
    }

    #[inline(always)]
    pub const fn emission_level(self) -> u8 {
        match self as u16 {
            567..=572 | 19 | 21..=23 | 142 | 144 => 15,
            573..=578 => 14,
            474 => 12,
            712..=713 => 9,
            437 | 440 | 443 | 446 => 8,
            _ => 0,
        }
    }

    pub const fn is_leaves_const(self) -> bool {
        matches!(
            self as u16,
            179..=180
                | 184..=185
                | 189
                | 191
                | 193..=194
                | 198..=199
                | 203..=204
                | 208..=209
                | 213..=214
                | 218..=219
                | 223..=224
                | 228..=229
                | 234..=237
                | 241..=242
                | 246..=247
                | 251..=252
                | 256..=257
                | 261..=262
                | 266..=267
        )
    }

    pub const fn is_cross_billboard_const(self) -> bool {
        matches!(
            self as u16,
            8
                | 430..=431
                | 433..=434
                | 436..=437
                | 439..=440
                | 442..=443
                | 445..=446
                | 460
                | 462
                | 464
                | 468
                | 471
                | 473
                | 476
                | 478
                | 481
                | 485
                | 490
                | 493
                | 496
                | 498
                | 500
                | 502
                | 504
                | 506
                | 518..=521
                | 705
                | 707
                | 709
                | 712
                | 714
                | 716
                | 718
                | 721
        )
    }

    pub const fn is_transparent_const(self) -> bool {
        self.is_leaves_const()
            || matches!(
                self as u16,
                2 | 26 | 28
                    | 11..=15
                    | 17
                    | 272
                    | 275
                    | 292
                    | 295
                    | 297
                    | 429..=446
                    | 460
                    | 462
                    | 464..=465
                    | 468..=469
                    | 471
                    | 473
                    | 476
                    | 478..=479
                    | 481
                    | 485
                    | 490
                    | 493
                    | 496
                    | 498
                    | 500
                    | 502
                    | 504
                    | 506
                    | 514
                    | 518..=521
                    | 528
                    | 530
                    | 537
                    | 596..=612
                    | 671..=687
                    | 705
                    | 707
                    | 709
                    | 712
                    | 714
                    | 716
                    | 718
                    | 721
            )
    }

    pub const fn is_light_const(self) -> bool {
        matches!(
            self as u16,
            19 | 21..=23
                | 142
                | 144
                | 431
                | 434
                | 437
                | 440
                | 443
                | 446
                | 474
                | 567..=578
                | 712..=713
        )
    }
}

static VOXEL_DATA: [(&str, &str, &str); 723] = [
    ("air", "Air", ""),
    ("occupied", "Occupied", ""),
    ("water_occupied", "Waterlogged Occupied", ""),
    ("debug_accept", "Debug Accept Marker", "debug_accept"),
    ("debug_fail", "Debug Fail Marker", "debug_fail"),
    ("debug_instance", "Debug Instance Marker", "debug_instance"),
    ("debug_start", "Debug Start Marker", "debug_start"),
    ("null_block", "Null Block", "null_block"),
    ("null_crossmesh", "Null Crossmesh", "null_crossmesh"),
    ("null_generic", "Null Generic", "null_generic"),
    ("null_liquid", "Null Liquid", "null_liquid"),
    ("frost_blue_ice", "Blue Ice", "frost_blue_ice"),
    (
        "frost_blue_ice_cracked",
        "Cracked Blue Ice",
        "frost_blue_ice_cracked",
    ),
    ("frost_ice", "Clear Ice", "frost_ice"),
    ("frost_ice_black", "Black Ice", "frost_ice_black"),
    (
        "frost_ice_black_fragile",
        "Fragile Black Ice",
        "frost_ice_black_fragile",
    ),
    (
        "frost_ice_black_packed",
        "Packed Black Ice",
        "frost_ice_black_packed",
    ),
    ("frost_ice_fragile", "Fragile Ice", "frost_ice_fragile"),
    ("frost_ice_packed", "Packed Ice", "frost_ice_packed"),
    ("liquid_acid", "Acid", "liquid_acid_still"),
    ("liquid_blood", "Blood", "liquid_blood_still"),
    ("liquid_core_magma", "Core Magma", "liquid_core_magma_still"),
    ("liquid_lava", "Lava", "liquid_lava_still"),
    ("liquid_molten", "Molten Slag", "liquid_molten_still"),
    ("liquid_ooze", "Ooze", "liquid_ooze_still"),
    ("liquid_sludge", "Sludge", "liquid_sludge_still"),
    (
        "liquid_spring_water",
        "Spring Water",
        "liquid_spring_water_still",
    ),
    ("liquid_tar", "Tar", "liquid_tar_still"),
    ("liquid_water", "Water", "liquid_water_still"),
    ("ash_pumice", "Pumice Ash", "soil_ash_pumice"),
    ("ash_volcanic", "Volcanic Ash", "soil_ash_vulcanic"),
    ("caliche_coarse", "Caliche Coarse", "soil_caliche_coarse"),
    ("caliche_cracked", "Caliche Cracked", "soil_caliche_cracked"),
    ("caliche_dirt", "Caliche Dirt", "soil_caliche_dirt"),
    ("caliche_grass", "Caliche Grass", "soil_caliche_dirt"),
    ("clay_blue", "Blue Clay", "soil_clay_blue"),
    ("clay_red", "Red Clay", "soil_clay_red"),
    ("clay_white", "White Clay", "soil_clay_white"),
    ("clay_yellow", "Yellow Clay", "soil_clay_yellow"),
    ("gravel_ashy", "Ashy Gravel", "soil_gravel_ashy"),
    ("gravel_dirty", "Dirty Gravel", "soil_gravel_dirty"),
    ("gravel_sandy", "Sandy Gravel", "soil_gravel_sandy"),
    ("gravel_snowy", "Snowy Gravel", "soil_gravel_snowy"),
    ("laterite_coarse", "Laterite Coarse", "soil_laterite_coarse"),
    ("laterite_dirt", "Laterite Dirt", "soil_laterite_dirt"),
    ("laterite_grass", "Laterite Grass", "soil_laterite_dirt"),
    (
        "laterite_hardpan",
        "Laterite Hardpan",
        "soil_laterite_hardpan",
    ),
    ("loam_coarse", "Loam Coarse", "soil_loam_coarse"),
    ("loam_dirt", "Loam Dirt", "soil_loam_dirt"),
    ("loam_grass", "Loam Grass", "soil_loam_dirt"),
    ("loam_mud", "Loam Mud", "soil_loam_mud"),
    ("loam_mulch", "Loam Mulch", "soil_loam_dirt"),
    ("loam_rooted", "Loam Rooted", "soil_loam_rooted"),
    ("loam_snow", "Loam Snow", "soil_loam_dirt"),
    ("loam_tilled", "Loam Tilled", "soil_loam_dirt"),
    ("moss_amber", "Amber Moss", "soil_moss_amber"),
    ("moss_cave", "Cave Moss", "soil_moss_cave"),
    ("moss_crimson", "Crimson Moss", "soil_moss_crimson"),
    ("moss_forest", "Forest Moss", "soil_moss_forest"),
    ("peat_coarse", "Peat Coarse", "soil_peat_coarse"),
    ("peat_dirt", "Peat Dirt", "soil_peat_dirt"),
    ("peat_grass", "Peat Grass", "soil_peat_dirt"),
    ("peat_mud", "Peat Mud", "soil_peat_mud"),
    ("peat_mulch", "Peat Mulch", "soil_peat_dirt"),
    ("peat_rooted", "Peat Rooted", "soil_peat_rooted"),
    ("peat_snow", "Peat Snow", "soil_peat_dirt"),
    ("peat_tilled", "Peat Tilled", "soil_peat_dirt"),
    (
        "permafrost_coarse",
        "Permafrost Coarse",
        "soil_permafrost_coarse",
    ),
    ("permafrost_dirt", "Permafrost Dirt", "soil_permafrost_dirt"),
    (
        "permafrost_grass",
        "Permafrost Grass",
        "soil_permafrost_dirt",
    ),
    ("permafrost_icy", "Permafrost Icy", "soil_permafrost_icy"),
    ("powder_snow", "Powder Snow", "soil_snow_powder"),
    ("sand_black", "Black Sand", "soil_sand_black"),
    ("sand_common", "Common Sand", "soil_sand_common"),
    ("sand_dune", "Dune Sand", "soil_sand_dune"),
    ("sand_pink", "Pink Sand", "soil_sand_pink"),
    ("sand_red", "Red Sand", "soil_sand_red"),
    ("sand_white", "White Sand", "soil_sand_white"),
    ("silt_coarse", "Silt Coarse", "soil_silt_coarse"),
    ("silt_dirt", "Silt Dirt", "soil_silt_dirt"),
    ("silt_grass", "Silt Grass", "soil_silt_dirt"),
    ("silt_mud", "Silt Mud", "soil_silt_mud"),
    ("silt_mulch", "Silt Mulch", "soil_silt_dirt"),
    ("silt_rooted", "Silt Rooted", "soil_silt_rooted"),
    ("silt_snow", "Silt Snow", "soil_silt_dirt"),
    ("silt_tilled", "Silt Tilled", "soil_silt_dirt"),
    ("snow_block", "Snow Block", "soil_snow"),
    ("andesite", "Andesite", "rock_andesite"),
    ("andesite_grass", "Andesite Grass Outcrop", "rock_andesite"),
    ("andesite_snow", "Andesite Snow Outcrop", "rock_andesite"),
    ("argillite", "Argillite", "rock_argillite"),
    (
        "argillite_grass",
        "Argillite Grass Outcrop",
        "rock_argillite",
    ),
    ("argillite_snow", "Argillite Snow Outcrop", "rock_argillite"),
    ("basalt", "Basalt", "rock_basalt"),
    ("basalt_grass", "Basalt Grass Outcrop", "rock_basalt"),
    ("basalt_snow", "Basalt Snow Outcrop", "rock_basalt"),
    ("brimstone", "Brimstone", "rock_brimstone"),
    ("calcite", "Calcite", "rock_calcite"),
    ("chalk", "Chalk", "rock_chalk"),
    ("chalk_grass", "Chalk Grass Outcrop", "rock_chalk"),
    ("chalk_snow", "Chalk Snow Outcrop", "rock_chalk"),
    (
        "cobbled_andesite",
        "Cobbled Andesite",
        "rock_andesite_cobbled",
    ),
    (
        "cobbled_argillite",
        "Cobbled Argillite",
        "rock_argillite_cobbled",
    ),
    ("cobbled_basalt", "Cobbled Basalt", "rock_basalt_cobbled"),
    (
        "cobbled_brimstone",
        "Cobbled Brimstone",
        "rock_brimstone_cobbled",
    ),
    ("cobbled_calcite", "Cobbled Calcite", "rock_calcite_cobbled"),
    ("cobbled_chalk", "Cobbled Chalk", "rock_chalk_cobbled"),
    ("cobbled_diorite", "Cobbled Diorite", "rock_diorite_cobbled"),
    (
        "cobbled_dolomite",
        "Cobbled Dolomite",
        "rock_dolomite_cobbled",
    ),
    ("cobbled_gabbro", "Cobbled Gabbro", "rock_gabbro_cobbled"),
    ("cobbled_gneiss", "Cobbled Gneiss", "rock_gneiss_cobbled"),
    ("cobbled_granite", "Cobbled Granite", "rock_granite_cobbled"),
    ("cobbled_karst", "Cobbled Karst", "rock_karst_cobbled"),
    (
        "cobbled_limestone",
        "Cobbled Limestone",
        "rock_limestone_cobbled",
    ),
    ("cobbled_magma", "Cobbled Magma", "rock_magma_cobbled"),
    ("cobbled_marble", "Cobbled Marble", "rock_marble_cobbled"),
    (
        "cobbled_obsidian",
        "Cobbled Obsidian",
        "rock_obsidian_cobbled",
    ),
    (
        "cobbled_sandstone_black",
        "Cobbled Black Sandstone",
        "rock_sandstone_black_cobbled",
    ),
    (
        "cobbled_sandstone_common",
        "Cobbled Sandstone",
        "rock_sandstone_common_cobbled",
    ),
    (
        "cobbled_sandstone_dune",
        "Cobbled Dune Sandstone",
        "rock_sandstone_dune_cobbled",
    ),
    (
        "cobbled_sandstone_pink",
        "Cobbled Pink Sandstone",
        "rock_sandstone_pink_cobbled",
    ),
    (
        "cobbled_sandstone_red",
        "Cobbled Red Sandstone",
        "rock_sandstone_red_cobbled",
    ),
    (
        "cobbled_sandstone_white",
        "Cobbled White Sandstone",
        "rock_sandstone_white_cobbled",
    ),
    ("cobbled_scoria", "Cobbled Scoria", "rock_scoria_cobbled"),
    (
        "cobbled_serpentinite",
        "Cobbled Serpentinite",
        "rock_serpentinite_cobbled",
    ),
    ("cobbled_slate", "Cobbled Slate", "rock_slate_cobbled"),
    ("cobbled_tuff", "Cobbled Tuff", "rock_tuff_cobbled"),
    ("diorite", "Diorite", "rock_diorite"),
    ("dolomite", "Dolomite (Alpine Stone)", "rock_dolomite"),
    ("dolomite_grass", "Dolomite Grass Outcrop", "rock_dolomite"),
    ("dolomite_snow", "Dolomite Snow Outcrop", "rock_dolomite"),
    ("gabbro", "Gabbro", "rock_gabbro"),
    ("gneiss", "Gneiss", "rock_gneiss"),
    ("gneiss_grass", "Gneiss Grass Outcrop", "rock_gneiss"),
    ("gneiss_snow", "Gneiss Snow Outcrop", "rock_gneiss"),
    ("granite", "Granite", "rock_granite"),
    ("granite_grass", "Granite Grass Outcrop", "rock_granite"),
    ("granite_snow", "Granite Snow Outcrop", "rock_granite"),
    ("karst", "Karst Limestone", "rock_karst"),
    ("limestone", "Limestone", "rock_limestone"),
    (
        "limestone_grass",
        "Limestone Grass Outcrop",
        "rock_limestone",
    ),
    ("limestone_snow", "Limestone Snow Outcrop", "rock_limestone"),
    ("magma", "Magma Rock", "rock_magma"),
    ("mantle", "Primordial Mantle", "rock_mantle"),
    ("mantle_plume", "Mantle Plume", "rock_mantle_plume"),
    ("marble", "Marble", "rock_marble"),
    ("mossy_andesite", "Mossy Andesite", "rock_andesite"),
    ("mossy_argillite", "Mossy Argillite", "rock_argillite"),
    ("mossy_basalt", "Mossy Basalt", "rock_basalt"),
    (
        "mossy_cobbled_andesite",
        "Mossy Cobbled Andesite",
        "rock_andesite_cobbled",
    ),
    (
        "mossy_cobbled_argillite",
        "Mossy Cobbled Argillite",
        "rock_argillite_cobbled",
    ),
    (
        "mossy_cobbled_basalt",
        "Mossy Cobbled Basalt",
        "rock_basalt_cobbled",
    ),
    (
        "mossy_cobbled_dolomite",
        "Mossy Cobbled Dolomite",
        "rock_dolomite_cobbled",
    ),
    (
        "mossy_cobbled_gneiss",
        "Mossy Cobbled Gneiss",
        "rock_gneiss_cobbled",
    ),
    (
        "mossy_cobbled_granite",
        "Mossy Cobbled Granite",
        "rock_granite_cobbled",
    ),
    (
        "mossy_cobbled_limestone",
        "Mossy Cobbled Limestone",
        "rock_limestone_cobbled",
    ),
    (
        "mossy_cobbled_serpentinite",
        "Mossy Cobbled Serpentinite",
        "rock_serpentinite_cobbled",
    ),
    ("mossy_dolomite", "Mossy Dolomite", "rock_dolomite"),
    ("mossy_gneiss", "Mossy Gneiss", "rock_gneiss"),
    ("mossy_granite", "Mossy Granite", "rock_granite"),
    ("mossy_limestone", "Mossy Limestone", "rock_limestone"),
    (
        "mossy_serpentinite",
        "Mossy Serpentinite",
        "rock_serpentinite",
    ),
    ("obsidian", "Obsidian", "rock_obsidian"),
    (
        "pebble_flint_one",
        "Flint Pebble (Single)",
        "pebble_flint_one",
    ),
    (
        "pebble_flint_three",
        "Flint Pebble (Cluster)",
        "pebble_flint_three",
    ),
    (
        "pebble_flint_two",
        "Flint Pebble (Pair)",
        "pebble_flint_two",
    ),
    (
        "pebble_quartz_one",
        "Quartz Pebble (Single)",
        "pebble_quartz_one",
    ),
    (
        "pebble_quartz_three",
        "Quartz Pebble (Cluster)",
        "pebble_quartz_three",
    ),
    (
        "pebble_quartz_two",
        "Quartz Pebble (Pair)",
        "pebble_quartz_two",
    ),
    ("sandstone_black", "Black Sandstone", "rock_sandstone_black"),
    (
        "sandstone_common",
        "Common Sandstone",
        "rock_sandstone_common",
    ),
    ("sandstone_dune", "Dune Sandstone", "rock_sandstone_dune"),
    ("sandstone_pink", "Pink Sandstone", "rock_sandstone_pink"),
    ("sandstone_red", "Red Sandstone", "rock_sandstone_red"),
    ("sandstone_white", "White Sandstone", "rock_sandstone_white"),
    ("scoria", "Scoria", "rock_scoria"),
    ("serpentinite", "Serpentinite", "rock_serpentinite"),
    ("slate", "Slate", "rock_slate"),
    ("tuff", "Tuff", "rock_tuff"),
    ("acacia_leaves", "Acacia Leaves", "tree_acacia_leaves"),
    (
        "acacia_leaves_dead",
        "Dead Acacia Leaves",
        "tree_acacia_leaves_dead",
    ),
    ("acacia_log", "Acacia Log", "tree_acacia_bark"),
    ("acacia_planks", "Acacia Planks", "tree_acacia_planks"),
    ("acacia_wood", "Acacia Wood", "tree_acacia_bark"),
    ("aspen_leaves", "Golden Aspen Leaves", "tree_aspen_leaves"),
    (
        "aspen_leaves_dead",
        "Dead Aspen Leaves",
        "tree_aspen_leaves_dead",
    ),
    ("aspen_log", "Aspen Log", "tree_aspen_bark"),
    ("aspen_planks", "Aspen Planks", "tree_aspen_planks"),
    ("aspen_wood", "Aspen Wood", "tree_aspen_bark"),
    (
        "bamboo_large_leaves",
        "Large Bamboo Leaves",
        "tree_bamboo_large_leaves",
    ),
    ("bamboo_planks", "Bamboo Planks", "tree_bamboo_planks"),
    (
        "bamboo_small_leaves",
        "Small Bamboo Leaves",
        "tree_bamboo_small_leaves",
    ),
    ("bamboo_stalk", "Bamboo Stalk", "tree_bamboo_stalk"),
    ("baobab_leaves", "Baobab Leaves", "tree_baobab_leaves"),
    (
        "baobab_leaves_dead",
        "Dead Baobab Leaves",
        "tree_baobab_leaves_dead",
    ),
    ("baobab_log", "Baobab Log", "tree_baobab_bark"),
    ("baobab_planks", "Baobab Planks", "tree_baobab_planks"),
    ("baobab_wood", "Baobab Wood", "tree_baobab_bark"),
    ("birch_leaves", "Birch Leaves", "tree_birch_leaves"),
    (
        "birch_leaves_dead",
        "Dead Birch Leaves",
        "tree_birch_leaves_dead",
    ),
    ("birch_log", "Birch Log", "tree_birch_bark"),
    ("birch_planks", "Birch Planks", "tree_birch_planks"),
    ("birch_wood", "Birch Wood", "tree_birch_bark"),
    (
        "cherry_leaves",
        "Cherry Blossom Leaves",
        "tree_cherry_leaves",
    ),
    (
        "cherry_leaves_dead",
        "Dead Cherry Leaves",
        "tree_cherry_leaves_dead",
    ),
    ("cherry_log", "Cherry Log", "tree_cherry_bark"),
    ("cherry_planks", "Cherry Planks", "tree_cherry_planks"),
    ("cherry_wood", "Cherry Wood", "tree_cherry_bark"),
    ("cypress_leaves", "Cypress Foliage", "tree_cypress_leaves"),
    (
        "cypress_leaves_dead",
        "Dead Cypress Foliage",
        "tree_cypress_leaves_dead",
    ),
    ("cypress_log", "Cypress Log", "tree_cypress_bark"),
    ("cypress_planks", "Cypress Planks", "tree_cypress_planks"),
    ("cypress_wood", "Cypress Wood", "tree_cypress_bark"),
    ("fir_leaves", "Fir Needles", "tree_fir_leaves"),
    (
        "fir_leaves_dead",
        "Dead Fir Needles",
        "tree_fir_leaves_dead",
    ),
    ("fir_log", "Fir Log", "tree_fir_bark"),
    ("fir_planks", "Fir Planks", "tree_fir_planks"),
    ("fir_wood", "Fir Wood", "tree_fir_bark"),
    ("kapok_leaves", "Kapok Leaves", "tree_kapok_leaves"),
    (
        "kapok_leaves_dead",
        "Dead Kapok Leaves",
        "tree_kapok_leaves_dead",
    ),
    ("kapok_log", "Kapok Log", "tree_kapok_bark"),
    ("kapok_planks", "Kapok Planks", "tree_kapok_planks"),
    ("kapok_wood", "Kapok Wood", "tree_kapok_bark"),
    ("mahogany_leaves", "Mahogany Leaves", "tree_mahogany_leaves"),
    (
        "mahogany_leaves_dead",
        "Dead Mahogany Leaves",
        "tree_mahogany_leaves_dead",
    ),
    ("mahogany_log", "Mahogany Log", "tree_mahogany_bark"),
    ("mahogany_planks", "Mahogany Planks", "tree_mahogany_planks"),
    ("mahogany_wood", "Mahogany Wood", "tree_mahogany_bark"),
    ("mangrove_leaves", "Mangrove Leaves", "tree_mangrove_leaves"),
    (
        "mangrove_leaves_dead",
        "Dead Mangrove Leaves",
        "tree_mangrove_leaves_dead",
    ),
    ("mangrove_log", "Mangrove Log", "tree_mangrove_bark"),
    ("mangrove_planks", "Mangrove Planks", "tree_mangrove_planks"),
    (
        "mangrove_roots",
        "Mangrove Aerial Roots",
        "tree_mangrove_roots",
    ),
    ("mangrove_wood", "Mangrove Wood", "tree_mangrove_bark"),
    (
        "maple_leaves_dead",
        "Dead Maple Leaves",
        "tree_maple_leaves_dead",
    ),
    (
        "maple_leaves_orange",
        "Orange Maple Leaves",
        "tree_maple_leaves_orange",
    ),
    (
        "maple_leaves_red",
        "Red Maple Leaves",
        "tree_maple_leaves_red",
    ),
    (
        "maple_leaves_yellow",
        "Yellow Maple Leaves",
        "tree_maple_leaves_yellow",
    ),
    ("maple_log", "Maple Log", "tree_maple_bark"),
    ("maple_planks", "Maple Planks", "tree_maple_planks"),
    ("maple_wood", "Maple Wood", "tree_maple_bark"),
    ("oak_leaves", "Oak Leaves", "tree_oak_leaves"),
    ("oak_leaves_dead", "Dead Oak Leaves", "tree_oak_leaves_dead"),
    ("oak_log", "Oak Log", "tree_oak_bark"),
    ("oak_planks", "Oak Planks", "tree_oak_planks"),
    ("oak_wood", "Oak Wood", "tree_oak_bark"),
    ("palm_leaves", "Palm Fronds", "tree_palm_leaves"),
    (
        "palm_leaves_dead",
        "Dead Palm Fronds",
        "tree_palm_leaves_dead",
    ),
    ("palm_log", "Palm Log", "tree_palm_bark"),
    ("palm_planks", "Palm Planks", "tree_palm_planks"),
    ("palm_wood", "Palm Wood", "tree_palm_bark"),
    ("pine_leaves", "Pine Needles", "tree_pine_leaves"),
    (
        "pine_leaves_dead",
        "Dead Pine Needles",
        "tree_pine_leaves_dead",
    ),
    ("pine_log", "Pine Log", "tree_pine_bark"),
    ("pine_planks", "Pine Planks", "tree_pine_planks"),
    ("pine_wood", "Pine Wood", "tree_pine_bark"),
    ("redwood_leaves", "Redwood Foliage", "tree_redwood_leaves"),
    (
        "redwood_leaves_dead",
        "Dead Redwood Foliage",
        "tree_redwood_leaves_dead",
    ),
    ("redwood_log", "Redwood Log", "tree_redwood_bark"),
    ("redwood_planks", "Redwood Planks", "tree_redwood_planks"),
    ("redwood_wood", "Redwood Wood", "tree_redwood_bark"),
    (
        "willow_leaves",
        "Weeping Willow Leaves",
        "tree_willow_leaves",
    ),
    (
        "willow_leaves_dead",
        "Dead Willow Leaves",
        "tree_willow_leaves_dead",
    ),
    ("willow_log", "Willow Log", "tree_willow_bark"),
    ("willow_planks", "Willow Planks", "tree_willow_planks"),
    ("willow_wood", "Willow Wood", "tree_willow_bark"),
    ("yew_leaves", "Yew Foliage", "tree_yew_leaves"),
    (
        "yew_leaves_dead",
        "Dead Yew Foliage",
        "tree_yew_leaves_dead",
    ),
    ("yew_log", "Yew Log", "tree_yew_bark"),
    ("yew_planks", "Yew Planks", "tree_yew_planks"),
    ("yew_wood", "Yew Wood", "tree_yew_bark"),
    ("gem_diamond_gabbro", "Diamond Ore (Gabbro)", "rock_gabbro"),
    (
        "gem_diamond_rough",
        "Rough Diamond Block",
        "gem_diamond_rough",
    ),
    (
        "gem_diamond_serpentinite",
        "Diamond Ore (Serpentinite)",
        "rock_serpentinite",
    ),
    ("gem_emerald_marble", "Emerald Ore (Marble)", "rock_marble"),
    (
        "gem_emerald_rough",
        "Rough Emerald Block",
        "gem_emerald_rough",
    ),
    ("gem_emerald_slate", "Emerald Ore (Slate)", "rock_slate"),
    ("gem_jade_marble", "Jade Ore (Marble)", "rock_marble"),
    ("gem_jade_rough", "Rough Jade Block", "gem_jade_rough"),
    (
        "gem_jade_serpentinite",
        "Jade Ore (Serpentinite)",
        "rock_serpentinite",
    ),
    ("gem_onyx_calcite", "Onyx Ore (Calcite)", "rock_calcite"),
    ("gem_onyx_marble", "Onyx Ore (Marble)", "rock_marble"),
    ("gem_onyx_rough", "Rough Onyx Block", "gem_onyx_rough"),
    (
        "gem_onyx_sandstone_black",
        "Onyx Ore (Black Sandstone)",
        "rock_sandstone_black",
    ),
    ("gem_onyx_slate", "Onyx Ore (Slate)", "rock_slate"),
    ("gem_opal_basalt", "Opal Ore (Basalt)", "rock_basalt"),
    ("gem_opal_rough", "Rough Opal Block", "gem_opal_rough"),
    (
        "gem_opal_sandstone_pink",
        "Opal Ore (Pink Sandstone)",
        "rock_sandstone_pink",
    ),
    (
        "gem_opal_sandstone_white",
        "Opal Ore (White Sandstone)",
        "rock_sandstone_white",
    ),
    ("gem_opal_tuff", "Opal Ore (Tuff)", "rock_tuff"),
    ("gem_ruby_basalt", "Ruby Ore (Basalt)", "rock_basalt"),
    ("gem_ruby_marble", "Ruby Ore (Marble)", "rock_marble"),
    ("gem_ruby_rough", "Rough Ruby Block", "gem_ruby_rough"),
    (
        "gem_sapphire_basalt",
        "Sapphire Ore (Basalt)",
        "rock_basalt",
    ),
    (
        "gem_sapphire_marble",
        "Sapphire Ore (Marble)",
        "rock_marble",
    ),
    (
        "gem_sapphire_rough",
        "Rough Sapphire Block",
        "gem_sapphire_rough",
    ),
    ("gem_topaz_granite", "Topaz Ore (Granite)", "rock_granite"),
    ("gem_topaz_rough", "Rough Topaz Block", "gem_topaz_rough"),
    (
        "gem_topaz_sandstone_pink",
        "Topaz Ore (Pink Sandstone)",
        "rock_sandstone_pink",
    ),
    (
        "ore_aluminum_argillite",
        "Aluminum Ore (Argillite)",
        "rock_argillite",
    ),
    ("ore_aluminum_chalk", "Aluminum Ore (Chalk)", "rock_chalk"),
    (
        "ore_aluminum_limestone",
        "Aluminum Ore (Limestone)",
        "rock_limestone",
    ),
    ("ore_aluminum_raw", "Raw Aluminum Block", "ore_aluminum_raw"),
    (
        "ore_aluminum_sandstone_white",
        "Aluminum Ore (White Sandstone)",
        "rock_sandstone_white",
    ),
    (
        "ore_cinnabar_basalt",
        "Cinnabar Ore (Basalt)",
        "rock_basalt",
    ),
    (
        "ore_cinnabar_brimstone",
        "Cinnabar Ore (Brimstone)",
        "rock_brimstone",
    ),
    ("ore_cinnabar_karst", "Cinnabar Ore (Karst)", "rock_karst"),
    ("ore_cinnabar_raw", "Raw Cinnabar Block", "ore_cinnabar_raw"),
    ("ore_cinnabar_tuff", "Cinnabar Ore (Tuff)", "rock_tuff"),
    ("ore_coal_andesite", "Coal Ore (Andesite)", "rock_andesite"),
    (
        "ore_coal_argillite",
        "Coal Ore (Argillite)",
        "rock_argillite",
    ),
    ("ore_coal_chalk", "Coal Ore (Chalk)", "rock_chalk"),
    ("ore_coal_diorite", "Coal Ore (Diorite)", "rock_diorite"),
    ("ore_coal_dolomite", "Coal Ore (Dolomite)", "rock_dolomite"),
    ("ore_coal_gneiss", "Coal Ore (Gneiss)", "rock_gneiss"),
    ("ore_coal_granite", "Coal Ore (Granite)", "rock_granite"),
    (
        "ore_coal_limestone",
        "Coal Ore (Limestone)",
        "rock_limestone",
    ),
    ("ore_coal_raw", "Raw Coal Block", "ore_coal_raw"),
    (
        "ore_coal_sandstone_black",
        "Coal Ore (Black Sandstone)",
        "rock_sandstone_black",
    ),
    (
        "ore_coal_sandstone_common",
        "Coal Ore (Common Sandstone)",
        "rock_sandstone_common",
    ),
    (
        "ore_coal_sandstone_dune",
        "Coal Ore (Dune Sandstone)",
        "rock_sandstone_dune",
    ),
    ("ore_coal_slate", "Coal Ore (Slate)", "rock_slate"),
    (
        "ore_copper_andesite",
        "Copper Ore (Andesite)",
        "rock_andesite",
    ),
    (
        "ore_copper_argillite",
        "Copper Ore (Argillite)",
        "rock_argillite",
    ),
    ("ore_copper_chalk", "Copper Ore (Chalk)", "rock_chalk"),
    ("ore_copper_diorite", "Copper Ore (Diorite)", "rock_diorite"),
    (
        "ore_copper_dolomite",
        "Copper Ore (Dolomite)",
        "rock_dolomite",
    ),
    ("ore_copper_gabbro", "Copper Ore (Gabbro)", "rock_gabbro"),
    ("ore_copper_gneiss", "Copper Ore (Gneiss)", "rock_gneiss"),
    ("ore_copper_granite", "Copper Ore (Granite)", "rock_granite"),
    (
        "ore_copper_limestone",
        "Copper Ore (Limestone)",
        "rock_limestone",
    ),
    ("ore_copper_raw", "Raw Copper Block", "ore_copper_raw"),
    (
        "ore_copper_sandstone_common",
        "Copper Ore (Common Sandstone)",
        "rock_sandstone_common",
    ),
    (
        "ore_copper_sandstone_red",
        "Copper Ore (Red Sandstone)",
        "rock_sandstone_red",
    ),
    (
        "ore_copper_serpentinite",
        "Copper Ore (Serpentinite)",
        "rock_serpentinite",
    ),
    ("ore_copper_slate", "Copper Ore (Slate)", "rock_slate"),
    ("ore_copper_tuff", "Copper Ore (Tuff)", "rock_tuff"),
    ("ore_gold_granite", "Gold Ore (Granite)", "rock_granite"),
    ("ore_gold_marble", "Gold Ore (Marble)", "rock_marble"),
    ("ore_gold_raw", "Raw Gold Block", "ore_gold_raw"),
    (
        "ore_gold_sandstone_red",
        "Gold Ore (Red Sandstone)",
        "rock_sandstone_red",
    ),
    ("ore_gold_slate", "Gold Ore (Slate)", "rock_slate"),
    ("ore_iron_andesite", "Iron Ore (Andesite)", "rock_andesite"),
    ("ore_iron_basalt", "Iron Ore (Basalt)", "rock_basalt"),
    ("ore_iron_diorite", "Iron Ore (Diorite)", "rock_diorite"),
    ("ore_iron_dolomite", "Iron Ore (Dolomite)", "rock_dolomite"),
    ("ore_iron_gabbro", "Iron Ore (Gabbro)", "rock_gabbro"),
    ("ore_iron_gneiss", "Iron Ore (Gneiss)", "rock_gneiss"),
    ("ore_iron_granite", "Iron Ore (Granite)", "rock_granite"),
    ("ore_iron_raw", "Raw Iron Block", "ore_iron_raw"),
    (
        "ore_iron_sandstone_black",
        "Iron Ore (Black Sandstone)",
        "rock_sandstone_black",
    ),
    (
        "ore_iron_sandstone_common",
        "Iron Ore (Common Sandstone)",
        "rock_sandstone_common",
    ),
    (
        "ore_iron_sandstone_red",
        "Iron Ore (Red Sandstone)",
        "rock_sandstone_red",
    ),
    ("ore_iron_scoria", "Iron Ore (Scoria)", "rock_scoria"),
    (
        "ore_iron_serpentinite",
        "Iron Ore (Serpentinite)",
        "rock_serpentinite",
    ),
    (
        "ore_lazurite_calcite",
        "Lazurite Ore (Calcite)",
        "rock_calcite",
    ),
    (
        "ore_lazurite_limestone",
        "Lazurite Ore (Limestone)",
        "rock_limestone",
    ),
    (
        "ore_lazurite_marble",
        "Lazurite Ore (Marble)",
        "rock_marble",
    ),
    ("ore_lazurite_raw", "Raw Lazurite Block", "ore_lazurite_raw"),
    (
        "ore_lazurite_sandstone_black",
        "Lazurite Ore (Black Sandstone)",
        "rock_sandstone_black",
    ),
    ("ore_lead_andesite", "Lead Ore (Andesite)", "rock_andesite"),
    (
        "ore_lead_argillite",
        "Lead Ore (Argillite)",
        "rock_argillite",
    ),
    ("ore_lead_chalk", "Lead Ore (Chalk)", "rock_chalk"),
    ("ore_lead_diorite", "Lead Ore (Diorite)", "rock_diorite"),
    ("ore_lead_karst", "Lead Ore (Karst)", "rock_karst"),
    (
        "ore_lead_limestone",
        "Lead Ore (Limestone)",
        "rock_limestone",
    ),
    ("ore_lead_raw", "Raw Lead Block", "ore_lead_raw"),
    (
        "ore_lead_sandstone_common",
        "Lead Ore (Common Sandstone)",
        "rock_sandstone_common",
    ),
    ("ore_lead_slate", "Lead Ore (Slate)", "rock_slate"),
    (
        "ore_manganese_argillite",
        "Manganese Ore (Argillite)",
        "rock_argillite",
    ),
    ("ore_manganese_chalk", "Manganese Ore (Chalk)", "rock_chalk"),
    (
        "ore_manganese_dolomite",
        "Manganese Ore (Dolomite)",
        "rock_dolomite",
    ),
    (
        "ore_manganese_gneiss",
        "Manganese Ore (Gneiss)",
        "rock_gneiss",
    ),
    (
        "ore_manganese_limestone",
        "Manganese Ore (Limestone)",
        "rock_limestone",
    ),
    (
        "ore_manganese_raw",
        "Raw Manganese Block",
        "ore_manganese_raw",
    ),
    ("ore_manganese_slate", "Manganese Ore (Slate)", "rock_slate"),
    ("ore_nickel_basalt", "Nickel Ore (Basalt)", "rock_basalt"),
    ("ore_nickel_gabbro", "Nickel Ore (Gabbro)", "rock_gabbro"),
    ("ore_nickel_raw", "Raw Nickel Block", "ore_nickel_raw"),
    ("ore_nickel_scoria", "Nickel Ore (Scoria)", "rock_scoria"),
    (
        "ore_nickel_serpentinite",
        "Nickel Ore (Serpentinite)",
        "rock_serpentinite",
    ),
    (
        "ore_platinum_basalt",
        "Platinum Ore (Basalt)",
        "rock_basalt",
    ),
    (
        "ore_platinum_gabbro",
        "Platinum Ore (Gabbro)",
        "rock_gabbro",
    ),
    ("ore_platinum_raw", "Raw Platinum Block", "ore_platinum_raw"),
    (
        "ore_platinum_scoria",
        "Platinum Ore (Scoria)",
        "rock_scoria",
    ),
    (
        "ore_platinum_serpentinite",
        "Platinum Ore (Serpentinite)",
        "rock_serpentinite",
    ),
    (
        "ore_quartz_andesite",
        "Quartz Ore (Andesite)",
        "rock_andesite",
    ),
    ("ore_quartz_gneiss", "Quartz Ore (Gneiss)", "rock_gneiss"),
    ("ore_quartz_granite", "Quartz Ore (Granite)", "rock_granite"),
    ("ore_quartz_raw", "Raw Quartz Block", "ore_quartz_raw"),
    (
        "ore_quartz_sandstone_common",
        "Quartz Ore (Common Sandstone)",
        "rock_sandstone_common",
    ),
    (
        "ore_quartz_sandstone_red",
        "Quartz Ore (Red Sandstone)",
        "rock_sandstone_red",
    ),
    (
        "ore_quartz_sandstone_white",
        "Quartz Ore (White Sandstone)",
        "rock_sandstone_white",
    ),
    ("ore_saltpeter_karst", "Saltpeter Ore (Karst)", "rock_karst"),
    (
        "ore_saltpeter_limestone",
        "Saltpeter Ore (Limestone)",
        "rock_limestone",
    ),
    (
        "ore_saltpeter_raw",
        "Raw Saltpeter Block",
        "ore_saltpeter_raw",
    ),
    (
        "ore_saltpeter_sandstone_dune",
        "Saltpeter Ore (Dune Sandstone)",
        "rock_sandstone_dune",
    ),
    (
        "ore_saltpeter_sandstone_white",
        "Saltpeter Ore (White Sandstone)",
        "rock_sandstone_white",
    ),
    (
        "ore_silver_andesite",
        "Silver Ore (Andesite)",
        "rock_andesite",
    ),
    ("ore_silver_diorite", "Silver Ore (Diorite)", "rock_diorite"),
    ("ore_silver_gneiss", "Silver Ore (Gneiss)", "rock_gneiss"),
    ("ore_silver_marble", "Silver Ore (Marble)", "rock_marble"),
    ("ore_silver_raw", "Raw Silver Block", "ore_silver_raw"),
    ("ore_sulfur_basalt", "Sulfur Ore (Basalt)", "rock_basalt"),
    (
        "ore_sulfur_brimstone",
        "Sulfur Ore (Brimstone)",
        "rock_brimstone",
    ),
    ("ore_sulfur_raw", "Raw Sulfur Block", "ore_sulfur_raw"),
    ("ore_sulfur_scoria", "Sulfur Ore (Scoria)", "rock_scoria"),
    ("ore_sulfur_tuff", "Sulfur Ore (Tuff)", "rock_tuff"),
    ("ore_tin_andesite", "Tin Ore (Andesite)", "rock_andesite"),
    ("ore_tin_diorite", "Tin Ore (Diorite)", "rock_diorite"),
    ("ore_tin_dolomite", "Tin Ore (Dolomite)", "rock_dolomite"),
    ("ore_tin_gneiss", "Tin Ore (Gneiss)", "rock_gneiss"),
    ("ore_tin_granite", "Tin Ore (Granite)", "rock_granite"),
    ("ore_tin_raw", "Raw Tin Block", "ore_tin_raw"),
    (
        "ore_tin_sandstone_common",
        "Tin Ore (Common Sandstone)",
        "rock_sandstone_common",
    ),
    (
        "ore_uranium_diorite",
        "Uranium Ore (Diorite)",
        "rock_diorite",
    ),
    ("ore_uranium_gabbro", "Uranium Ore (Gabbro)", "rock_gabbro"),
    (
        "ore_uranium_granite",
        "Uranium Ore (Granite)",
        "rock_granite",
    ),
    ("ore_uranium_raw", "Raw Uranium Block", "ore_uranium_raw"),
    ("ore_uranium_slate", "Uranium Ore (Slate)", "rock_slate"),
    ("ore_zinc_andesite", "Zinc Ore (Andesite)", "rock_andesite"),
    (
        "ore_zinc_argillite",
        "Zinc Ore (Argillite)",
        "rock_argillite",
    ),
    ("ore_zinc_chalk", "Zinc Ore (Chalk)", "rock_chalk"),
    ("ore_zinc_diorite", "Zinc Ore (Diorite)", "rock_diorite"),
    ("ore_zinc_dolomite", "Zinc Ore (Dolomite)", "rock_dolomite"),
    ("ore_zinc_gneiss", "Zinc Ore (Gneiss)", "rock_gneiss"),
    ("ore_zinc_karst", "Zinc Ore (Karst)", "rock_karst"),
    (
        "ore_zinc_limestone",
        "Zinc Ore (Limestone)",
        "rock_limestone",
    ),
    ("ore_zinc_raw", "Raw Zinc Block", "ore_zinc_raw"),
    (
        "crystal_amethyst_block",
        "Amethyst Block",
        "crystal_amethyst",
    ),
    (
        "crystal_amethyst_cluster",
        "Amethyst Cluster",
        "crystal_amethyst_cluster",
    ),
    (
        "crystal_amethyst_glinted",
        "Glinted Amethyst Cluster",
        "crystal_amethyst_cluster_glinted",
    ),
    (
        "crystal_aquamarine_block",
        "Aquamarine Block",
        "crystal_aquamarine",
    ),
    (
        "crystal_aquamarine_cluster",
        "Aquamarine Cluster",
        "crystal_aquamarine_cluster",
    ),
    (
        "crystal_aquamarine_glinted",
        "Glinted Aquamarine Cluster",
        "crystal_aquamarine_cluster_glinted",
    ),
    ("crystal_citrine_block", "Citrine Block", "crystal_citrine"),
    (
        "crystal_citrine_cluster",
        "Citrine Cluster",
        "crystal_citrine_cluster",
    ),
    (
        "crystal_citrine_glinted",
        "Glinted Citrine Cluster",
        "crystal_citrine_cluster_glinted",
    ),
    (
        "crystal_fluorite_block",
        "Fluorite Block",
        "crystal_fluorite",
    ),
    (
        "crystal_fluorite_cluster",
        "Fluorite Cluster",
        "crystal_fluorite_cluster",
    ),
    (
        "crystal_fluorite_glinted",
        "Glinted Fluorite Cluster",
        "crystal_fluorite_cluster_glinted",
    ),
    (
        "crystal_pink_salt_block",
        "Pink Salt Block",
        "crystal_pink_salt",
    ),
    (
        "crystal_pink_salt_cluster",
        "Pink Salt Cluster",
        "crystal_pink_salt_cluster",
    ),
    (
        "crystal_pink_salt_glinted",
        "Glinted Pink Salt Cluster",
        "crystal_pink_salt_cluster_glinted",
    ),
    (
        "crystal_selenite_block",
        "Selenite Block",
        "crystal_selenite",
    ),
    (
        "crystal_selenite_cluster",
        "Selenite Cluster",
        "crystal_selenite_cluster",
    ),
    (
        "crystal_selenite_glinted",
        "Glinted Selenite Cluster",
        "crystal_selenite_cluster_glinted",
    ),
    ("dripstone_calcite", "Calcite Flowstone", "rock_calcite"),
    ("dripstone_karst", "Karst Dripstone", "rock_karst"),
    ("fossil_amber", "Fossil Amber", "fossil_amber"),
    ("fossil_bitumen", "Fossil Bitumen", "fossil_bitumen"),
    ("fossil_bones", "Fossil Bones", "fossil_bones"),
    ("fossil_matrix", "Fossil Matrix", "fossil_matrix"),
    ("speleothem_basalt", "Lava Icicle", "None"),
    ("speleothem_brimstone", "Brimstone Spike", "None"),
    ("speleothem_ice", "Glacial Icicle", "None"),
    ("speleothem_salt_straw", "Pink Salt Soda Straw", "None"),
    ("algae_mat", "Algae Mat", "aqua_algae_mat"),
    ("algae_reefstone", "Algae Reefstone", "aqua_reefstone"),
    (
        "barnacles_reefstone",
        "Barnacles Reefstone",
        "aqua_reefstone",
    ),
    ("bone_worms", "Bone Worms", "vege_bone_worms"),
    ("brain_coral", "Brain Coral", "aqua_brain_coral"),
    ("brain_coral_fan", "Brain Coral Fan", "vege_brain_coral_fan"),
    ("bubble_coral", "Bubble Coral", "aqua_bubble_coral"),
    (
        "bubble_coral_fan",
        "Bubble Coral Fan",
        "vege_bubble_coral_fan",
    ),
    ("bull_kelp", "Bull Kelp", "vege_bull_kelp"),
    ("coralstone", "Coralstone", "aqua_coralstone"),
    ("dead_coral", "Dead Coral Block", "aqua_dead_coral"),
    ("dead_coral_fan", "Dead Coral Fan", "vege_dead_coral_fan"),
    ("delesseria", "Delesseria", "vege_delesseria"),
    ("dying_coral", "Dying Coral Block", "aqua_dying_coral"),
    ("dying_coral_fan", "Dying Coral Fan", "vege_dying_coral_fan"),
    ("fire_coral", "Fire Coral", "aqua_fire_coral"),
    ("fire_coral_fan", "Fire Coral Fan", "vege_fire_coral_fan"),
    (
        "geothermal_vent",
        "Geothermal Vent",
        "aqua_geothermal_vent_side",
    ),
    ("heart_coral", "Heart Coral", "aqua_heart_coral"),
    ("heart_coral_fan", "Heart Coral Fan", "vege_heart_coral_fan"),
    ("horn_coral", "Horn Coral", "aqua_horn_coral"),
    ("horn_coral_fan", "Horn Coral Fan", "vege_horn_coral_fan"),
    ("kelp", "Giant Kelp", "vege_kelp"),
    ("leaf_coral", "Leaf Coral", "aqua_leaf_coral"),
    ("leaf_coral_fan", "Leaf Coral Fan", "vege_leaf_coral_fan"),
    ("pearl_block", "Pearl Block", "aqua_pearl_side"),
    ("red_algae", "Red Algae", "vege_red_algae"),
    ("reed_coral", "Reed Coral", "aqua_reed_coral"),
    ("reed_coral_fan", "Reed Coral Fan", "vege_reed_coral_fan"),
    ("reefstone", "Reefstone", "aqua_reefstone"),
    ("sargassum", "Sargassum", "vege_sargassum"),
    ("scute_block", "Turtle Scute Block", "aqua_scute_side"),
    ("sea_anemone", "Sea Anemone", "aqua_sea_anemone_side"),
    ("seagrass", "Seagrass", "vege_seagrass"),
    ("seaweed", "Seaweed", "vege_seaweed"),
    ("spike_coral", "Spike Coral", "aqua_spike_coral"),
    ("spike_coral_fan", "Spike Coral Fan", "vege_spike_coral_fan"),
    ("sponge", "Natural Sponge", "aqua_sponge"),
    ("staghorn_coral", "Staghorn Coral", "aqua_staghorn_coral"),
    (
        "staghorn_coral_fan",
        "Staghorn Coral Fan",
        "vege_staghorn_coral_fan",
    ),
    ("sun_coral", "Sun Coral", "aqua_sun_coral"),
    ("sun_coral_fan", "Sun Coral Fan", "vege_sun_coral_fan"),
    ("thorn_coral", "Thorn Coral", "aqua_thorn_coral"),
    ("thorn_coral_fan", "Thorn Coral Fan", "vege_thorn_coral_fan"),
    ("tube_coral", "Tube Coral", "aqua_tube_coral"),
    ("tube_coral_fan", "Tube Coral Fan", "vege_tube_coral_fan"),
    ("wax_coral", "Wax Coral", "aqua_wax_coral"),
    ("wax_coral_fan", "Wax Coral Fan", "vege_wax_coral_fan"),
    ("zigzag_coral", "Zigzag Coral", "aqua_zigzag_coral"),
    (
        "zigzag_coral_fan",
        "Zigzag Coral Fan",
        "vege_zigzag_coral_fan",
    ),
    ("ant_hill", "Ant Hill (Closed)", "nest_ant_hill"),
    ("ant_hill_open", "Ant Hill (Active)", "nest_ant_hill_open"),
    ("beaver_dam", "Beaver Dam", "nest_beaver_dam"),
    ("bee_nest", "Wild Bee Nest", "nest_bee_nest_side"),
    ("bird_nest", "Songbird Nest", "nest_bird_nest_side"),
    ("bloody_brain", "Bloody Brain Block", "bio_brain"),
    ("bloody_flesh", "Bloody Flesh Block", "bio_flesh_side"),
    ("blubber_block", "Blubber Block", "bio_blubber"),
    ("bone_block", "Bone Block", "bio_bone_side"),
    ("brain_block", "Brain Block", "bio_brain"),
    ("calcined_bone", "Calcined Bone", "bio_calcined_bone_side"),
    ("cobweb", "Cobweb Block", "organic_cobweb"),
    ("cobweb_arch", "Cobweb Arch", "organic_cobweb_arch"),
    ("cobweb_hanging", "Hanging Cobweb", "organic_cobweb_hanging"),
    (
        "cobweb_stretched",
        "Stretched Cobweb",
        "organic_cobweb_stretched",
    ),
    ("coprolite_block", "Coprolite Block", "organic_coprolite"),
    ("dust_block", "Organic Dust", "organic_dust"),
    ("flesh_block", "Raw Flesh Block", "bio_flesh_side"),
    ("fur_block", "Animal Fur Block", "bio_fur_side"),
    ("guano_block", "Guano Block", "organic_guano"),
    ("hay_nest", "Hay Poultry Nest", "nest_hay_nest_side"),
    ("honey_block", "Honey Block", "organic_honey_block_side"),
    (
        "honeycomb_block",
        "Honeycomb Block",
        "organic_honeycomb_block",
    ),
    ("jelly_block", "Royal Jelly Block", "organic_jelly_side"),
    ("loam_burrow", "Loam Animal Burrow", "soil_loam_dirt"),
    ("peat_burrow", "Peat Animal Burrow", "soil_peat_dirt"),
    ("poop_block", "Manure Block", "organic_poop"),
    ("seashell_block", "Seashell Block", "organic_seashell"),
    ("silk_cocoon", "Wild Silk Cocoon", "nest_silk_cocoon_side"),
    ("silt_burrow", "Silt Animal Burrow", "soil_silt_dirt"),
    ("slime_block", "Slime Block", "organic_slime_side"),
    ("spider_egg", "Spider Egg Sac", "nest_spider_egg_side"),
    (
        "termite_mound",
        "Termite Mound (Closed)",
        "nest_termite_mound",
    ),
    (
        "termite_mound_open",
        "Termite Mound (Active)",
        "nest_termite_mound_open",
    ),
    ("wasp_nest", "Paper Wasp Nest", "nest_wasp_nest_side"),
    ("wax_block", "Beeswax Block", "organic_wax"),
    ("bag", "Burlap Bag", "util_bag_side"),
    ("barrel", "Barrel", "util_barrel_side"),
    ("basket", "Basket", "util_basket_side"),
    ("bookshelf", "Bookshelf", "util_bookshelf_side"),
    ("bookshelf_empty", "Empty Bookshelf", "util_bookshelf_side"),
    ("brick_black", "Black Bricks", "deco_bricks_painted"),
    ("brick_blue", "Blue Bricks", "deco_bricks_painted"),
    ("brick_brown", "Brown Bricks", "deco_bricks_painted"),
    ("brick_dark_blue", "Dark Blue Bricks", "deco_bricks_painted"),
    ("brick_dark_grey", "Dark Grey Bricks", "deco_bricks_painted"),
    ("brick_green", "Green Bricks", "deco_bricks_painted"),
    (
        "brick_light_grey",
        "Light Grey Bricks",
        "deco_bricks_painted",
    ),
    (
        "brick_light_pink",
        "Light Pink Bricks",
        "deco_bricks_painted",
    ),
    ("brick_lime", "Lime Bricks", "deco_bricks_painted"),
    ("brick_orange", "Orange Bricks", "deco_bricks_painted"),
    ("brick_pink", "Pink Bricks", "deco_bricks_painted"),
    ("brick_purple", "Purple Bricks", "deco_bricks_painted"),
    ("brick_red", "Red Bricks", "deco_bricks_painted"),
    ("brick_turquoise", "Turquoise Bricks", "deco_bricks_painted"),
    ("brick_white", "White Bricks", "deco_bricks_painted"),
    ("brick_yellow", "Yellow Bricks", "deco_bricks_painted"),
    ("bricks", "Bricks", "deco_bricks"),
    ("corn_bale", "Corn Bale", "deco_corn_bale_side"),
    ("crate", "Crate", "util_crate_side"),
    ("emit_lamp", "Lamp", "emit_lamp"),
    ("emit_light_blue", "Blue Light", "emit_light_blue"),
    ("emit_light_cold", "Cold Light", "emit_light_cold"),
    ("emit_light_green", "Green Light", "emit_light_green"),
    ("emit_light_red", "Red Light", "emit_light_red"),
    ("emit_light_warm", "Warm Light", "emit_light_warm"),
    ("emit_torch_blue", "Blue Torch", "emit_torch_blue"),
    ("emit_torch_cold", "Cold Torch", "emit_torch_cold"),
    ("emit_torch_fire", "Fire Torch", "emit_torch_fire"),
    ("emit_torch_green", "Green Torch", "emit_torch_green"),
    ("emit_torch_red", "Red Torch", "emit_torch_red"),
    ("emit_torch_warm", "Warm Torch", "emit_torch_warm"),
    ("fabric", "Fabric", "deco_fabric"),
    ("fabric_black", "Black Fabric", "deco_fabric_painted"),
    ("fabric_blue", "Blue Fabric", "deco_fabric_painted"),
    ("fabric_brown", "Brown Fabric", "deco_fabric_painted"),
    (
        "fabric_dark_blue",
        "Dark Blue Fabric",
        "deco_fabric_painted",
    ),
    (
        "fabric_dark_grey",
        "Dark Grey Fabric",
        "deco_fabric_painted",
    ),
    ("fabric_green", "Green Fabric", "deco_fabric_painted"),
    (
        "fabric_light_grey",
        "Light Grey Fabric",
        "deco_fabric_painted",
    ),
    (
        "fabric_light_pink",
        "Light Pink Fabric",
        "deco_fabric_painted",
    ),
    ("fabric_lime", "Lime Fabric", "deco_fabric_painted"),
    ("fabric_orange", "Orange Fabric", "deco_fabric_painted"),
    ("fabric_pink", "Pink Fabric", "deco_fabric_painted"),
    ("fabric_purple", "Purple Fabric", "deco_fabric_painted"),
    ("fabric_red", "Red Fabric", "deco_fabric_painted"),
    (
        "fabric_turquoise",
        "Turquoise Fabric",
        "deco_fabric_painted",
    ),
    ("fabric_white", "White Fabric", "deco_fabric_painted"),
    ("fabric_yellow", "Yellow Fabric", "deco_fabric_painted"),
    ("glass", "Glass", "deco_glass"),
    ("glass_black", "Black Glass", "deco_glass_painted"),
    ("glass_blue", "Blue Glass", "deco_glass_painted"),
    ("glass_brown", "Brown Glass", "deco_glass_painted"),
    ("glass_dark_blue", "Dark Blue Glass", "deco_glass_painted"),
    ("glass_dark_grey", "Dark Grey Glass", "deco_glass_painted"),
    ("glass_green", "Green Glass", "deco_glass_painted"),
    ("glass_light_grey", "Light Grey Glass", "deco_glass_painted"),
    ("glass_light_pink", "Light Pink Glass", "deco_glass_painted"),
    ("glass_lime", "Lime Glass", "deco_glass_painted"),
    ("glass_orange", "Orange Glass", "deco_glass_painted"),
    ("glass_pink", "Pink Glass", "deco_glass_painted"),
    ("glass_purple", "Purple Glass", "deco_glass_painted"),
    ("glass_red", "Red Glass", "deco_glass_painted"),
    ("glass_turquoise", "Turquoise Glass", "deco_glass_painted"),
    ("glass_white", "White Glass", "deco_glass_painted"),
    ("glass_yellow", "Yellow Glass", "deco_glass_painted"),
    ("hay_bale", "Hay Bale", "deco_hay_bale_side"),
    ("linen", "Linen", "deco_linen"),
    ("linen_black", "Black Linen", "deco_linen_painted"),
    ("linen_blue", "Blue Linen", "deco_linen_painted"),
    ("linen_brown", "Brown Linen", "deco_linen_painted"),
    ("linen_dark_blue", "Dark Blue Linen", "deco_linen_painted"),
    ("linen_dark_grey", "Dark Grey Linen", "deco_linen_painted"),
    ("linen_green", "Green Linen", "deco_linen_painted"),
    ("linen_light_grey", "Light Grey Linen", "deco_linen_painted"),
    ("linen_light_pink", "Light Pink Linen", "deco_linen_painted"),
    ("linen_lime", "Lime Linen", "deco_linen_painted"),
    ("linen_orange", "Orange Linen", "deco_linen_painted"),
    ("linen_pink", "Pink Linen", "deco_linen_painted"),
    ("linen_purple", "Purple Linen", "deco_linen_painted"),
    ("linen_red", "Red Linen", "deco_linen_painted"),
    ("linen_turquoise", "Turquoise Linen", "deco_linen_painted"),
    ("linen_white", "White Linen", "deco_linen_painted"),
    ("linen_yellow", "Yellow Linen", "deco_linen_painted"),
    ("path_dirty", "Dirty Path", "deco_path_dirty"),
    ("path_sandy", "Sandy Path", "deco_path_sandy"),
    ("path_snowy", "Snowy Path", "deco_path_snowy"),
    ("path_stony", "Stony Path", "deco_path_stony"),
    ("plaster", "Plaster", "deco_plaster"),
    ("plaster_black", "Black Plaster", "deco_plaster_painted"),
    ("plaster_blue", "Blue Plaster", "deco_plaster_painted"),
    ("plaster_brown", "Brown Plaster", "deco_plaster_painted"),
    (
        "plaster_dark_blue",
        "Dark Blue Plaster",
        "deco_plaster_painted",
    ),
    (
        "plaster_dark_grey",
        "Dark Grey Plaster",
        "deco_plaster_painted",
    ),
    ("plaster_green", "Green Plaster", "deco_plaster_painted"),
    (
        "plaster_light_grey",
        "Light Grey Plaster",
        "deco_plaster_painted",
    ),
    (
        "plaster_light_pink",
        "Light Pink Plaster",
        "deco_plaster_painted",
    ),
    ("plaster_lime", "Lime Plaster", "deco_plaster_painted"),
    ("plaster_orange", "Orange Plaster", "deco_plaster_painted"),
    ("plaster_pink", "Pink Plaster", "deco_plaster_painted"),
    ("plaster_purple", "Purple Plaster", "deco_plaster_painted"),
    ("plaster_red", "Red Plaster", "deco_plaster_painted"),
    (
        "plaster_turquoise",
        "Turquoise Plaster",
        "deco_plaster_painted",
    ),
    ("plaster_white", "White Plaster", "deco_plaster_painted"),
    ("plaster_yellow", "Yellow Plaster", "deco_plaster_painted"),
    ("rice_bale", "Rice Bale", "deco_rice_bale_side"),
    ("shingles", "Shingles", "deco_shingles"),
    ("shingles_black", "Black Shingles", "deco_shingles_painted"),
    ("shingles_blue", "Blue Shingles", "deco_shingles_painted"),
    ("shingles_brown", "Brown Shingles", "deco_shingles_painted"),
    (
        "shingles_dark_blue",
        "Dark Blue Shingles",
        "deco_shingles_painted",
    ),
    (
        "shingles_dark_grey",
        "Dark Grey Shingles",
        "deco_shingles_painted",
    ),
    ("shingles_green", "Green Shingles", "deco_shingles_painted"),
    (
        "shingles_light_grey",
        "Light Grey Shingles",
        "deco_shingles_painted",
    ),
    (
        "shingles_light_pink",
        "Light Pink Shingles",
        "deco_shingles_painted",
    ),
    ("shingles_lime", "Lime Shingles", "deco_shingles_painted"),
    (
        "shingles_orange",
        "Orange Shingles",
        "deco_shingles_painted",
    ),
    ("shingles_pink", "Pink Shingles", "deco_shingles_painted"),
    (
        "shingles_purple",
        "Purple Shingles",
        "deco_shingles_painted",
    ),
    ("shingles_red", "Red Shingles", "deco_shingles_painted"),
    (
        "shingles_turquoise",
        "Turquoise Shingles",
        "deco_shingles_painted",
    ),
    ("shingles_white", "White Shingles", "deco_shingles_painted"),
    (
        "shingles_yellow",
        "Yellow Shingles",
        "deco_shingles_painted",
    ),
    ("straw_bale", "Straw Bale", "deco_straw_bale_side"),
    ("tinted_glass", "Tinted Glass", "deco_tinted_glass"),
    (
        "tinted_glass_black",
        "Black Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_blue",
        "Blue Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_brown",
        "Brown Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_dark_blue",
        "Dark Blue Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_dark_grey",
        "Dark Grey Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_green",
        "Green Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_light_grey",
        "Light Grey Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_light_pink",
        "Light Pink Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_lime",
        "Lime Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_orange",
        "Orange Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_pink",
        "Pink Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_purple",
        "Purple Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_red",
        "Red Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_turquoise",
        "Turquoise Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_white",
        "White Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    (
        "tinted_glass_yellow",
        "Yellow Tinted Glass",
        "deco_tinted_glass_painted",
    ),
    ("wool", "Wool", "deco_wool"),
    ("wool_black", "Black Wool", "deco_wool_painted"),
    ("wool_blue", "Blue Wool", "deco_wool_painted"),
    ("wool_brown", "Brown Wool", "deco_wool_painted"),
    ("wool_dark_blue", "Dark Blue Wool", "deco_wool_painted"),
    ("wool_dark_grey", "Dark Grey Wool", "deco_wool_painted"),
    ("wool_green", "Green Wool", "deco_wool_painted"),
    ("wool_light_grey", "Light Grey Wool", "deco_wool_painted"),
    ("wool_light_pink", "Light Pink Wool", "deco_wool_painted"),
    ("wool_lime", "Lime Wool", "deco_wool_painted"),
    ("wool_orange", "Orange Wool", "deco_wool_painted"),
    ("wool_pink", "Pink Wool", "deco_wool_painted"),
    ("wool_purple", "Purple Wool", "deco_wool_painted"),
    ("wool_red", "Red Wool", "deco_wool_painted"),
    ("wool_turquoise", "Turquoise Wool", "deco_wool_painted"),
    ("wool_white", "White Wool", "deco_wool_painted"),
    ("wool_yellow", "Yellow Wool", "deco_wool_painted"),
    ("fungi_amanita", "Amanita (Fly Agaric)", "fungi_amanita"),
    ("fungi_amanita_cap", "Amanita Cap", "fungi_amanita_cap"),
    (
        "fungi_chanterelle",
        "Golden Chanterelle",
        "fungi_chanterelle",
    ),
    (
        "fungi_chanterelle_cap",
        "Chanterelle Cap",
        "fungi_chanterelle_cap",
    ),
    ("fungi_indigo", "Indigo Milk Cap", "fungi_indigo"),
    ("fungi_indigo_cap", "Indigo Cap", "fungi_indigo_cap"),
    ("fungi_inside", "Fungi Inside Flesh", "fungi_inside"),
    ("fungi_mycena", "Mycena (Bioluminescent)", "fungi_mycena"),
    ("fungi_mycena_cap", "Mycena Cap", "fungi_mycena_cap"),
    ("fungi_porcini", "Porcini (King Bolete)", "fungi_porcini"),
    ("fungi_porcini_cap", "Porcini Cap", "fungi_porcini_cap"),
    ("fungi_russula", "Russula (Sickener)", "fungi_russula"),
    ("fungi_russula_cap", "Russula Cap", "fungi_russula_cap"),
    ("fungi_shiitake", "Shiitake", "fungi_shiitake"),
    ("fungi_shiitake_cap", "Shiitake Cap", "fungi_shiitake_cap"),
    ("fungi_stem", "Fungi Stem", "fungi_stem"),
    ("fungi_verdigris", "Verdigris Agaric", "fungi_verdigris"),
    (
        "fungi_verdigris_cap",
        "Verdigris Cap",
        "fungi_verdigris_cap",
    ),
];

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
            Self::GneissGrass => Some("rock_gneiss_grass_side"),
            Self::GneissSnow => Some("rock_gneiss_snow_side"),
            Self::GraniteGrass => Some("rock_granite_grass_side"),
            Self::GraniteSnow => Some("rock_granite_snow_side"),
            Self::LimestoneGrass => Some("rock_limestone_grass_side"),
            Self::LimestoneSnow => Some("rock_limestone_snow_side"),
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
            | Self::GneissGrass
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
            | Self::GneissSnow
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
            Self::LoamBurrow | Self::PeatBurrow | Self::SiltBurrow => Some("nest_burrow_top"),
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
            Self::GneissGrass | Self::GneissSnow => Some("rock_gneiss"),
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
        let id = *self as u16;
        match id {
            34
            | 45
            | 49
            | 61
            | 69
            | 80
            | 88
            | 91
            | 94
            | 99
            | 129
            | 133
            | 136
            | 140
            | 531..=532
            | 536 => Some("soil_grass_side_overlay"),
            146..=148 | 157..=161 => Some("mossy_rock_overlay"),
            149..=156 => Some("mossy_cobbled_overlay"),
            271 | 273 => Some("gem_diamond_overlay"),
            274 | 276 => Some("gem_emerald_overlay"),
            277 | 279 => Some("gem_jade_overlay"),
            280..=281 | 283..=284 => Some("gem_onyx_overlay"),
            285 | 287..=289 => Some("gem_opal_overlay"),
            290..=291 => Some("gem_ruby_overlay"),
            293..=294 => Some("gem_sapphire_overlay"),
            296 | 298 => Some("gem_topaz_overlay"),
            299..=301 | 303 => Some("ore_aluminum_overlay"),
            304..=306 | 308 => Some("ore_cinnabar_overlay"),
            309..=316 | 318..=321 => Some("ore_coal_overlay"),
            322..=330 | 332..=336 => Some("ore_copper_overlay"),
            337..=338 | 340..=341 => Some("ore_gold_overlay"),
            342..=348 | 350..=354 => Some("ore_iron_overlay"),
            355..=357 | 359 => Some("ore_lazurite_overlay"),
            360..=365 | 367..=368 => Some("ore_lead_overlay"),
            369..=373 | 375 => Some("ore_manganese_overlay"),
            376..=377 | 379..=380 => Some("ore_nickel_overlay"),
            381..=382 | 384..=385 => Some("ore_platinum_overlay"),
            386..=388 | 390..=392 => Some("ore_quartz_overlay"),
            393..=394 | 396..=397 => Some("ore_saltpeter_overlay"),
            398..=401 => Some("ore_silver_overlay"),
            403..=404 | 406..=407 => Some("ore_sulfur_overlay"),
            408..=412 | 414 => Some("ore_tin_overlay"),
            415..=417 | 419 => Some("ore_uranium_overlay"),
            420..=427 => Some("ore_zinc_overlay"),
            458 => Some("algae_dead_overlay"),
            459 => Some("barnacles_overlay"),
            512..=513 => Some("bloody_overlay"),
            _ => None,
        }
    }

    pub fn default_shape(self) -> BlockShape {
        let id = self as u16;
        if self.is_cross_billboard_const() {
            BlockShape::CrossBillboard
        } else if self.is_torch_const() {
            BlockShape::Torch
        } else if self.is_basket_const() {
            BlockShape::Basket
        } else if matches!(id, 163 | 166) {
            BlockShape::PebbleOne
        } else if matches!(id, 165 | 168) {
            BlockShape::PebbleTwo
        } else if matches!(id, 164 | 167) {
            BlockShape::PebbleThree
        } else if matches!(
            id,
            181 | 186
                | 192
                | 195
                | 200
                | 205
                | 210
                | 215
                | 220
                | 225
                | 230
                | 238
                | 243
                | 248
                | 253
                | 258
                | 263
                | 268
                | 465
                | 469
                | 479
        ) {
            BlockShape::Column
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
                | Self::GneissGrass
                | Self::GraniteGrass
                | Self::LimestoneGrass
                | Self::LoamBurrow
                | Self::PeatBurrow
                | Self::SiltBurrow
        )
    }

    pub fn is_tinted(self) -> bool {
        let id = self as u16;
        matches!(
            id,
            2 | 28
                | 34
                | 45
                | 49
                | 61
                | 69
                | 80
                | 88
                | 91
                | 94
                | 99
                | 129
                | 133
                | 136
                | 140
                | 179
                | 184
                | 189
                | 191
                | 193
                | 198
                | 203
                | 208
                | 213
                | 218
                | 223
                | 228
                | 241
                | 246
                | 251
                | 261
                | 266
                | 531..=532 | 536
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
                | Self::BambooStalk
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
        {
            return NaturalRotationMode::None;
        }

        let id = self as u16;

        // Trees & Wood (logs, planks, wood, bamboo): 179..=270
        if matches!(id, 179..=270) {
            return NaturalRotationMode::None;
        }

        // Architecture, furniture, utilities, lighting, fibers: 543..=720
        if matches!(id, 543..=720) {
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
                    | Self::GneissSnow
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

        // Natural isotropic: Soils, sands, gravels, clays, ashes (29..=86)
        if matches!(id, 29..=86) {
            return NaturalRotationMode::Isotropic;
        }

        // Geology & Rocks (87..=178)
        if matches!(id, 87..=178) {
            return NaturalRotationMode::Isotropic;
        }

        // Ores, Minerals & Gemology (271..=428)
        if matches!(id, 271..=428) {
            return NaturalRotationMode::Isotropic;
        }

        // Crystals, Speleothems, Volcanism, Fossils (429..=530)
        if matches!(id, 429..=530) {
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
            548..=563 => id - 548, // Brick
            580..=595 => id - 580, // Fabric
            597..=612 => id - 597, // Glass
            615..=630 => id - 615, // Linen
            636..=651 => id - 636, // Plaster
            654..=669 => id - 654, // Shingles
            672..=687 => id - 672, // TintedGlass
            689..=704 => id - 689, // Wool
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
            s if s.is_tinted() => [0.55, 0.94, 0.42, 1.0],
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
            | Self::GneissSnow
            | Self::GraniteSnow
            | Self::LimestoneSnow => [245, 248, 252, 255],
            Self::PermafrostIcy => [210, 225, 240, 255],
            Self::FrostBlueIce | Self::FrostBlueIceCracked => [120, 180, 240, 255],
            Self::FrostIce | Self::FrostIceFragile | Self::FrostIcePacked => [180, 215, 245, 255],
            Self::FrostIceBlack | Self::FrostIceBlackFragile | Self::FrostIceBlackPacked => [60, 70, 85, 255],

            // Sand variants
            Self::SandCommon => [222, 202, 148, 255],
            Self::SandDune => [228, 188, 120, 255],
            Self::SandRed => [194, 102, 62, 255],
            Self::SandWhite => [242, 238, 222, 255],
            Self::SandPink => [228, 180, 185, 255],
            Self::SandBlack => [46, 44, 52, 255],
            Self::GravelSandy | Self::PathSandy => [204, 184, 138, 255],

            // Sandstones
            Self::SandstoneCommon | Self::CobbledSandstoneCommon => [212, 190, 138, 255],
            Self::SandstoneDune | Self::CobbledSandstoneDune => [218, 178, 112, 255],
            Self::SandstoneRed | Self::CobbledSandstoneRed => [184, 92, 54, 255],
            Self::SandstoneWhite | Self::CobbledSandstoneWhite => [232, 228, 212, 255],
            Self::SandstonePink | Self::CobbledSandstonePink => [218, 170, 175, 255],
            Self::SandstoneBlack | Self::CobbledSandstoneBlack => [42, 40, 48, 255],

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
            Self::LoamDirt | Self::LoamCoarse | Self::LoamRooted | Self::LoamBurrow => [110, 80, 55, 255],
            Self::LoamTilled => [95, 68, 45, 255],
            Self::PeatDirt | Self::PeatCoarse | Self::PeatRooted | Self::PeatBurrow => [70, 52, 40, 255],
            Self::PeatTilled => [60, 45, 35, 255],
            Self::SiltDirt | Self::SiltCoarse | Self::SiltRooted | Self::SiltBurrow => [135, 110, 80, 255],
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
            | Self::Magma
            | Self::MantlePlume => Color::srgb(1.0, 0.45, 0.15),
            Self::GeothermalVent => Color::srgb(1.0, 0.55, 0.20),
            Self::FungiMycena | Self::FungiMycenaCap => Color::srgb(0.20, 0.90, 0.70),
            Self::CrystalCitrineGlinted => Color::srgb(1.0, 0.85, 0.30),
            Self::CrystalFluoriteGlinted => Color::srgb(0.70, 0.40, 0.95),
            Self::CrystalPinkSaltGlinted => Color::srgb(1.0, 0.60, 0.75),
            Self::CrystalSeleniteGlinted => Color::srgb(0.85, 0.92, 1.0),
            _ => Color::WHITE,
        }
    }

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
    pub fn has_custom_mesh(self) -> bool {
        (VOXEL_PROPS[self as usize].flags & (FLAG_TORCH | FLAG_BASKET | FLAG_CROSS_BILLBOARD)) != 0
            || self.is_pebble_const()
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
        matches!(self, Self::Mantle | Self::MantlePlume)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_rotation_modes() {
        // Natural isotropic blocks: rock, sand, gravel, dirt
        assert_eq!(
            Voxel::Granite.natural_rotation_mode(),
            NaturalRotationMode::Isotropic
        );
        assert_eq!(
            Voxel::Basalt.natural_rotation_mode(),
            NaturalRotationMode::Isotropic
        );
        assert_eq!(
            Voxel::Andesite.natural_rotation_mode(),
            NaturalRotationMode::Isotropic
        );
        assert_eq!(
            Voxel::SandCommon.natural_rotation_mode(),
            NaturalRotationMode::Isotropic
        );
        assert_eq!(
            Voxel::LoamDirt.natural_rotation_mode(),
            NaturalRotationMode::Isotropic
        );
        assert_eq!(
            Voxel::GravelSandy.natural_rotation_mode(),
            NaturalRotationMode::Isotropic
        );

        // Natural soils with side overlays: top rotated 4-way, side flipped horizontally only
        assert_eq!(
            Voxel::LoamGrass.natural_rotation_mode(),
            NaturalRotationMode::SoilSide
        );
        assert_eq!(
            Voxel::PeatGrass.natural_rotation_mode(),
            NaturalRotationMode::SoilSide
        );
        assert_eq!(
            Voxel::BasaltSnow.natural_rotation_mode(),
            NaturalRotationMode::SoilSide
        );
        assert_eq!(
            Voxel::LoamSnow.natural_rotation_mode(),
            NaturalRotationMode::SoilSide
        );

        // Directional / architectural blocks: must NOT rotate
        assert_eq!(
            Voxel::OakLog.natural_rotation_mode(),
            NaturalRotationMode::None
        );
        assert_eq!(
            Voxel::BirchPlanks.natural_rotation_mode(),
            NaturalRotationMode::None
        );
        assert_eq!(
            Voxel::Bricks.natural_rotation_mode(),
            NaturalRotationMode::None
        );
        assert_eq!(
            Voxel::BrickRed.natural_rotation_mode(),
            NaturalRotationMode::None
        );
        assert_eq!(
            Voxel::Barrel.natural_rotation_mode(),
            NaturalRotationMode::None
        );
        assert_eq!(
            Voxel::EmitTorchFire.natural_rotation_mode(),
            NaturalRotationMode::None
        );
    }

    #[test]
    fn test_axis_aligned_pillar_properties() {
        // Normal to axis conversion
        assert_eq!(PillarAxis::from_normal(IVec3::new(1, 0, 0)), PillarAxis::X);
        assert_eq!(PillarAxis::from_normal(IVec3::new(-1, 0, 0)), PillarAxis::X);
        assert_eq!(PillarAxis::from_normal(IVec3::new(0, 0, 1)), PillarAxis::Z);
        assert_eq!(PillarAxis::from_normal(IVec3::new(0, 0, -1)), PillarAxis::Z);
        assert_eq!(PillarAxis::from_normal(IVec3::new(0, 1, 0)), PillarAxis::Y);
        assert_eq!(PillarAxis::from_normal(IVec3::new(0, -1, 0)), PillarAxis::Y);

        // Axis-aligned pillar detection
        assert!(Voxel::OakLog.is_axis_aligned());
        assert!(Voxel::BaobabLog.is_axis_aligned());
        assert!(Voxel::BoneBlock.is_axis_aligned());
        assert!(Voxel::FleshBlock.is_axis_aligned());
        assert!(Voxel::HayBale.is_axis_aligned());
        assert!(Voxel::BambooStalk.is_axis_aligned());

        // Non-pillar blocks
        assert!(!Voxel::OakPlanks.is_axis_aligned());
        assert!(!Voxel::Granite.is_axis_aligned());
        assert!(!Voxel::LoamDirt.is_axis_aligned());

        // BlockShape::Full has 3 orientations (Y, X, Z)
        assert_eq!(BlockShape::Full.orientation_count(), 3);
        assert_eq!(BlockShape::Full.orientation_name(0), "Vertical (Y)");
        assert_eq!(BlockShape::Full.orientation_name(1), "Horizontal (X)");
        assert_eq!(BlockShape::Full.orientation_name(2), "Horizontal (Z)");
    }

    #[test]
    fn test_voxel_discriminants_continuity() {
        for id in 0..=722u16 {
            let maybe_voxel = Voxel::from_u16(id);
            assert!(maybe_voxel.is_some(), "Voxel discriminant {id} must be valid");
            assert_eq!(maybe_voxel.unwrap() as u16, id);
        }
        assert_eq!(Voxel::from_u16(723), None);
        assert_eq!(Voxel::from_u16(1000), None);

        assert_eq!(Voxel::ALL.len(), 720);
        assert_eq!(Voxel::ALL[0], Voxel::DebugAccept);
        assert_eq!(Voxel::ALL[719], Voxel::FungiVerdigrisCap);
    }

    #[test]
    fn test_block_shapes_all() {
        assert_eq!(BlockShape::all().len(), 10);
        assert_eq!(BlockShape::shapable().len(), 4);
    }
}
