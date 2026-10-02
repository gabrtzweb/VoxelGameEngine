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
}

impl BlockShape {
    pub const fn all() -> [BlockShape; 4] {
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
        }
    }

    pub const fn orientation_count(self) -> u8 {
        match self {
            Self::Full | Self::Basket => 1,
            Self::Slab => 6,
            Self::Stair => 8,
            Self::Column => 6,
            Self::Torch => 5,
        }
    }

    pub fn orientation_name(self, orientation: u8) -> &'static str {
        match self {
            Self::Full | Self::Basket => "Standard",
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
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::Slab,
            Self::Slab => Self::Stair,
            Self::Stair => Self::Column,
            Self::Column => Self::Full,
            other => other,
        }
    }

    /// Returns the local AABB bounding boxes ([min, max] in 0.0..=1.0 coordinates) for this shape and orientation.
    /// Full, Slab, and Column have 1 box; Stair has 2 boxes (base slab + step).
    pub fn local_boxes(self, orientation: u8) -> ([Vec3; 2], Option<[Vec3; 2]>) {
        match self {
            Self::Full | Self::Basket => ([Vec3::ZERO, Vec3::ONE], None),
            Self::Slab => {
                let bbox = match orientation % 6 {
                    0 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.5, 1.0)], // Floor
                    1 => [Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0)], // Ceiling
                    2 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.5)], // North (-Z)
                    3 => [Vec3::new(0.0, 0.0, 0.5), Vec3::new(1.0, 1.0, 1.0)], // South (+Z)
                    4 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 1.0)], // West (-X)
                    _ => [Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)], // East (+X)
                };
                (bbox, None)
            }
            Self::Column => {
                let bbox = match orientation % 6 {
                    0 => [Vec3::new(0.25, 0.0, 0.25), Vec3::new(0.75, 1.0, 0.75)], // Centered Vert
                    1 => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5)],     // MinX, MinZ
                    2 => [Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.5)],     // MaxX, MinZ
                    3 => [Vec3::new(0.0, 0.0, 0.5), Vec3::new(0.5, 1.0, 1.0)],     // MinX, MaxZ
                    4 => [Vec3::new(0.5, 0.0, 0.5), Vec3::new(1.0, 1.0, 1.0)],     // MaxX, MaxZ
                    _ => [Vec3::new(0.0, 0.25, 0.25), Vec3::new(1.0, 0.75, 0.75)], // Centered Horiz
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
                    (false, 0) => [Vec3::new(0.5, 0.5, 0.0), Vec3::new(1.0, 1.0, 1.0)], // +X
                    (false, 1) => [Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.5, 1.0, 1.0)], // -X
                    (false, 2) => [Vec3::new(0.0, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0)], // +Z
                    (false, 3) => [Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.0, 1.0, 0.5)], // -Z

                    (true, 0) => [Vec3::new(0.5, 0.0, 0.0), Vec3::new(1.0, 0.5, 1.0)], // +X
                    (true, 1) => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 1.0)], // -X
                    (true, 2) => [Vec3::new(0.0, 0.0, 0.5), Vec3::new(1.0, 0.5, 1.0)], // +Z
                    (true, 3) => [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.5, 0.5)], // -Z
                    _ => unreachable!(),
                };

                (base, Some(step))
            }
            Self::Torch => {
                let bbox = match orientation % 5 {
                    0 => [Vec3::new(0.40, 0.0, 0.40), Vec3::new(0.60, 0.65, 0.60)], // Floor
                    1 => [Vec3::new(0.40, 0.12, 0.0), Vec3::new(0.60, 0.78, 0.45)], // Wall North (-Z)
                    2 => [Vec3::new(0.40, 0.12, 0.55), Vec3::new(0.60, 0.78, 1.0)], // Wall South (+Z)
                    3 => [Vec3::new(0.0, 0.12, 0.40), Vec3::new(0.45, 0.78, 0.60)], // Wall West (-X)
                    _ => [Vec3::new(0.55, 0.12, 0.40), Vec3::new(1.0, 0.78, 0.60)], // Wall East (+X)
                };
                (bbox, None)
            }
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[allow(non_camel_case_types)]
pub enum Voxel {
    #[default]
    Air = 0,
    Occupied = 1,
    WaterOccupied = 2,

    // Testing / Debug Blocks (8)
    Null_Block = 3,
    Null_Liquid = 4,
    Test_Accept = 5,
    Test_Debug = 6,
    Test_Numbers = 7,
    Test_Fail = 8,
    Test_Instance = 9,
    Test_Start = 10,

    // Liquid / Fluid Blocks (8)
    Liquid_Acid = 11,
    Liquid_Blood = 12,
    Liquid_Lava = 13,
    Liquid_Molten = 14,
    Liquid_Ooze = 15,
    Liquid_Sludge = 16,
    Liquid_Tar = 17,
    Liquid_Water = 18,

    // Frost Blocks (4)
    Frost_Black_Ice = 19,
    Frost_Fragile_Ice = 20,
    Frost_Ice = 21,
    Frost_Packed_Ice = 22,

    // Soil Blocks (32)
    Soil_Ash = 23,
    Soil_Black_Sand = 24,
    Soil_Clay = 25,
    Soil_Dirt = 26,
    Soil_Grass = 27,
    Soil_Gravel = 28,
    Soil_Moss = 29,
    Soil_Mud = 30,
    Soil_Mulch = 31,
    Soil_Packed_Dirt = 32,
    Soil_Packed_Mud = 33,
    Soil_Packed_Peat = 34,
    Soil_Packed_Silt = 35,
    Soil_Peat = 36,
    Soil_Peat_Grass = 37,
    Soil_Peat_Mulch = 38,
    Soil_Red_Moss = 39,
    Soil_Red_Sand = 40,
    Soil_Rooted_Dirt = 41,
    Soil_Sand = 42,
    Soil_Scorched_Black_Sand = 43,
    Soil_Scorched_Red_Sand = 44,
    Soil_Scorched_Sand = 45,
    Soil_Scorched_White_Sand = 46,
    Soil_Silt = 47,
    Soil_Silt_Grass = 48,
    Soil_Silt_Mulch = 49,
    Soil_Snow = 50,
    Soil_Snowy_Grass = 51,
    Soil_Snowy_Peat = 52,
    Soil_Snowy_Silt = 53,
    Soil_White_Sand = 54,

    // Rocky Blocks (96)
    Rock_Andesite = 55,
    Cobbled_Andesite = 56,
    Mossy_Andesite = 57,
    Mossy_Cobbled_Andesite = 58,
    Rock_Azurite = 59,
    Cobbled_Azurite = 60,
    Mossy_Azurite = 61,
    Mossy_Cobbled_Azurite = 62,
    Rock_Basalt = 63,
    Cobbled_Basalt = 64,
    Mossy_Basalt = 65,
    Mossy_Cobbled_Basalt = 66,
    Rock_Black_Sandstone = 67,
    Cobbled_Black_Sandstone = 68,
    Mossy_Black_Sandstone = 69,
    Mossy_Cobbled_Black_Sandstone = 70,
    Rock_Calcite = 71,
    Cobbled_Calcite = 72,
    Mossy_Calcite = 73,
    Mossy_Cobbled_Calcite = 74,
    Rock_Chalk = 75,
    Cobbled_Chalk = 76,
    Mossy_Chalk = 77,
    Mossy_Cobbled_Chalk = 78,
    Rock_Chert = 79,
    Cobbled_Chert = 80,
    Mossy_Chert = 81,
    Mossy_Cobbled_Chert = 82,
    Rock_Cinnabar = 83,
    Cobbled_Cinnabar = 84,
    Mossy_Cinnabar = 85,
    Mossy_Cobbled_Cinnabar = 86,
    Rock_Diorite = 87,
    Cobbled_Diorite = 88,
    Mossy_Diorite = 89,
    Mossy_Cobbled_Diorite = 90,
    Rock_Gabbro = 91,
    Cobbled_Gabbro = 92,
    Mossy_Gabbro = 93,
    Mossy_Cobbled_Gabbro = 94,
    Rock_Granite = 95,
    Cobbled_Granite = 96,
    Mossy_Granite = 97,
    Mossy_Cobbled_Granite = 98,
    Rock_Karst = 99,
    Cobbled_Karst = 100,
    Mossy_Karst = 101,
    Mossy_Cobbled_Karst = 102,
    Rock_Limestone = 103,
    Cobbled_Limestone = 104,
    Mossy_Limestone = 105,
    Mossy_Cobbled_Limestone = 106,
    Rock_Marl = 107,
    Cobbled_Marl = 108,
    Mossy_Marl = 109,
    Mossy_Cobbled_Marl = 110,
    Rock_Pitchstone = 111,
    Cobbled_Pitchstone = 112,
    Mossy_Pitchstone = 113,
    Mossy_Cobbled_Pitchstone = 114,
    Rock_Porphyry = 115,
    Cobbled_Porphyry = 116,
    Mossy_Porphyry = 117,
    Mossy_Cobbled_Porphyry = 118,
    Rock_Red_Sandstone = 119,
    Cobbled_Red_Sandstone = 120,
    Mossy_Red_Sandstone = 121,
    Mossy_Cobbled_Red_Sandstone = 122,
    Rock_Sandstone = 123,
    Cobbled_Sandstone = 124,
    Mossy_Sandstone = 125,
    Mossy_Cobbled_Sandstone = 126,
    Rock_Scoria = 127,
    Cobbled_Scoria = 128,
    Mossy_Scoria = 129,
    Mossy_Cobbled_Scoria = 130,
    Rock_Serpentine = 131,
    Cobbled_Serpentine = 132,
    Mossy_Serpentine = 133,
    Mossy_Cobbled_Serpentine = 134,
    Rock_Slate = 135,
    Cobbled_Slate = 136,
    Mossy_Slate = 137,
    Mossy_Cobbled_Slate = 138,
    Rock_Stone = 139,
    Cobbled_Stone = 140,
    Mossy_Stone = 141,
    Mossy_Cobbled_Stone = 142,
    Rock_Tuffite = 143,
    Cobbled_Tuffite = 144,
    Mossy_Tuffite = 145,
    Mossy_Cobbled_Tuffite = 146,
    Rock_White_Sandstone = 147,
    Cobbled_White_Sandstone = 148,
    Mossy_White_Sandstone = 149,
    Mossy_Cobbled_White_Sandstone = 150,

    // Special Rocky Blocks (8)
    Rock_Alabaster = 151,
    Rock_Brimstone = 152,
    Rock_Cryolite = 153,
    Rock_Dreadstone = 154,
    Rock_Flint = 155,
    Rock_Magma = 156,
    Rock_Obsidian = 157,
    Rock_Terracotta = 158,

    // Wood Blocks (56)
    Tree_Acacia_Bark = 159,
    Tree_Acacia_Log = 160,
    Tree_Acacia_Leaves = 161,
    Tree_Acacia_Planks = 162,
    Tree_Birch_Bark = 163,
    Tree_Birch_Log = 164,
    Tree_Birch_Leaves = 165,
    Tree_Birch_Planks = 166,
    Tree_Cherry_Bark = 167,
    Tree_Cherry_Log = 168,
    Tree_Cherry_Leaves = 169,
    Tree_Cherry_Planks = 170,
    Tree_Mahogany_Bark = 171,
    Tree_Mahogany_Log = 172,
    Tree_Mahogany_Leaves = 173,
    Tree_Mahogany_Planks = 174,
    Tree_Mangrove_Bark = 175,
    Tree_Mangrove_Log = 176,
    Tree_Mangrove_Leaves = 177,
    Tree_Mangrove_Planks = 178,
    Tree_Mangrove_Roots = 179,
    Tree_Maple_Bark = 180,
    Tree_Maple_Log = 181,
    Tree_Maple_Leaves_Red = 182,
    Tree_Maple_Leaves_Orange = 183,
    Tree_Maple_Leaves_Yellow = 184,
    Tree_Maple_Planks = 185,
    Tree_Oak_Bark = 186,
    Tree_Oak_Log = 187,
    Tree_Oak_Leaves = 188,
    Tree_Oak_Leaves_Lush = 189,
    Tree_Oak_Leaves_Flowering = 190,
    Tree_Oak_Planks = 191,
    Tree_Palm_Bark = 192,
    Tree_Palm_Log = 193,
    Tree_Palm_Leaves = 194,
    Tree_Palm_Planks = 195,
    Tree_Pine_Bark = 196,
    Tree_Pine_Log = 197,
    Tree_Pine_Leaves = 198,
    Tree_Pine_Planks = 199,
    Tree_Willow_Bark = 200,
    Tree_Willow_Log = 201,
    Tree_Willow_Leaves = 202,
    Tree_Willow_Planks = 203,
    Tree_Yew_Bark = 204,
    Tree_Yew_Log = 205,
    Tree_Yew_Leaves = 206,
    Tree_Yew_Planks = 207,
    Tree_Cactus = 208,
    Tree_Charred_Bark = 209,
    Tree_Charred_Log = 210,
    Tree_Charred_Planks = 211,
    Tree_Dead_Bark = 212,
    Tree_Dead_Log = 213,
    Tree_Dead_Planks = 214,

    // Aquatic Blocks (8)
    Aqua_Algae_Mat = 215,
    Aqua_Brain_Coral = 216,
    Aqua_Bubble_Coral = 217,
    Aqua_Fire_Coral = 218,
    Aqua_Geothermal_Vent = 219,
    Aqua_Horn_Coral = 220,
    Aqua_Sponge = 221,
    Aqua_Tube_Coral = 222,

    // Light Source Blocks (8)
    Emit_Blue_Light = 223,
    Emit_Blue_Torch = 224,
    Emit_Cold_Light = 225,
    Emit_Green_Light = 226,
    Emit_Green_Torch = 227,
    Emit_Red_Light = 228,
    Emit_Red_Torch = 229,
    Emit_Warm_Light = 230,

    // Decoration Blocks (16)
    Deco_Barrel = 231,
    Deco_Basket = 232,
    Deco_Bone = 233,
    Deco_Bookshelf = 234,
    Deco_Brick = 235,
    Deco_Fabric = 236,
    Deco_Flesh = 237,
    Deco_Glass = 238,
    Deco_Hay = 239,
    Deco_Plaster = 240,
    Deco_Slime = 241,
    Deco_Stone_Path = 242,
    Deco_Thatch = 243,
    Deco_Wax = 244,
    Deco_Wicker = 245,
    Deco_Wool = 246,

}

impl Voxel {
    /// All voxels that map to a texture and are loaded into the terrain texture array.
    pub const ALL: [Voxel; 244] = [
        // Testing / Debug Blocks
        Voxel::Null_Block,
        Voxel::Null_Liquid,
        Voxel::Test_Accept,
        Voxel::Test_Debug,
        Voxel::Test_Numbers,
        Voxel::Test_Fail,
        Voxel::Test_Instance,
        Voxel::Test_Start,
        // Liquid / Fluid Blocks
        Voxel::Liquid_Acid,
        Voxel::Liquid_Blood,
        Voxel::Liquid_Lava,
        Voxel::Liquid_Molten,
        Voxel::Liquid_Ooze,
        Voxel::Liquid_Sludge,
        Voxel::Liquid_Tar,
        Voxel::Liquid_Water,
        // Frost Blocks
        Voxel::Frost_Black_Ice,
        Voxel::Frost_Fragile_Ice,
        Voxel::Frost_Ice,
        Voxel::Frost_Packed_Ice,
        // Soil Blocks
        Voxel::Soil_Ash,
        Voxel::Soil_Black_Sand,
        Voxel::Soil_Clay,
        Voxel::Soil_Dirt,
        Voxel::Soil_Grass,
        Voxel::Soil_Gravel,
        Voxel::Soil_Moss,
        Voxel::Soil_Mud,
        Voxel::Soil_Mulch,
        Voxel::Soil_Packed_Dirt,
        Voxel::Soil_Packed_Mud,
        Voxel::Soil_Packed_Peat,
        Voxel::Soil_Packed_Silt,
        Voxel::Soil_Peat,
        Voxel::Soil_Peat_Grass,
        Voxel::Soil_Peat_Mulch,
        Voxel::Soil_Red_Moss,
        Voxel::Soil_Red_Sand,
        Voxel::Soil_Rooted_Dirt,
        Voxel::Soil_Sand,
        Voxel::Soil_Scorched_Black_Sand,
        Voxel::Soil_Scorched_Red_Sand,
        Voxel::Soil_Scorched_Sand,
        Voxel::Soil_Scorched_White_Sand,
        Voxel::Soil_Silt,
        Voxel::Soil_Silt_Grass,
        Voxel::Soil_Silt_Mulch,
        Voxel::Soil_Snow,
        Voxel::Soil_Snowy_Grass,
        Voxel::Soil_Snowy_Peat,
        Voxel::Soil_Snowy_Silt,
        Voxel::Soil_White_Sand,
        // Rocky Blocks
        Voxel::Rock_Andesite,
        Voxel::Cobbled_Andesite,
        Voxel::Mossy_Andesite,
        Voxel::Mossy_Cobbled_Andesite,
        Voxel::Rock_Azurite,
        Voxel::Cobbled_Azurite,
        Voxel::Mossy_Azurite,
        Voxel::Mossy_Cobbled_Azurite,
        Voxel::Rock_Basalt,
        Voxel::Cobbled_Basalt,
        Voxel::Mossy_Basalt,
        Voxel::Mossy_Cobbled_Basalt,
        Voxel::Rock_Black_Sandstone,
        Voxel::Cobbled_Black_Sandstone,
        Voxel::Mossy_Black_Sandstone,
        Voxel::Mossy_Cobbled_Black_Sandstone,
        Voxel::Rock_Calcite,
        Voxel::Cobbled_Calcite,
        Voxel::Mossy_Calcite,
        Voxel::Mossy_Cobbled_Calcite,
        Voxel::Rock_Chalk,
        Voxel::Cobbled_Chalk,
        Voxel::Mossy_Chalk,
        Voxel::Mossy_Cobbled_Chalk,
        Voxel::Rock_Chert,
        Voxel::Cobbled_Chert,
        Voxel::Mossy_Chert,
        Voxel::Mossy_Cobbled_Chert,
        Voxel::Rock_Cinnabar,
        Voxel::Cobbled_Cinnabar,
        Voxel::Mossy_Cinnabar,
        Voxel::Mossy_Cobbled_Cinnabar,
        Voxel::Rock_Diorite,
        Voxel::Cobbled_Diorite,
        Voxel::Mossy_Diorite,
        Voxel::Mossy_Cobbled_Diorite,
        Voxel::Rock_Gabbro,
        Voxel::Cobbled_Gabbro,
        Voxel::Mossy_Gabbro,
        Voxel::Mossy_Cobbled_Gabbro,
        Voxel::Rock_Granite,
        Voxel::Cobbled_Granite,
        Voxel::Mossy_Granite,
        Voxel::Mossy_Cobbled_Granite,
        Voxel::Rock_Karst,
        Voxel::Cobbled_Karst,
        Voxel::Mossy_Karst,
        Voxel::Mossy_Cobbled_Karst,
        Voxel::Rock_Limestone,
        Voxel::Cobbled_Limestone,
        Voxel::Mossy_Limestone,
        Voxel::Mossy_Cobbled_Limestone,
        Voxel::Rock_Marl,
        Voxel::Cobbled_Marl,
        Voxel::Mossy_Marl,
        Voxel::Mossy_Cobbled_Marl,
        Voxel::Rock_Pitchstone,
        Voxel::Cobbled_Pitchstone,
        Voxel::Mossy_Pitchstone,
        Voxel::Mossy_Cobbled_Pitchstone,
        Voxel::Rock_Porphyry,
        Voxel::Cobbled_Porphyry,
        Voxel::Mossy_Porphyry,
        Voxel::Mossy_Cobbled_Porphyry,
        Voxel::Rock_Red_Sandstone,
        Voxel::Cobbled_Red_Sandstone,
        Voxel::Mossy_Red_Sandstone,
        Voxel::Mossy_Cobbled_Red_Sandstone,
        Voxel::Rock_Sandstone,
        Voxel::Cobbled_Sandstone,
        Voxel::Mossy_Sandstone,
        Voxel::Mossy_Cobbled_Sandstone,
        Voxel::Rock_Scoria,
        Voxel::Cobbled_Scoria,
        Voxel::Mossy_Scoria,
        Voxel::Mossy_Cobbled_Scoria,
        Voxel::Rock_Serpentine,
        Voxel::Cobbled_Serpentine,
        Voxel::Mossy_Serpentine,
        Voxel::Mossy_Cobbled_Serpentine,
        Voxel::Rock_Slate,
        Voxel::Cobbled_Slate,
        Voxel::Mossy_Slate,
        Voxel::Mossy_Cobbled_Slate,
        Voxel::Rock_Stone,
        Voxel::Cobbled_Stone,
        Voxel::Mossy_Stone,
        Voxel::Mossy_Cobbled_Stone,
        Voxel::Rock_Tuffite,
        Voxel::Cobbled_Tuffite,
        Voxel::Mossy_Tuffite,
        Voxel::Mossy_Cobbled_Tuffite,
        Voxel::Rock_White_Sandstone,
        Voxel::Cobbled_White_Sandstone,
        Voxel::Mossy_White_Sandstone,
        Voxel::Mossy_Cobbled_White_Sandstone,
        // Special Rocky Blocks
        Voxel::Rock_Alabaster,
        Voxel::Rock_Brimstone,
        Voxel::Rock_Cryolite,
        Voxel::Rock_Dreadstone,
        Voxel::Rock_Flint,
        Voxel::Rock_Magma,
        Voxel::Rock_Obsidian,
        Voxel::Rock_Terracotta,
        // Wood Blocks
        Voxel::Tree_Acacia_Bark,
        Voxel::Tree_Acacia_Log,
        Voxel::Tree_Acacia_Leaves,
        Voxel::Tree_Acacia_Planks,
        Voxel::Tree_Birch_Bark,
        Voxel::Tree_Birch_Log,
        Voxel::Tree_Birch_Leaves,
        Voxel::Tree_Birch_Planks,
        Voxel::Tree_Cherry_Bark,
        Voxel::Tree_Cherry_Log,
        Voxel::Tree_Cherry_Leaves,
        Voxel::Tree_Cherry_Planks,
        Voxel::Tree_Mahogany_Bark,
        Voxel::Tree_Mahogany_Log,
        Voxel::Tree_Mahogany_Leaves,
        Voxel::Tree_Mahogany_Planks,
        Voxel::Tree_Mangrove_Bark,
        Voxel::Tree_Mangrove_Log,
        Voxel::Tree_Mangrove_Leaves,
        Voxel::Tree_Mangrove_Planks,
        Voxel::Tree_Mangrove_Roots,
        Voxel::Tree_Maple_Bark,
        Voxel::Tree_Maple_Log,
        Voxel::Tree_Maple_Leaves_Red,
        Voxel::Tree_Maple_Leaves_Orange,
        Voxel::Tree_Maple_Leaves_Yellow,
        Voxel::Tree_Maple_Planks,
        Voxel::Tree_Oak_Bark,
        Voxel::Tree_Oak_Log,
        Voxel::Tree_Oak_Leaves,
        Voxel::Tree_Oak_Leaves_Lush,
        Voxel::Tree_Oak_Leaves_Flowering,
        Voxel::Tree_Oak_Planks,
        Voxel::Tree_Palm_Bark,
        Voxel::Tree_Palm_Log,
        Voxel::Tree_Palm_Leaves,
        Voxel::Tree_Palm_Planks,
        Voxel::Tree_Pine_Bark,
        Voxel::Tree_Pine_Log,
        Voxel::Tree_Pine_Leaves,
        Voxel::Tree_Pine_Planks,
        Voxel::Tree_Willow_Bark,
        Voxel::Tree_Willow_Log,
        Voxel::Tree_Willow_Leaves,
        Voxel::Tree_Willow_Planks,
        Voxel::Tree_Yew_Bark,
        Voxel::Tree_Yew_Log,
        Voxel::Tree_Yew_Leaves,
        Voxel::Tree_Yew_Planks,
        Voxel::Tree_Cactus,
        Voxel::Tree_Charred_Bark,
        Voxel::Tree_Charred_Log,
        Voxel::Tree_Charred_Planks,
        Voxel::Tree_Dead_Bark,
        Voxel::Tree_Dead_Log,
        Voxel::Tree_Dead_Planks,
        // Aquatic Blocks
        Voxel::Aqua_Algae_Mat,
        Voxel::Aqua_Brain_Coral,
        Voxel::Aqua_Bubble_Coral,
        Voxel::Aqua_Fire_Coral,
        Voxel::Aqua_Geothermal_Vent,
        Voxel::Aqua_Horn_Coral,
        Voxel::Aqua_Sponge,
        Voxel::Aqua_Tube_Coral,
        // Light Source Blocks
        Voxel::Emit_Blue_Light,
        Voxel::Emit_Blue_Torch,
        Voxel::Emit_Cold_Light,
        Voxel::Emit_Green_Light,
        Voxel::Emit_Green_Torch,
        Voxel::Emit_Red_Light,
        Voxel::Emit_Red_Torch,
        Voxel::Emit_Warm_Light,
        // Decoration Blocks
        Voxel::Deco_Barrel,
        Voxel::Deco_Basket,
        Voxel::Deco_Bone,
        Voxel::Deco_Bookshelf,
        Voxel::Deco_Brick,
        Voxel::Deco_Fabric,
        Voxel::Deco_Flesh,
        Voxel::Deco_Glass,
        Voxel::Deco_Hay,
        Voxel::Deco_Plaster,
        Voxel::Deco_Slime,
        Voxel::Deco_Stone_Path,
        Voxel::Deco_Thatch,
        Voxel::Deco_Wax,
        Voxel::Deco_Wicker,
        Voxel::Deco_Wool,
    ];

    /// The base texture name under `assets/textures/blocks/` without extension.
    pub fn texture_name(self) -> Option<&'static str> {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => None,
            // Testing / Debug Blocks
            Self::Null_Block => Some("null_block"),
            Self::Null_Liquid => Some("null_liquid"),
            Self::Test_Accept => Some("test_accept"),
            Self::Test_Debug => Some("test_debug"),
            Self::Test_Numbers => Some("test_numbers"),
            Self::Test_Fail => Some("test_fail"),
            Self::Test_Instance => Some("test_instance"),
            Self::Test_Start => Some("test_start"),
            // Liquid / Fluid Blocks
            Self::Liquid_Acid => Some("liquid_acid_still"),
            Self::Liquid_Blood => Some("liquid_blood_still"),
            Self::Liquid_Lava => Some("liquid_lava_still"),
            Self::Liquid_Molten => Some("liquid_molten_still"),
            Self::Liquid_Ooze => Some("liquid_ooze_still"),
            Self::Liquid_Sludge => Some("liquid_sludge_still"),
            Self::Liquid_Tar => Some("liquid_tar_still"),
            Self::Liquid_Water => Some("liquid_water_still"),
            // Frost Blocks
            Self::Frost_Black_Ice => Some("frost_black_ice"),
            Self::Frost_Fragile_Ice => Some("frost_fragile_ice"),
            Self::Frost_Ice => Some("frost_ice"),
            Self::Frost_Packed_Ice => Some("frost_packed_ice"),
            // Soil Blocks
            Self::Soil_Ash => Some("soil_ash"),
            Self::Soil_Black_Sand => Some("soil_black_sand"),
            Self::Soil_Clay => Some("soil_clay"),
            Self::Soil_Dirt => Some("soil_dirt"),
            Self::Soil_Grass => Some("soil_grass"),
            Self::Soil_Gravel => Some("soil_gravel"),
            Self::Soil_Moss => Some("soil_moss"),
            Self::Soil_Mud => Some("soil_mud"),
            Self::Soil_Mulch => Some("soil_mulch"),
            Self::Soil_Packed_Dirt => Some("soil_packed_dirt"),
            Self::Soil_Packed_Mud => Some("soil_packed_mud"),
            Self::Soil_Packed_Peat => Some("soil_packed_peat"),
            Self::Soil_Packed_Silt => Some("soil_packed_silt"),
            Self::Soil_Peat => Some("soil_peat"),
            Self::Soil_Peat_Grass => Some("soil_grass"),
            Self::Soil_Peat_Mulch => Some("soil_peat_mulch"),
            Self::Soil_Red_Moss => Some("soil_red_moss"),
            Self::Soil_Red_Sand => Some("soil_red_sand"),
            Self::Soil_Rooted_Dirt => Some("soil_rooted_dirt"),
            Self::Soil_Sand => Some("soil_sand"),
            Self::Soil_Scorched_Black_Sand => Some("soil_scorched_black_sand"),
            Self::Soil_Scorched_Red_Sand => Some("soil_scorched_red_sand"),
            Self::Soil_Scorched_Sand => Some("soil_scorched_sand"),
            Self::Soil_Scorched_White_Sand => Some("soil_scorched_white_sand"),
            Self::Soil_Silt => Some("soil_silt"),
            Self::Soil_Silt_Grass => Some("soil_grass"),
            Self::Soil_Silt_Mulch => Some("soil_silt_mulch"),
            Self::Soil_Snow => Some("soil_snow"),
            Self::Soil_Snowy_Grass => Some("soil_snow"),
            Self::Soil_Snowy_Peat => Some("soil_snow"),
            Self::Soil_Snowy_Silt => Some("soil_snow"),
            Self::Soil_White_Sand => Some("soil_white_sand"),
            // Rocky Blocks
            Self::Rock_Andesite => Some("rock_andesite"),
            Self::Cobbled_Andesite => Some("cobbled_andesite"),
            Self::Mossy_Andesite => Some("rock_andesite"),
            Self::Mossy_Cobbled_Andesite => Some("cobbled_andesite"),
            Self::Rock_Azurite => Some("rock_azurite"),
            Self::Cobbled_Azurite => Some("cobbled_azurite"),
            Self::Mossy_Azurite => Some("rock_azurite"),
            Self::Mossy_Cobbled_Azurite => Some("cobbled_azurite"),
            Self::Rock_Basalt => Some("rock_basalt"),
            Self::Cobbled_Basalt => Some("cobbled_basalt"),
            Self::Mossy_Basalt => Some("rock_basalt"),
            Self::Mossy_Cobbled_Basalt => Some("cobbled_basalt"),
            Self::Rock_Black_Sandstone => Some("rock_black_sandstone"),
            Self::Cobbled_Black_Sandstone => Some("cobbled_black_sandstone"),
            Self::Mossy_Black_Sandstone => Some("rock_black_sandstone"),
            Self::Mossy_Cobbled_Black_Sandstone => Some("cobbled_black_sandstone"),
            Self::Rock_Calcite => Some("rock_calcite"),
            Self::Cobbled_Calcite => Some("cobbled_calcite"),
            Self::Mossy_Calcite => Some("rock_calcite"),
            Self::Mossy_Cobbled_Calcite => Some("cobbled_calcite"),
            Self::Rock_Chalk => Some("rock_chalk"),
            Self::Cobbled_Chalk => Some("cobbled_chalk"),
            Self::Mossy_Chalk => Some("rock_chalk"),
            Self::Mossy_Cobbled_Chalk => Some("cobbled_chalk"),
            Self::Rock_Chert => Some("rock_chert"),
            Self::Cobbled_Chert => Some("cobbled_chert"),
            Self::Mossy_Chert => Some("rock_chert"),
            Self::Mossy_Cobbled_Chert => Some("cobbled_chert"),
            Self::Rock_Cinnabar => Some("rock_cinnabar"),
            Self::Cobbled_Cinnabar => Some("cobbled_cinnabar"),
            Self::Mossy_Cinnabar => Some("rock_cinnabar"),
            Self::Mossy_Cobbled_Cinnabar => Some("cobbled_cinnabar"),
            Self::Rock_Diorite => Some("rock_diorite"),
            Self::Cobbled_Diorite => Some("cobbled_diorite"),
            Self::Mossy_Diorite => Some("rock_diorite"),
            Self::Mossy_Cobbled_Diorite => Some("cobbled_diorite"),
            Self::Rock_Gabbro => Some("rock_gabbro"),
            Self::Cobbled_Gabbro => Some("cobbled_gabbro"),
            Self::Mossy_Gabbro => Some("rock_gabbro"),
            Self::Mossy_Cobbled_Gabbro => Some("cobbled_gabbro"),
            Self::Rock_Granite => Some("rock_granite"),
            Self::Cobbled_Granite => Some("cobbled_granite"),
            Self::Mossy_Granite => Some("rock_granite"),
            Self::Mossy_Cobbled_Granite => Some("cobbled_granite"),
            Self::Rock_Karst => Some("rock_karst"),
            Self::Cobbled_Karst => Some("cobbled_karst"),
            Self::Mossy_Karst => Some("rock_karst"),
            Self::Mossy_Cobbled_Karst => Some("cobbled_karst"),
            Self::Rock_Limestone => Some("rock_limestone"),
            Self::Cobbled_Limestone => Some("cobbled_limestone"),
            Self::Mossy_Limestone => Some("rock_limestone"),
            Self::Mossy_Cobbled_Limestone => Some("cobbled_limestone"),
            Self::Rock_Marl => Some("rock_marl"),
            Self::Cobbled_Marl => Some("cobbled_marl"),
            Self::Mossy_Marl => Some("rock_marl"),
            Self::Mossy_Cobbled_Marl => Some("cobbled_marl"),
            Self::Rock_Pitchstone => Some("rock_pitchstone"),
            Self::Cobbled_Pitchstone => Some("cobbled_pitchstone"),
            Self::Mossy_Pitchstone => Some("rock_pitchstone"),
            Self::Mossy_Cobbled_Pitchstone => Some("cobbled_pitchstone"),
            Self::Rock_Porphyry => Some("rock_porphyry"),
            Self::Cobbled_Porphyry => Some("cobbled_porphyry"),
            Self::Mossy_Porphyry => Some("rock_porphyry"),
            Self::Mossy_Cobbled_Porphyry => Some("cobbled_porphyry"),
            Self::Rock_Red_Sandstone => Some("rock_red_sandstone"),
            Self::Cobbled_Red_Sandstone => Some("cobbled_red_sandstone"),
            Self::Mossy_Red_Sandstone => Some("rock_red_sandstone"),
            Self::Mossy_Cobbled_Red_Sandstone => Some("cobbled_red_sandstone"),
            Self::Rock_Sandstone => Some("rock_sandstone"),
            Self::Cobbled_Sandstone => Some("cobbled_sandstone"),
            Self::Mossy_Sandstone => Some("rock_sandstone"),
            Self::Mossy_Cobbled_Sandstone => Some("cobbled_sandstone"),
            Self::Rock_Scoria => Some("rock_scoria"),
            Self::Cobbled_Scoria => Some("cobbled_scoria"),
            Self::Mossy_Scoria => Some("rock_scoria"),
            Self::Mossy_Cobbled_Scoria => Some("cobbled_scoria"),
            Self::Rock_Serpentine => Some("rock_serpentine"),
            Self::Cobbled_Serpentine => Some("cobbled_serpentine"),
            Self::Mossy_Serpentine => Some("rock_serpentine"),
            Self::Mossy_Cobbled_Serpentine => Some("cobbled_serpentine"),
            Self::Rock_Slate => Some("rock_slate"),
            Self::Cobbled_Slate => Some("cobbled_slate"),
            Self::Mossy_Slate => Some("rock_slate"),
            Self::Mossy_Cobbled_Slate => Some("cobbled_slate"),
            Self::Rock_Stone => Some("rock_stone"),
            Self::Cobbled_Stone => Some("cobbled_stone"),
            Self::Mossy_Stone => Some("rock_stone"),
            Self::Mossy_Cobbled_Stone => Some("cobbled_stone"),
            Self::Rock_Tuffite => Some("rock_tuffite"),
            Self::Cobbled_Tuffite => Some("cobbled_tuff"),
            Self::Mossy_Tuffite => Some("rock_tuffite"),
            Self::Mossy_Cobbled_Tuffite => Some("cobbled_tuff"),
            Self::Rock_White_Sandstone => Some("rock_white_sandstone"),
            Self::Cobbled_White_Sandstone => Some("cobbled_white_sandstone"),
            Self::Mossy_White_Sandstone => Some("rock_white_sandstone"),
            Self::Mossy_Cobbled_White_Sandstone => Some("cobbled_white_sandstone"),
            // Special Rocky Blocks
            Self::Rock_Alabaster => Some("rock_alabaster"),
            Self::Rock_Brimstone => Some("rock_brimstone"),
            Self::Rock_Cryolite => Some("rock_cryolite"),
            Self::Rock_Dreadstone => Some("rock_dreadstone"),
            Self::Rock_Flint => Some("rock_flint"),
            Self::Rock_Magma => Some("rock_magma"),
            Self::Rock_Obsidian => Some("rock_obsidian"),
            Self::Rock_Terracotta => Some("rock_terracotta"),
            // Wood Blocks
            Self::Tree_Acacia_Bark => Some("tree_acacia_bark"),
            Self::Tree_Acacia_Log => Some("tree_acacia_log"),
            Self::Tree_Acacia_Leaves => Some("tree_acacia_leaves"),
            Self::Tree_Acacia_Planks => Some("tree_acacia_planks"),
            Self::Tree_Birch_Bark => Some("tree_birch_bark"),
            Self::Tree_Birch_Log => Some("tree_birch_log"),
            Self::Tree_Birch_Leaves => Some("tree_birch_leaves"),
            Self::Tree_Birch_Planks => Some("tree_birch_planks"),
            Self::Tree_Cherry_Bark => Some("tree_cherry_bark"),
            Self::Tree_Cherry_Log => Some("tree_cherry_log"),
            Self::Tree_Cherry_Leaves => Some("tree_cherry_leaves"),
            Self::Tree_Cherry_Planks => Some("tree_cherry_planks"),
            Self::Tree_Mahogany_Bark => Some("tree_mahogany_bark"),
            Self::Tree_Mahogany_Log => Some("tree_mahogany_log"),
            Self::Tree_Mahogany_Leaves => Some("tree_mahogany_leaves"),
            Self::Tree_Mahogany_Planks => Some("tree_mahogany_planks"),
            Self::Tree_Mangrove_Bark => Some("tree_mangrove_bark"),
            Self::Tree_Mangrove_Log => Some("tree_mangrove_log"),
            Self::Tree_Mangrove_Leaves => Some("tree_mangrove_leaves"),
            Self::Tree_Mangrove_Planks => Some("tree_mangrove_planks"),
            Self::Tree_Mangrove_Roots => Some("tree_mangrove_roots"),
            Self::Tree_Maple_Bark => Some("tree_maple_bark"),
            Self::Tree_Maple_Log => Some("tree_maple_log"),
            Self::Tree_Maple_Leaves_Red => Some("tree_maple_leaves_red"),
            Self::Tree_Maple_Leaves_Orange => Some("tree_maple_leaves_orange_"),
            Self::Tree_Maple_Leaves_Yellow => Some("tree_maple_leaves_yellow"),
            Self::Tree_Maple_Planks => Some("tree_maple_planks"),
            Self::Tree_Oak_Bark => Some("tree_oak_bark"),
            Self::Tree_Oak_Log => Some("tree_oak_log"),
            Self::Tree_Oak_Leaves => Some("tree_oak_leaves"),
            Self::Tree_Oak_Leaves_Lush => Some("tree_oak_leaves_lush"),
            Self::Tree_Oak_Leaves_Flowering => Some("tree_oak_leaves_flowering"),
            Self::Tree_Oak_Planks => Some("tree_oak_planks"),
            Self::Tree_Palm_Bark => Some("tree_palm_bark"),
            Self::Tree_Palm_Log => Some("tree_palm_log"),
            Self::Tree_Palm_Leaves => Some("tree_palm_leaves"),
            Self::Tree_Palm_Planks => Some("tree_palm_planks"),
            Self::Tree_Pine_Bark => Some("tree_pine_bark"),
            Self::Tree_Pine_Log => Some("tree_pine_log"),
            Self::Tree_Pine_Leaves => Some("tree_pine_leaves"),
            Self::Tree_Pine_Planks => Some("tree_pine_planks"),
            Self::Tree_Willow_Bark => Some("tree_willow_bark"),
            Self::Tree_Willow_Log => Some("tree_willow_log"),
            Self::Tree_Willow_Leaves => Some("tree_willow_leaves"),
            Self::Tree_Willow_Planks => Some("tree_willow_planks"),
            Self::Tree_Yew_Bark => Some("tree_yew_bark"),
            Self::Tree_Yew_Log => Some("tree_yew_log"),
            Self::Tree_Yew_Leaves => Some("tree_yew_leaves"),
            Self::Tree_Yew_Planks => Some("tree_yew_planks"),
            Self::Tree_Cactus => Some("tree_cactus_side"),
            Self::Tree_Charred_Bark => Some("tree_charred_bark"),
            Self::Tree_Charred_Log => Some("tree_charred_log"),
            Self::Tree_Charred_Planks => Some("tree_charred_planks"),
            Self::Tree_Dead_Bark => Some("tree_dead_bark"),
            Self::Tree_Dead_Log => Some("tree_dead_log"),
            Self::Tree_Dead_Planks => Some("tree_dead_planks"),
            // Aquatic Blocks
            Self::Aqua_Algae_Mat => Some("aqua_algae_mat"),
            Self::Aqua_Brain_Coral => Some("aqua_brain_coral"),
            Self::Aqua_Bubble_Coral => Some("aqua_bubble_coral"),
            Self::Aqua_Fire_Coral => Some("aqua_fire_coral"),
            Self::Aqua_Geothermal_Vent => Some("aqua_geothermal_vent"),
            Self::Aqua_Horn_Coral => Some("aqua_horn_coral"),
            Self::Aqua_Sponge => Some("aqua_sponge"),
            Self::Aqua_Tube_Coral => Some("aqua_tube_coral"),
            // Light Source Blocks
            Self::Emit_Blue_Light => Some("emit_blue_light"),
            Self::Emit_Blue_Torch => Some("emit_blue_torch"),
            Self::Emit_Cold_Light => Some("emit_cold_light"),
            Self::Emit_Green_Light => Some("emit_green_light"),
            Self::Emit_Green_Torch => Some("emit_green_torch"),
            Self::Emit_Red_Light => Some("emit_red_light"),
            Self::Emit_Red_Torch => Some("emit_red_torch"),
            Self::Emit_Warm_Light => Some("emit_warm_light"),
            // Decoration Blocks
            Self::Deco_Barrel => Some("deco_barrel"),
            Self::Deco_Basket => Some("deco_basket_side"),
            Self::Deco_Bone => Some("deco_bone"),
            Self::Deco_Bookshelf => Some("deco_bookshelf"),
            Self::Deco_Brick => Some("deco_bricks"),
            Self::Deco_Fabric => Some("deco_fabric"),
            Self::Deco_Flesh => Some("deco_flesh"),
            Self::Deco_Glass => Some("deco_glass"),
            Self::Deco_Hay => Some("deco_hay"),
            Self::Deco_Plaster => Some("deco_plaster"),
            Self::Deco_Slime => Some("deco_slime"),
            Self::Deco_Stone_Path => Some("deco_stone_path"),
            Self::Deco_Thatch => Some("deco_thatch"),
            Self::Deco_Wax => Some("deco_wax"),
            Self::Deco_Wicker => Some("deco_wicker"),
            Self::Deco_Wool => Some("deco_wool"),
        }
    }

    /// Optional texture override for the side faces (+X, -X, +Z, -Z).
    /// If None, falls back to `texture_name()`.
    pub fn side_texture_name(self) -> Option<&'static str> {
        match self {
            Self::Rock_Basalt | Self::Mossy_Basalt => Some("rock_basalt_side"),
            Self::Soil_Grass => Some("soil_grass_side"),
            Self::Soil_Peat_Grass => Some("soil_peat_grass_side"),
            Self::Soil_Silt_Grass => Some("soil_silt_grass_side"),
            Self::Soil_Mulch => Some("soil_mulch_side"),
            Self::Soil_Peat_Mulch => Some("soil_peat_mulch_side"),
            Self::Soil_Silt_Mulch => Some("soil_silt_mulch_side"),
            Self::Soil_Snowy_Grass => Some("soil_snowy_grass_side"),
            Self::Soil_Snowy_Peat => Some("soil_snowy_peat_side"),
            Self::Soil_Snowy_Silt => Some("soil_snowy_silt_side"),
            Self::Tree_Acacia_Log => Some("tree_acacia_bark"),
            Self::Tree_Birch_Log => Some("tree_birch_bark"),
            Self::Tree_Charred_Log => Some("tree_charred_bark"),
            Self::Tree_Cherry_Log => Some("tree_cherry_bark"),
            Self::Tree_Dead_Log => Some("tree_dead_bark"),
            Self::Tree_Mahogany_Log => Some("tree_mahogany_bark"),
            Self::Tree_Mangrove_Log => Some("tree_mangrove_bark"),
            Self::Tree_Maple_Log => Some("tree_maple_bark"),
            Self::Tree_Oak_Log => Some("tree_oak_bark"),
            Self::Tree_Palm_Log => Some("tree_palm_bark"),
            Self::Tree_Pine_Log => Some("tree_pine_bark"),
            Self::Tree_Willow_Log => Some("tree_willow_bark"),
            Self::Tree_Yew_Log => Some("tree_yew_bark"),
            Self::Tree_Cactus => Some("tree_cactus_side"),
            Self::Tree_Mangrove_Roots => Some("tree_mangrove_roots"),
            Self::Aqua_Algae_Mat => Some("aqua_algae_mat"),
            Self::Aqua_Geothermal_Vent => Some("aqua_geothermal_vent_side"),
            Self::Deco_Barrel => Some("deco_barrel"),
            Self::Deco_Basket => Some("deco_basket_side"),
            Self::Deco_Bookshelf => Some("deco_bookshelf"),
            Self::Deco_Bone => Some("deco_bone"),
            Self::Deco_Hay => Some("deco_hay"),
            Self::Deco_Thatch => Some("deco_thatch_sides"),
            _ => None,
        }
    }

    /// Optional texture override for the top face (+Y).
    /// If None, falls back to `texture_name()`.
    pub fn top_texture_name(self) -> Option<&'static str> {
        match self {
            Self::Rock_Basalt | Self::Mossy_Basalt => Some("rock_basalt"),
            Self::Soil_Grass | Self::Soil_Peat_Grass | Self::Soil_Silt_Grass => Some("soil_grass"),
            Self::Soil_Mulch => Some("soil_mulch"),
            Self::Soil_Peat_Mulch => Some("soil_peat_mulch"),
            Self::Soil_Silt_Mulch => Some("soil_silt_mulch"),
            Self::Soil_Snowy_Grass | Self::Soil_Snowy_Peat | Self::Soil_Snowy_Silt => Some("soil_snow"),
            Self::Tree_Acacia_Log => Some("tree_acacia_log"),
            Self::Tree_Birch_Log => Some("tree_birch_log"),
            Self::Tree_Charred_Log => Some("tree_charred_log"),
            Self::Tree_Cherry_Log => Some("tree_cherry_log"),
            Self::Tree_Dead_Log => Some("tree_dead_log"),
            Self::Tree_Mahogany_Log => Some("tree_mahogany_log"),
            Self::Tree_Mangrove_Log => Some("tree_mangrove_log"),
            Self::Tree_Maple_Log => Some("tree_maple_log"),
            Self::Tree_Oak_Log => Some("tree_oak_log"),
            Self::Tree_Palm_Log => Some("tree_palm_log"),
            Self::Tree_Pine_Log => Some("tree_pine_log"),
            Self::Tree_Willow_Log => Some("tree_willow_log"),
            Self::Tree_Yew_Log => Some("tree_yew_log"),
            Self::Tree_Cactus => Some("tree_cactus_top"),
            Self::Tree_Mangrove_Roots => Some("tree_mangrove_roots_top"),
            Self::Aqua_Algae_Mat => Some("aqua_algae_mat_top"),
            Self::Aqua_Geothermal_Vent => Some("aqua_geothermal_vent"),
            Self::Deco_Barrel => Some("deco_barrel_top"),
            Self::Deco_Basket => Some("deco_basket_top"),
            Self::Deco_Bookshelf => Some("deco_bookshelf_top"),
            Self::Deco_Bone => Some("deco_bone_top"),
            Self::Deco_Hay => Some("deco_hay_top"),
            Self::Deco_Thatch => Some("deco_thatch"),
            _ => None,
        }
    }

    /// Optional texture override for the bottom face (-Y).
    /// If None, falls back to `texture_name()`.
    pub fn bottom_texture_name(self) -> Option<&'static str> {
        match self {
            Self::Rock_Basalt | Self::Mossy_Basalt => Some("rock_basalt"),
            Self::Soil_Grass | Self::Soil_Snowy_Grass | Self::Soil_Mulch => Some("soil_dirt"),
            Self::Soil_Peat_Grass | Self::Soil_Snowy_Peat | Self::Soil_Peat_Mulch => Some("soil_peat"),
            Self::Soil_Silt_Grass | Self::Soil_Snowy_Silt | Self::Soil_Silt_Mulch => Some("soil_silt"),
            Self::Tree_Acacia_Log => Some("tree_acacia_log"),
            Self::Tree_Birch_Log => Some("tree_birch_log"),
            Self::Tree_Charred_Log => Some("tree_charred_log"),
            Self::Tree_Cherry_Log => Some("tree_cherry_log"),
            Self::Tree_Dead_Log => Some("tree_dead_log"),
            Self::Tree_Mahogany_Log => Some("tree_mahogany_log"),
            Self::Tree_Mangrove_Log => Some("tree_mangrove_log"),
            Self::Tree_Maple_Log => Some("tree_maple_log"),
            Self::Tree_Oak_Log => Some("tree_oak_log"),
            Self::Tree_Palm_Log => Some("tree_palm_log"),
            Self::Tree_Pine_Log => Some("tree_pine_log"),
            Self::Tree_Willow_Log => Some("tree_willow_log"),
            Self::Tree_Yew_Log => Some("tree_yew_log"),
            Self::Tree_Cactus => Some("tree_cactus_bot"),
            Self::Tree_Mangrove_Roots => Some("tree_mangrove_roots_top"),
            Self::Aqua_Algae_Mat => Some("aqua_algae_mat_top"),
            Self::Aqua_Geothermal_Vent => Some("aqua_geothermal_vent"),
            Self::Deco_Barrel => Some("deco_barrel_bot"),
            Self::Deco_Basket => Some("deco_basket_bot"),
            Self::Deco_Bookshelf => Some("deco_bookshelf_top"),
            Self::Deco_Bone => Some("deco_bone_top"),
            Self::Deco_Hay => Some("deco_hay_top"),
            Self::Deco_Thatch => Some("deco_thatch"),
            _ => None,
        }
    }

    /// Optional overlay texture (like mossy rock or mossy cobbled layers) rendered on top of the base texture.
    pub fn overlay_texture_name(&self) -> Option<&'static str> {
        match self {
            Self::Mossy_Andesite |
            Self::Mossy_Azurite |
            Self::Mossy_Basalt |
            Self::Mossy_Black_Sandstone |
            Self::Mossy_Calcite |
            Self::Mossy_Chalk |
            Self::Mossy_Chert |
            Self::Mossy_Cinnabar |
            Self::Mossy_Diorite |
            Self::Mossy_Gabbro |
            Self::Mossy_Granite |
            Self::Mossy_Karst |
            Self::Mossy_Limestone |
            Self::Mossy_Marl |
            Self::Mossy_Pitchstone |
            Self::Mossy_Porphyry |
            Self::Mossy_Red_Sandstone |
            Self::Mossy_Sandstone |
            Self::Mossy_Scoria |
            Self::Mossy_Serpentine |
            Self::Mossy_Slate |
            Self::Mossy_Stone |
            Self::Mossy_Tuffite |
            Self::Mossy_White_Sandstone => Some("mossy_rock_overlay"),

            Self::Mossy_Cobbled_Andesite |
            Self::Mossy_Cobbled_Azurite |
            Self::Mossy_Cobbled_Basalt |
            Self::Mossy_Cobbled_Black_Sandstone |
            Self::Mossy_Cobbled_Calcite |
            Self::Mossy_Cobbled_Chalk |
            Self::Mossy_Cobbled_Chert |
            Self::Mossy_Cobbled_Cinnabar |
            Self::Mossy_Cobbled_Diorite |
            Self::Mossy_Cobbled_Gabbro |
            Self::Mossy_Cobbled_Granite |
            Self::Mossy_Cobbled_Karst |
            Self::Mossy_Cobbled_Limestone |
            Self::Mossy_Cobbled_Marl |
            Self::Mossy_Cobbled_Pitchstone |
            Self::Mossy_Cobbled_Porphyry |
            Self::Mossy_Cobbled_Red_Sandstone |
            Self::Mossy_Cobbled_Sandstone |
            Self::Mossy_Cobbled_Scoria |
            Self::Mossy_Cobbled_Serpentine |
            Self::Mossy_Cobbled_Slate |
            Self::Mossy_Cobbled_Stone |
            Self::Mossy_Cobbled_Tuffite |
            Self::Mossy_Cobbled_White_Sandstone => Some("mossy_cobbled_overlay"),

            _ => None,
        }
    }

    /// Whether this voxel receives custom biome or foliage tinting.
    pub fn is_tinted(self) -> bool {
        matches!(
            self,
            Self::Soil_Grass
                | Self::Soil_Peat_Grass
                | Self::Soil_Silt_Grass
                | Self::Soil_Snowy_Grass
                | Self::Liquid_Water
                | Self::WaterOccupied
                | Self::Tree_Acacia_Leaves
                | Self::Tree_Birch_Leaves
                | Self::Tree_Mahogany_Leaves
                | Self::Tree_Mangrove_Leaves
                | Self::Tree_Oak_Leaves
                | Self::Tree_Palm_Leaves
                | Self::Tree_Pine_Leaves
                | Self::Tree_Willow_Leaves
                | Self::Tree_Yew_Leaves
        )
    }

    /// Color tint applied to vertices for biome / atmospheric coloring.
    pub fn tint_color(self) -> [f32; 4] {
        match self {
            Self::Soil_Grass | Self::Soil_Peat_Grass | Self::Soil_Silt_Grass => [0.55, 0.94, 0.42, 1.0],
            Self::Soil_Snowy_Grass => [0.90, 0.95, 1.0, 1.0],
            Self::Liquid_Water | Self::WaterOccupied => [0.35, 0.65, 0.92, 1.0],
            Self::Tree_Oak_Leaves => [0.60, 1.15, 0.35, 1.0],
            Self::Tree_Birch_Leaves => [0.85, 1.25, 0.40, 1.0],
            Self::Tree_Pine_Leaves => [0.40, 0.90, 0.55, 1.0],
            Self::Tree_Acacia_Leaves => [0.72, 0.92, 0.28, 1.0],
            Self::Tree_Mahogany_Leaves => [0.45, 1.00, 0.40, 1.0],
            Self::Tree_Mangrove_Leaves => [0.40, 0.95, 0.42, 1.0],
            Self::Tree_Palm_Leaves => [0.55, 1.05, 0.30, 1.0],
            Self::Tree_Willow_Leaves => [0.52, 0.88, 0.48, 1.0],
            Self::Tree_Yew_Leaves => [0.28, 0.70, 0.38, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Biome-aware blended color tint at a specific world coordinate.
    pub fn tint_color_at(self, world_voxel: IVec3) -> [f32; 4] {
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

    /// Fallback 1x1 solid RGBA pixel if the texture file is not found on disk.
    pub fn fallback_color(self) -> [u8; 4] {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => [0, 0, 0, 0],
            // Testing / Debug Blocks
            Self::Null_Block => [255, 0, 255, 255],
            Self::Null_Liquid => [255, 0, 255, 255],
            Self::Test_Accept => [50, 200, 50, 255],
            Self::Test_Debug => [200, 200, 50, 255],
            Self::Test_Numbers => [200, 200, 50, 255],
            Self::Test_Fail => [200, 50, 50, 255],
            Self::Test_Instance => [200, 200, 50, 255],
            Self::Test_Start => [50, 150, 250, 255],
            // Liquid / Fluid Blocks
            Self::Liquid_Acid => [120, 230, 40, 255],
            Self::Liquid_Blood => [180, 20, 20, 255],
            Self::Liquid_Lava => [230, 100, 20, 255],
            Self::Liquid_Molten => [255, 140, 0, 255],
            Self::Liquid_Ooze => [130, 50, 160, 255],
            Self::Liquid_Sludge => [70, 80, 50, 255],
            Self::Liquid_Tar => [25, 25, 30, 255],
            Self::Liquid_Water => [60, 140, 220, 255],
            // Frost Blocks
            Self::Frost_Black_Ice => [30, 35, 45, 170],
            Self::Frost_Fragile_Ice => [140, 185, 235, 200],
            Self::Frost_Ice => [140, 185, 235, 220],
            Self::Frost_Packed_Ice => [160, 200, 245, 255],
            // Soil Blocks
            Self::Soil_Ash => [140, 140, 145, 255],
            Self::Soil_Black_Sand => [45, 45, 50, 255],
            Self::Soil_Clay => [150, 155, 175, 255],
            Self::Soil_Dirt => [107, 66, 33, 255],
            Self::Soil_Grass => [110, 180, 80, 255],
            Self::Soil_Gravel => [130, 125, 125, 255],
            Self::Soil_Moss => [80, 130, 60, 255],
            Self::Soil_Mud => [85, 60, 45, 255],
            Self::Soil_Mulch => [90, 55, 35, 255],
            Self::Soil_Packed_Dirt => [100, 60, 30, 255],
            Self::Soil_Packed_Mud => [80, 55, 40, 255],
            Self::Soil_Packed_Peat => [70, 50, 35, 255],
            Self::Soil_Packed_Silt => [115, 110, 100, 255],
            Self::Soil_Peat => [75, 55, 40, 255],
            Self::Soil_Peat_Grass => [95, 150, 60, 255],
            Self::Soil_Peat_Mulch => [80, 50, 35, 255],
            Self::Soil_Red_Moss => [175, 45, 45, 255],
            Self::Soil_Red_Sand => [190, 100, 50, 255],
            Self::Soil_Rooted_Dirt => [115, 75, 40, 255],
            Self::Soil_Sand => [209, 194, 128, 255],
            Self::Soil_Scorched_Black_Sand => [35, 35, 40, 255],
            Self::Soil_Scorched_Red_Sand => [160, 70, 40, 255],
            Self::Soil_Scorched_Sand => [175, 150, 100, 255],
            Self::Soil_Scorched_White_Sand => [190, 190, 185, 255],
            Self::Soil_Silt => [130, 125, 115, 255],
            Self::Soil_Silt_Grass => [110, 160, 70, 255],
            Self::Soil_Silt_Mulch => [100, 80, 60, 255],
            Self::Soil_Snow => [240, 245, 255, 255],
            Self::Soil_Snowy_Grass => [240, 245, 255, 255],
            Self::Soil_Snowy_Peat => [220, 225, 235, 255],
            Self::Soil_Snowy_Silt => [225, 230, 240, 255],
            Self::Soil_White_Sand => [235, 230, 215, 255],
            // Rocky Blocks
            Self::Rock_Andesite => [136, 136, 136, 255],
            Self::Cobbled_Andesite => [136, 136, 136, 255],
            Self::Mossy_Andesite => [122, 134, 117, 255],
            Self::Mossy_Cobbled_Andesite => [122, 134, 117, 255],
            Self::Rock_Azurite => [45, 85, 160, 255],
            Self::Cobbled_Azurite => [45, 85, 160, 255],
            Self::Mossy_Azurite => [53, 96, 135, 255],
            Self::Mossy_Cobbled_Azurite => [53, 96, 135, 255],
            Self::Rock_Basalt => [75, 75, 80, 255],
            Self::Cobbled_Basalt => [75, 75, 80, 255],
            Self::Mossy_Basalt => [76, 88, 75, 255],
            Self::Mossy_Cobbled_Basalt => [76, 88, 75, 255],
            Self::Rock_Black_Sandstone => [50, 50, 55, 255],
            Self::Cobbled_Black_Sandstone => [50, 50, 55, 255],
            Self::Mossy_Black_Sandstone => [57, 70, 56, 255],
            Self::Mossy_Cobbled_Black_Sandstone => [57, 70, 56, 255],
            Self::Rock_Calcite => [220, 220, 225, 255],
            Self::Cobbled_Calcite => [220, 220, 225, 255],
            Self::Mossy_Calcite => [185, 197, 183, 255],
            Self::Mossy_Cobbled_Calcite => [185, 197, 183, 255],
            Self::Rock_Chalk => [235, 235, 230, 255],
            Self::Cobbled_Chalk => [235, 235, 230, 255],
            Self::Mossy_Chalk => [196, 208, 187, 255],
            Self::Mossy_Cobbled_Chalk => [196, 208, 187, 255],
            Self::Rock_Chert => [160, 140, 120, 255],
            Self::Cobbled_Chert => [160, 140, 120, 255],
            Self::Mossy_Chert => [140, 137, 105, 255],
            Self::Mossy_Cobbled_Chert => [140, 137, 105, 255],
            Self::Rock_Cinnabar => [160, 50, 50, 255],
            Self::Cobbled_Cinnabar => [160, 50, 50, 255],
            Self::Mossy_Cinnabar => [140, 70, 52, 255],
            Self::Mossy_Cobbled_Cinnabar => [140, 70, 52, 255],
            Self::Rock_Diorite => [180, 180, 185, 255],
            Self::Cobbled_Diorite => [180, 180, 185, 255],
            Self::Mossy_Diorite => [155, 167, 153, 255],
            Self::Mossy_Cobbled_Diorite => [155, 167, 153, 255],
            Self::Rock_Gabbro => [90, 95, 95, 255],
            Self::Cobbled_Gabbro => [90, 95, 95, 255],
            Self::Mossy_Gabbro => [87, 103, 86, 255],
            Self::Mossy_Cobbled_Gabbro => [87, 103, 86, 255],
            Self::Rock_Granite => [150, 105, 90, 255],
            Self::Cobbled_Granite => [150, 105, 90, 255],
            Self::Mossy_Granite => [132, 111, 82, 255],
            Self::Mossy_Cobbled_Granite => [132, 111, 82, 255],
            Self::Rock_Karst => [140, 145, 140, 255],
            Self::Cobbled_Karst => [140, 145, 140, 255],
            Self::Mossy_Karst => [125, 141, 120, 255],
            Self::Mossy_Cobbled_Karst => [125, 141, 120, 255],
            Self::Rock_Limestone => [195, 185, 165, 255],
            Self::Cobbled_Limestone => [195, 185, 165, 255],
            Self::Mossy_Limestone => [166, 171, 138, 255],
            Self::Mossy_Cobbled_Limestone => [166, 171, 138, 255],
            Self::Rock_Marl => [170, 165, 150, 255],
            Self::Cobbled_Marl => [170, 165, 150, 255],
            Self::Mossy_Marl => [147, 156, 127, 255],
            Self::Mossy_Cobbled_Marl => [147, 156, 127, 255],
            Self::Rock_Pitchstone => [60, 70, 65, 255],
            Self::Cobbled_Pitchstone => [60, 70, 65, 255],
            Self::Mossy_Pitchstone => [65, 85, 63, 255],
            Self::Mossy_Cobbled_Pitchstone => [65, 85, 63, 255],
            Self::Rock_Porphyry => [140, 95, 105, 255],
            Self::Cobbled_Porphyry => [140, 95, 105, 255],
            Self::Mossy_Porphyry => [125, 103, 93, 255],
            Self::Mossy_Cobbled_Porphyry => [125, 103, 93, 255],
            Self::Rock_Red_Sandstone => [185, 95, 45, 255],
            Self::Cobbled_Red_Sandstone => [185, 95, 45, 255],
            Self::Mossy_Red_Sandstone => [158, 103, 48, 255],
            Self::Mossy_Cobbled_Red_Sandstone => [158, 103, 48, 255],
            Self::Rock_Sandstone => [215, 205, 150, 255],
            Self::Cobbled_Sandstone => [215, 205, 150, 255],
            Self::Mossy_Sandstone => [181, 186, 127, 255],
            Self::Mossy_Cobbled_Sandstone => [181, 186, 127, 255],
            Self::Rock_Scoria => [105, 60, 55, 255],
            Self::Cobbled_Scoria => [105, 60, 55, 255],
            Self::Mossy_Scoria => [98, 77, 56, 255],
            Self::Mossy_Cobbled_Scoria => [98, 77, 56, 255],
            Self::Rock_Serpentine => [70, 115, 85, 255],
            Self::Cobbled_Serpentine => [70, 115, 85, 255],
            Self::Mossy_Serpentine => [72, 118, 78, 255],
            Self::Mossy_Cobbled_Serpentine => [72, 118, 78, 255],
            Self::Rock_Slate => [80, 85, 95, 255],
            Self::Cobbled_Slate => [80, 85, 95, 255],
            Self::Mossy_Slate => [80, 96, 86, 255],
            Self::Mossy_Cobbled_Slate => [80, 96, 86, 255],
            Self::Rock_Stone => [122, 128, 133, 255],
            Self::Cobbled_Stone => [122, 128, 133, 255],
            Self::Mossy_Stone => [111, 128, 114, 255],
            Self::Mossy_Cobbled_Stone => [111, 128, 114, 255],
            Self::Rock_Tuffite => [105, 108, 100, 255],
            Self::Cobbled_Tuffite => [105, 108, 100, 255],
            Self::Mossy_Tuffite => [98, 113, 90, 255],
            Self::Mossy_Cobbled_Tuffite => [98, 113, 90, 255],
            Self::Rock_White_Sandstone => [230, 225, 210, 255],
            Self::Cobbled_White_Sandstone => [230, 225, 210, 255],
            Self::Mossy_White_Sandstone => [192, 201, 172, 255],
            Self::Mossy_Cobbled_White_Sandstone => [192, 201, 172, 255],
            // Special Rocky Blocks
            Self::Rock_Alabaster => [220, 220, 220, 255],
            Self::Rock_Brimstone => [210, 180, 50, 255],
            Self::Rock_Cryolite => [200, 230, 240, 255],
            Self::Rock_Dreadstone => [35, 30, 40, 255],
            Self::Rock_Flint => [55, 55, 60, 255],
            Self::Rock_Magma => [180, 70, 30, 255],
            Self::Rock_Obsidian => [25, 20, 35, 255],
            Self::Rock_Terracotta => [165, 95, 65, 255],
            // Wood Blocks
            Self::Tree_Acacia_Bark => [160, 85, 45, 255],
            Self::Tree_Acacia_Log => [160, 85, 45, 255],
            Self::Tree_Acacia_Leaves => [100, 140, 40, 255],
            Self::Tree_Acacia_Planks => [180, 100, 50, 255],
            Self::Tree_Birch_Bark => [225, 222, 210, 255],
            Self::Tree_Birch_Log => [225, 222, 210, 255],
            Self::Tree_Birch_Leaves => [133, 199, 56, 255],
            Self::Tree_Birch_Planks => [200, 195, 180, 255],
            Self::Tree_Cherry_Bark => [57, 39, 48, 255],
            Self::Tree_Cherry_Log => [57, 39, 48, 255],
            Self::Tree_Cherry_Leaves => [230, 176, 197, 255],
            Self::Tree_Cherry_Planks => [155, 125, 110, 255],
            Self::Tree_Mahogany_Bark => [100, 50, 40, 255],
            Self::Tree_Mahogany_Log => [100, 50, 40, 255],
            Self::Tree_Mahogany_Leaves => [70, 120, 45, 255],
            Self::Tree_Mahogany_Planks => [120, 60, 45, 255],
            Self::Tree_Mangrove_Bark => [110, 75, 60, 255],
            Self::Tree_Mangrove_Log => [110, 75, 60, 255],
            Self::Tree_Mangrove_Leaves => [65, 125, 55, 255],
            Self::Tree_Mangrove_Planks => [130, 85, 65, 255],
            Self::Tree_Mangrove_Roots => [95, 65, 50, 255],
            Self::Tree_Maple_Bark => [130, 90, 60, 255],
            Self::Tree_Maple_Log => [130, 90, 60, 255],
            Self::Tree_Maple_Leaves_Red => [185, 45, 30, 255],
            Self::Tree_Maple_Leaves_Orange => [225, 115, 25, 255],
            Self::Tree_Maple_Leaves_Yellow => [225, 185, 30, 255],
            Self::Tree_Maple_Planks => [150, 105, 70, 255],
            Self::Tree_Oak_Bark => [133, 94, 56, 255],
            Self::Tree_Oak_Log => [133, 94, 56, 255],
            Self::Tree_Oak_Leaves => [87, 166, 46, 255],
            Self::Tree_Oak_Leaves_Lush => [69, 91, 36, 255],
            Self::Tree_Oak_Leaves_Flowering => [87, 93, 51, 255],
            Self::Tree_Oak_Planks => [155, 115, 75, 255],
            Self::Tree_Palm_Bark => [140, 110, 70, 255],
            Self::Tree_Palm_Log => [140, 110, 70, 255],
            Self::Tree_Palm_Leaves => [90, 160, 50, 255],
            Self::Tree_Palm_Planks => [160, 130, 85, 255],
            Self::Tree_Pine_Bark => [74, 48, 28, 255],
            Self::Tree_Pine_Log => [74, 48, 28, 255],
            Self::Tree_Pine_Leaves => [46, 107, 66, 255],
            Self::Tree_Pine_Planks => [115, 80, 50, 255],
            Self::Tree_Willow_Bark => [90, 85, 60, 255],
            Self::Tree_Willow_Log => [90, 85, 60, 255],
            Self::Tree_Willow_Leaves => [75, 130, 50, 255],
            Self::Tree_Willow_Planks => [110, 105, 75, 255],
            Self::Tree_Yew_Bark => [105, 65, 45, 255],
            Self::Tree_Yew_Log => [105, 65, 45, 255],
            Self::Tree_Yew_Leaves => [45, 95, 55, 255],
            Self::Tree_Yew_Planks => [125, 80, 55, 255],
            Self::Tree_Cactus => [85, 135, 45, 255],
            Self::Tree_Charred_Bark => [30, 30, 30, 255],
            Self::Tree_Charred_Log => [35, 35, 35, 255],
            Self::Tree_Charred_Planks => [45, 45, 45, 255],
            Self::Tree_Dead_Bark => [110, 100, 90, 255],
            Self::Tree_Dead_Log => [120, 110, 100, 255],
            Self::Tree_Dead_Planks => [135, 125, 115, 255],
            // Aquatic Blocks
            Self::Aqua_Algae_Mat => [50, 120, 70, 255],
            Self::Aqua_Brain_Coral => [220, 100, 130, 255],
            Self::Aqua_Bubble_Coral => [180, 70, 160, 255],
            Self::Aqua_Fire_Coral => [220, 50, 50, 255],
            Self::Aqua_Geothermal_Vent => [80, 80, 80, 255],
            Self::Aqua_Horn_Coral => [220, 180, 60, 255],
            Self::Aqua_Sponge => [190, 175, 50, 255],
            Self::Aqua_Tube_Coral => [60, 120, 220, 255],
            // Light Source Blocks
            Self::Emit_Blue_Light => [64, 128, 255, 255],
            Self::Emit_Blue_Torch => [64, 128, 255, 255],
            Self::Emit_Cold_Light => [200, 230, 255, 255],
            Self::Emit_Green_Light => [64, 255, 64, 255],
            Self::Emit_Green_Torch => [64, 255, 64, 255],
            Self::Emit_Red_Light => [255, 64, 64, 255],
            Self::Emit_Red_Torch => [255, 64, 64, 255],
            Self::Emit_Warm_Light => [255, 199, 64, 255],
            // Decoration Blocks
            Self::Deco_Barrel => [133, 94, 56, 255],
            Self::Deco_Basket => [170, 130, 75, 255],
            Self::Deco_Bone => [225, 220, 200, 255],
            Self::Deco_Bookshelf => [140, 100, 60, 255],
            Self::Deco_Brick => [160, 80, 60, 255],
            Self::Deco_Fabric => [210, 210, 210, 255],
            Self::Deco_Flesh => [160, 60, 60, 255],
            Self::Deco_Glass => [200, 225, 235, 120],
            Self::Deco_Hay => [200, 180, 70, 255],
            Self::Deco_Plaster => [215, 215, 210, 255],
            Self::Deco_Slime => [100, 210, 90, 220],
            Self::Deco_Stone_Path => [120, 120, 120, 255],
            Self::Deco_Thatch => [190, 160, 80, 255],
            Self::Deco_Wax => [230, 210, 120, 255],
            Self::Deco_Wicker => [180, 140, 80, 255],
            Self::Deco_Wool => [230, 230, 230, 255],
        }
    }

    pub fn is_empty(self) -> bool {
        self == Self::Air
    }

    pub fn is_water(self) -> bool {
        matches!(self, Self::Liquid_Water | Self::WaterOccupied)
    }

    pub fn is_fluid(self) -> bool {
        self.is_water()
            || matches!(
                self,
                Self::Liquid_Acid
                    | Self::Liquid_Blood
                    | Self::Liquid_Lava
                    | Self::Liquid_Molten
                    | Self::Liquid_Ooze
                    | Self::Liquid_Sludge
                    | Self::Liquid_Tar
                    | Self::Null_Liquid
            )
    }

    pub fn max_fluid_spread(self) -> u8 {
        match self {
            Self::Liquid_Water | Self::WaterOccupied => 8,
            Self::Liquid_Acid => 5,
            Self::Liquid_Blood | Self::Null_Liquid => 4,
            Self::Liquid_Lava | Self::Liquid_Molten | Self::Liquid_Sludge | Self::Liquid_Ooze => 3,
            Self::Liquid_Tar => 2,
            _ => 0,
        }
    }

    pub fn is_collidable(self) -> bool {
        !self.is_empty()
            && !self.is_fluid()
            && !self.is_torch()
            && self != Self::Occupied
            && self != Self::WaterOccupied
    }

    pub fn is_torch(self) -> bool {
        matches!(
            self,
            Self::Emit_Blue_Torch | Self::Emit_Green_Torch | Self::Emit_Red_Torch
        )
    }

    pub fn is_basket(self) -> bool {
        self == Self::Deco_Basket
    }

    pub fn has_custom_mesh(self) -> bool {
        self.is_torch() || self.is_basket()
    }

    pub fn is_transparent(self) -> bool {
        self.is_water()
            || self == Self::Deco_Glass
            || matches!(self, Self::Frost_Ice | Self::Frost_Fragile_Ice | Self::Frost_Black_Ice)
    }

    pub fn is_leaves(self) -> bool {
        matches!(
            self,
            Self::Tree_Acacia_Leaves
                | Self::Tree_Birch_Leaves
                | Self::Tree_Cherry_Leaves
                | Self::Tree_Mahogany_Leaves
                | Self::Tree_Mangrove_Leaves
                | Self::Tree_Maple_Leaves_Red
                | Self::Tree_Maple_Leaves_Orange
                | Self::Tree_Maple_Leaves_Yellow
                | Self::Tree_Oak_Leaves
                | Self::Tree_Oak_Leaves_Lush
                | Self::Tree_Oak_Leaves_Flowering
                | Self::Tree_Palm_Leaves
                | Self::Tree_Pine_Leaves
                | Self::Tree_Willow_Leaves
                | Self::Tree_Yew_Leaves
        )
    }

    /// Solid opaque blocks that completely occlude light and adjacent faces (not leaves, transparent, fluid, or air).
    pub fn is_solid_opaque(self) -> bool {
        !self.is_empty()
            && !self.is_fluid()
            && !self.is_leaves()
            && !self.is_transparent()
            && !self.is_torch()
            && !self.is_basket()
            && self != Self::Occupied
            && self != Self::WaterOccupied
    }

    pub fn is_light(self) -> bool {
        matches!(
            self,
            Self::Emit_Blue_Light
                | Self::Emit_Blue_Torch
                | Self::Emit_Cold_Light
                | Self::Emit_Green_Light
                | Self::Emit_Green_Torch
                | Self::Emit_Red_Light
                | Self::Emit_Red_Torch
                | Self::Emit_Warm_Light
                | Self::Liquid_Lava
                | Self::Liquid_Molten
                | Self::Rock_Magma
        )
    }

    /// Returns true strictly for discrete player-placeable light fixtures (torches, lamps)
    /// that are eligible to spawn individual 3D GPU PointLight entities.
    /// Excludes bulk terrain/fluid emitters (Lava, Magma) which are rendered emissively in shaders.
    pub fn is_point_light_fixture(self) -> bool {
        matches!(
            self,
            Self::Emit_Blue_Light
                | Self::Emit_Blue_Torch
                | Self::Emit_Cold_Light
                | Self::Emit_Green_Light
                | Self::Emit_Green_Torch
                | Self::Emit_Red_Light
                | Self::Emit_Red_Torch
                | Self::Emit_Warm_Light
        )
    }

    pub fn light_color(self) -> Color {
        match self {
            Self::Emit_Warm_Light => Color::srgb(1.0, 0.82, 0.42),
            Self::Emit_Cold_Light => Color::srgb(0.80, 0.90, 1.0),
            Self::Emit_Red_Light | Self::Emit_Red_Torch => Color::srgb(1.0, 0.25, 0.25),
            Self::Emit_Green_Light | Self::Emit_Green_Torch => Color::srgb(0.25, 1.0, 0.25),
            Self::Emit_Blue_Light | Self::Emit_Blue_Torch => Color::srgb(0.25, 0.50, 1.0),
            Self::Liquid_Lava | Self::Liquid_Molten | Self::Rock_Magma => Color::srgb(1.0, 0.45, 0.15),
            _ => Color::WHITE,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Air => "Air",
            Self::Occupied => "Occupied",
            Self::WaterOccupied => "Waterlogged Occupied",
            // Testing / Debug Blocks
            Self::Null_Block => "Null Block",
            Self::Null_Liquid => "Null Liquid",
            Self::Test_Accept => "Test Accept",
            Self::Test_Debug => "Test Debug",
            Self::Test_Numbers => "Test Numbers",
            Self::Test_Fail => "Test Fail",
            Self::Test_Instance => "Test Instance",
            Self::Test_Start => "Test Start",
            // Liquid / Fluid Blocks
            Self::Liquid_Acid => "Acid",
            Self::Liquid_Blood => "Blood",
            Self::Liquid_Lava => "Lava",
            Self::Liquid_Molten => "Molten",
            Self::Liquid_Ooze => "Ooze",
            Self::Liquid_Sludge => "Sludge",
            Self::Liquid_Tar => "Tar",
            Self::Liquid_Water => "Water",
            // Frost Blocks
            Self::Frost_Black_Ice => "Black Ice",
            Self::Frost_Fragile_Ice => "Fragile Ice",
            Self::Frost_Ice => "Ice",
            Self::Frost_Packed_Ice => "Packed Ice",
            // Soil Blocks
            Self::Soil_Ash => "Ash",
            Self::Soil_Black_Sand => "Black Sand",
            Self::Soil_Clay => "Clay",
            Self::Soil_Dirt => "Dirt",
            Self::Soil_Grass => "Grass",
            Self::Soil_Gravel => "Gravel",
            Self::Soil_Moss => "Moss",
            Self::Soil_Mud => "Mud",
            Self::Soil_Mulch => "Mulch",
            Self::Soil_Packed_Dirt => "Packed Dirt",
            Self::Soil_Packed_Mud => "Packed Mud",
            Self::Soil_Packed_Peat => "Packed Peat",
            Self::Soil_Packed_Silt => "Packed Silt",
            Self::Soil_Peat => "Peat",
            Self::Soil_Peat_Grass => "Peat Grass",
            Self::Soil_Peat_Mulch => "Peat Mulch",
            Self::Soil_Red_Moss => "Red Moss",
            Self::Soil_Red_Sand => "Red Sand",
            Self::Soil_Rooted_Dirt => "Rooted Dirt",
            Self::Soil_Sand => "Sand",
            Self::Soil_Scorched_Black_Sand => "Scorched Black Sand",
            Self::Soil_Scorched_Red_Sand => "Scorched Red Sand",
            Self::Soil_Scorched_Sand => "Scorched Sand",
            Self::Soil_Scorched_White_Sand => "Scorched White Sand",
            Self::Soil_Silt => "Silt",
            Self::Soil_Silt_Grass => "Silt Grass",
            Self::Soil_Silt_Mulch => "Silt Mulch",
            Self::Soil_Snow => "Snow",
            Self::Soil_Snowy_Grass => "Snowy Grass",
            Self::Soil_Snowy_Peat => "Snowy Peat",
            Self::Soil_Snowy_Silt => "Snowy Silt",
            Self::Soil_White_Sand => "White Sand",
            // Rocky Blocks
            Self::Rock_Andesite => "Andesite",
            Self::Cobbled_Andesite => "Cobbled Andesite",
            Self::Mossy_Andesite => "Mossy Andesite",
            Self::Mossy_Cobbled_Andesite => "Mossy Cobbled Andesite",
            Self::Rock_Azurite => "Azurite",
            Self::Cobbled_Azurite => "Cobbled Azurite",
            Self::Mossy_Azurite => "Mossy Azurite",
            Self::Mossy_Cobbled_Azurite => "Mossy Cobbled Azurite",
            Self::Rock_Basalt => "Basalt",
            Self::Cobbled_Basalt => "Cobbled Basalt",
            Self::Mossy_Basalt => "Mossy Basalt",
            Self::Mossy_Cobbled_Basalt => "Mossy Cobbled Basalt",
            Self::Rock_Black_Sandstone => "Black Sandstone",
            Self::Cobbled_Black_Sandstone => "Cobbled Black Sandstone",
            Self::Mossy_Black_Sandstone => "Mossy Black Sandstone",
            Self::Mossy_Cobbled_Black_Sandstone => "Mossy Cobbled Black Sandstone",
            Self::Rock_Calcite => "Calcite",
            Self::Cobbled_Calcite => "Cobbled Calcite",
            Self::Mossy_Calcite => "Mossy Calcite",
            Self::Mossy_Cobbled_Calcite => "Mossy Cobbled Calcite",
            Self::Rock_Chalk => "Chalk",
            Self::Cobbled_Chalk => "Cobbled Chalk",
            Self::Mossy_Chalk => "Mossy Chalk",
            Self::Mossy_Cobbled_Chalk => "Mossy Cobbled Chalk",
            Self::Rock_Chert => "Chert",
            Self::Cobbled_Chert => "Cobbled Chert",
            Self::Mossy_Chert => "Mossy Chert",
            Self::Mossy_Cobbled_Chert => "Mossy Cobbled Chert",
            Self::Rock_Cinnabar => "Cinnabar",
            Self::Cobbled_Cinnabar => "Cobbled Cinnabar",
            Self::Mossy_Cinnabar => "Mossy Cinnabar",
            Self::Mossy_Cobbled_Cinnabar => "Mossy Cobbled Cinnabar",
            Self::Rock_Diorite => "Diorite",
            Self::Cobbled_Diorite => "Cobbled Diorite",
            Self::Mossy_Diorite => "Mossy Diorite",
            Self::Mossy_Cobbled_Diorite => "Mossy Cobbled Diorite",
            Self::Rock_Gabbro => "Gabbro",
            Self::Cobbled_Gabbro => "Cobbled Gabbro",
            Self::Mossy_Gabbro => "Mossy Gabbro",
            Self::Mossy_Cobbled_Gabbro => "Mossy Cobbled Gabbro",
            Self::Rock_Granite => "Granite",
            Self::Cobbled_Granite => "Cobbled Granite",
            Self::Mossy_Granite => "Mossy Granite",
            Self::Mossy_Cobbled_Granite => "Mossy Cobbled Granite",
            Self::Rock_Karst => "Karst",
            Self::Cobbled_Karst => "Cobbled Karst",
            Self::Mossy_Karst => "Mossy Karst",
            Self::Mossy_Cobbled_Karst => "Mossy Cobbled Karst",
            Self::Rock_Limestone => "Limestone",
            Self::Cobbled_Limestone => "Cobbled Limestone",
            Self::Mossy_Limestone => "Mossy Limestone",
            Self::Mossy_Cobbled_Limestone => "Mossy Cobbled Limestone",
            Self::Rock_Marl => "Marl",
            Self::Cobbled_Marl => "Cobbled Marl",
            Self::Mossy_Marl => "Mossy Marl",
            Self::Mossy_Cobbled_Marl => "Mossy Cobbled Marl",
            Self::Rock_Pitchstone => "Pitchstone",
            Self::Cobbled_Pitchstone => "Cobbled Pitchstone",
            Self::Mossy_Pitchstone => "Mossy Pitchstone",
            Self::Mossy_Cobbled_Pitchstone => "Mossy Cobbled Pitchstone",
            Self::Rock_Porphyry => "Porphyry",
            Self::Cobbled_Porphyry => "Cobbled Porphyry",
            Self::Mossy_Porphyry => "Mossy Porphyry",
            Self::Mossy_Cobbled_Porphyry => "Mossy Cobbled Porphyry",
            Self::Rock_Red_Sandstone => "Red Sandstone",
            Self::Cobbled_Red_Sandstone => "Cobbled Red Sandstone",
            Self::Mossy_Red_Sandstone => "Mossy Red Sandstone",
            Self::Mossy_Cobbled_Red_Sandstone => "Mossy Cobbled Red Sandstone",
            Self::Rock_Sandstone => "Sandstone",
            Self::Cobbled_Sandstone => "Cobbled Sandstone",
            Self::Mossy_Sandstone => "Mossy Sandstone",
            Self::Mossy_Cobbled_Sandstone => "Mossy Cobbled Sandstone",
            Self::Rock_Scoria => "Scoria",
            Self::Cobbled_Scoria => "Cobbled Scoria",
            Self::Mossy_Scoria => "Mossy Scoria",
            Self::Mossy_Cobbled_Scoria => "Mossy Cobbled Scoria",
            Self::Rock_Serpentine => "Serpentine",
            Self::Cobbled_Serpentine => "Cobbled Serpentine",
            Self::Mossy_Serpentine => "Mossy Serpentine",
            Self::Mossy_Cobbled_Serpentine => "Mossy Cobbled Serpentine",
            Self::Rock_Slate => "Slate",
            Self::Cobbled_Slate => "Cobbled Slate",
            Self::Mossy_Slate => "Mossy Slate",
            Self::Mossy_Cobbled_Slate => "Mossy Cobbled Slate",
            Self::Rock_Stone => "Stone",
            Self::Cobbled_Stone => "Cobbled Stone",
            Self::Mossy_Stone => "Mossy Stone",
            Self::Mossy_Cobbled_Stone => "Mossy Cobbled Stone",
            Self::Rock_Tuffite => "Tuffite",
            Self::Cobbled_Tuffite => "Cobbled Tuffite",
            Self::Mossy_Tuffite => "Mossy Tuffite",
            Self::Mossy_Cobbled_Tuffite => "Mossy Cobbled Tuffite",
            Self::Rock_White_Sandstone => "White Sandstone",
            Self::Cobbled_White_Sandstone => "Cobbled White Sandstone",
            Self::Mossy_White_Sandstone => "Mossy White Sandstone",
            Self::Mossy_Cobbled_White_Sandstone => "Mossy Cobbled White Sandstone",
            // Special Rocky Blocks
            Self::Rock_Alabaster => "Alabaster",
            Self::Rock_Brimstone => "Brimstone",
            Self::Rock_Cryolite => "Cryolite",
            Self::Rock_Dreadstone => "Dreadstone",
            Self::Rock_Flint => "Flint",
            Self::Rock_Magma => "Magma",
            Self::Rock_Obsidian => "Obsidian",
            Self::Rock_Terracotta => "Terracotta",
            // Wood Blocks
            Self::Tree_Acacia_Bark => "Acacia Bark",
            Self::Tree_Acacia_Log => "Acacia Log",
            Self::Tree_Acacia_Leaves => "Acacia Leaves",
            Self::Tree_Acacia_Planks => "Acacia Planks",
            Self::Tree_Birch_Bark => "Birch Bark",
            Self::Tree_Birch_Log => "Birch Log",
            Self::Tree_Birch_Leaves => "Birch Leaves",
            Self::Tree_Birch_Planks => "Birch Planks",
            Self::Tree_Cherry_Bark => "Cherry Bark",
            Self::Tree_Cherry_Log => "Cherry Log",
            Self::Tree_Cherry_Leaves => "Cherry Leaves",
            Self::Tree_Cherry_Planks => "Cherry Planks",
            Self::Tree_Mahogany_Bark => "Mahogany Bark",
            Self::Tree_Mahogany_Log => "Mahogany Log",
            Self::Tree_Mahogany_Leaves => "Mahogany Leaves",
            Self::Tree_Mahogany_Planks => "Mahogany Planks",
            Self::Tree_Mangrove_Bark => "Mangrove Bark",
            Self::Tree_Mangrove_Log => "Mangrove Log",
            Self::Tree_Mangrove_Leaves => "Mangrove Leaves",
            Self::Tree_Mangrove_Planks => "Mangrove Planks",
            Self::Tree_Mangrove_Roots => "Mangrove Roots",
            Self::Tree_Maple_Bark => "Maple Bark",
            Self::Tree_Maple_Log => "Maple Log",
            Self::Tree_Maple_Leaves_Red => "Red Maple Leaves",
            Self::Tree_Maple_Leaves_Orange => "Orange Maple Leaves",
            Self::Tree_Maple_Leaves_Yellow => "Yellow Maple Leaves",
            Self::Tree_Maple_Planks => "Maple Planks",
            Self::Tree_Oak_Bark => "Oak Bark",
            Self::Tree_Oak_Log => "Oak Log",
            Self::Tree_Oak_Leaves => "Oak Leaves",
            Self::Tree_Oak_Leaves_Lush => "Lush Oak Leaves",
            Self::Tree_Oak_Leaves_Flowering => "Flowering Oak Leaves",
            Self::Tree_Oak_Planks => "Oak Planks",
            Self::Tree_Palm_Bark => "Palm Bark",
            Self::Tree_Palm_Log => "Palm Log",
            Self::Tree_Palm_Leaves => "Palm Leaves",
            Self::Tree_Palm_Planks => "Palm Planks",
            Self::Tree_Pine_Bark => "Pine Bark",
            Self::Tree_Pine_Log => "Pine Log",
            Self::Tree_Pine_Leaves => "Pine Leaves",
            Self::Tree_Pine_Planks => "Pine Planks",
            Self::Tree_Willow_Bark => "Willow Bark",
            Self::Tree_Willow_Log => "Willow Log",
            Self::Tree_Willow_Leaves => "Willow Leaves",
            Self::Tree_Willow_Planks => "Willow Planks",
            Self::Tree_Yew_Bark => "Yew Bark",
            Self::Tree_Yew_Log => "Yew Log",
            Self::Tree_Yew_Leaves => "Yew Leaves",
            Self::Tree_Yew_Planks => "Yew Planks",
            Self::Tree_Cactus => "Cactus",
            Self::Tree_Charred_Bark => "Charred Wood Bark",
            Self::Tree_Charred_Log => "Charred Wood Log",
            Self::Tree_Charred_Planks => "Charred Wood Planks",
            Self::Tree_Dead_Bark => "Dead Wood Bark",
            Self::Tree_Dead_Log => "Dead Wood Log",
            Self::Tree_Dead_Planks => "Dead Wood Planks",
            // Aquatic Blocks
            Self::Aqua_Algae_Mat => "Algae Mat",
            Self::Aqua_Brain_Coral => "Brain Coral",
            Self::Aqua_Bubble_Coral => "Bubble Coral",
            Self::Aqua_Fire_Coral => "Fire Coral",
            Self::Aqua_Geothermal_Vent => "Geothermal Vent",
            Self::Aqua_Horn_Coral => "Horn Coral",
            Self::Aqua_Sponge => "Sponge",
            Self::Aqua_Tube_Coral => "Tube Coral",
            // Light Source Blocks
            Self::Emit_Blue_Light => "Blue Light Block",
            Self::Emit_Blue_Torch => "Blue Torch",
            Self::Emit_Cold_Light => "Cold Light Block",
            Self::Emit_Green_Light => "Green Light Block",
            Self::Emit_Green_Torch => "Green Torch",
            Self::Emit_Red_Light => "Red Light Block",
            Self::Emit_Red_Torch => "Red Torch",
            Self::Emit_Warm_Light => "Warm Light Block",
            // Decoration Blocks
            Self::Deco_Barrel => "Barrel",
            Self::Deco_Basket => "Basket",
            Self::Deco_Bone => "Bone Block",
            Self::Deco_Bookshelf => "Bookshelf",
            Self::Deco_Brick => "Brick",
            Self::Deco_Fabric => "Fabric",
            Self::Deco_Flesh => "Flesh",
            Self::Deco_Glass => "Glass",
            Self::Deco_Hay => "Hay",
            Self::Deco_Plaster => "Plaster",
            Self::Deco_Slime => "Slime",
            Self::Deco_Stone_Path => "Stone Path",
            Self::Deco_Thatch => "Thatch",
            Self::Deco_Wax => "Wax",
            Self::Deco_Wicker => "Wicker",
            Self::Deco_Wool => "Wool",
        }
    }

    /// Whether this voxel is completely unbreakable (like bedrock).
    pub fn is_unbreakable(self) -> bool {
        self == Self::Rock_Dreadstone
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torch_properties() {
        let torches = [Voxel::Emit_Red_Torch, Voxel::Emit_Green_Torch, Voxel::Emit_Blue_Torch];
        for &t in &torches {
            assert!(t.is_torch(), "Expected {t:?} to be a torch");
            assert!(t.has_custom_mesh(), "Expected {t:?} to have a custom mesh");
            assert!(!t.is_collidable(), "Expected torch {t:?} to be non-collidable (passable)");
            assert!(!t.is_solid_opaque(), "Expected torch {t:?} not to be solid opaque");
            assert!(t.is_light(), "Expected torch {t:?} to emit light");
        }
    }

    #[test]
    fn test_basket_properties() {
        let basket = Voxel::Deco_Basket;
        assert!(basket.is_basket(), "Expected Deco_Basket to be basket");
        assert!(basket.has_custom_mesh(), "Expected basket to have custom mesh");
        assert!(basket.is_collidable(), "Expected basket to be collidable");
        assert!(!basket.is_solid_opaque(), "Expected basket not to be solid opaque (hollow interior)");
    }

    #[test]
    fn test_torch_orientations_and_boxes() {
        assert_eq!(BlockShape::Torch.orientation_count(), 5);
        for orient in 0..5 {
            let (box_a, box_b) = BlockShape::Torch.local_boxes(orient);
            assert!(box_b.is_none());
            let min = box_a[0];
            let max = box_a[1];
            assert!(min.x < max.x && min.y < max.y && min.z < max.z);
        }
    }
}
