use bevy::prelude::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum BlockShape {
    #[default]
    Full = 0,
    Slab = 1,
    Stair = 2,
    Column = 3,
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
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Full => "FULL",
            Self::Slab => "SLAB",
            Self::Stair => "STAIR",
            Self::Column => "COLUMN",
        }
    }

    pub const fn orientation_count(self) -> u8 {
        match self {
            Self::Full => 1,
            Self::Slab => 6,
            Self::Stair => 8,
            Self::Column => 6,
        }
    }

    pub fn orientation_name(self, orientation: u8) -> &'static str {
        match self {
            Self::Full => "Standard",
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
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::Slab,
            Self::Slab => Self::Stair,
            Self::Stair => Self::Column,
            Self::Column => Self::Full,
        }
    }

    /// Returns the local AABB bounding boxes ([min, max] in 0.0..=1.0 coordinates) for this shape and orientation.
    /// Full, Slab, and Column have 1 box; Stair has 2 boxes (base slab + step).
    pub fn local_boxes(self, orientation: u8) -> ([Vec3; 2], Option<[Vec3; 2]>) {
        match self {
            Self::Full => ([Vec3::ZERO, Vec3::ONE], None),
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
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum Voxel {
    #[default]
    Air = 0,

    // Natural terrain & soil
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Water = 5,
    Light = 6, // Legacy alias for LightWarm
    Occupied = 7,
    WaterFlowing = 8,
    WaterOccupied = 9,

    // Rock & Masonry
    Cobblestone = 10,
    MossyCobblestone = 11,
    MossyStone = 12,
    Slate = 13,
    Cobbleslate = 14,
    Blackstone = 15,
    Cobbleblackstone = 16,
    Flint = 17,
    Magma = 18,

    // Soils, Organics & Fine Sediment
    Clay = 19,
    Gravel = 20,
    Moss = 21,
    Mud = 22,
    PackedDirt = 23,
    PackedMud = 24,
    Mulch = 25,
    Snow = 26,

    // Fluids
    Lava = 27,

    // RGB & Temperature Light Emitters
    LightWarm = 28,
    LightCold = 29,
    LightRed = 30,
    LightGreen = 31,
    LightBlue = 32,

    // Additional Rocks & Minerals
    Andesite = 33,
    Diorite = 34,
    Granite = 35,
    Dreadstone = 36,
    Tuff = 37,
    Sandstone = 38,
    RedSandstone = 39,

    // Additional Sands & Ices
    RedSand = 40,
    Ice = 41,
    PackedIce = 42,

    // Additional Geologies & Organics
    Blueschist = 43,
    Calcite = 44,
    Dripstone = 45,
    Limestone = 46,
    Ochrestone = 47,
    Rhodonite = 48,
    Serpentinite = 49,
    RedMoss = 50,

    // Woods & Foliage
    OakWood = 51,
    OakWoodLog = 52,
    BirchWood = 53,
    BirchWoodLog = 54,
    PineWood = 55,
    PineWoodLog = 56,
    OakLeaves = 57,
    BirchLeaves = 58,
    PineLeaves = 59,

    // Volcanic & Desert Flora
    Basalt = 60,
    Cactus = 61,

    // Snowy biomes
    SnowyGrass = 62,

    // Terracotta & Rainwood
    Terracotta = 63,
    RainwoodWood = 64,
    RainwoodWoodLog = 65,
    RainwoodLeaves = 66,

    // Soils
    RootedDirt = 67,

    // Planks
    OakPlanks = 68,
    BirchPlanks = 69,
    PinePlanks = 70,
    RainwoodPlanks = 71,
}

impl Voxel {
    /// All voxels that map to a texture and are loaded into the terrain texture array.
    pub const ALL: [Voxel; 70] = [
        Voxel::Grass,
        Voxel::Dirt,
        Voxel::Stone,
        Voxel::Sand,
        Voxel::Water,
        Voxel::WaterFlowing,
        Voxel::LightWarm,
        Voxel::LightCold,
        Voxel::LightRed,
        Voxel::LightGreen,
        Voxel::LightBlue,
        Voxel::Cobblestone,
        Voxel::MossyCobblestone,
        Voxel::MossyStone,
        Voxel::Slate,
        Voxel::Cobbleslate,
        Voxel::Blackstone,
        Voxel::Cobbleblackstone,
        Voxel::Flint,
        Voxel::Clay,
        Voxel::Gravel,
        Voxel::Moss,
        Voxel::Mud,
        Voxel::PackedDirt,
        Voxel::PackedMud,
        Voxel::Mulch,
        Voxel::Snow,
        Voxel::Magma,
        Voxel::Lava,
        Voxel::Andesite,
        Voxel::Diorite,
        Voxel::Granite,
        Voxel::Dreadstone,
        Voxel::Tuff,
        Voxel::Sandstone,
        Voxel::RedSandstone,
        Voxel::RedSand,
        Voxel::Ice,
        Voxel::PackedIce,
        Voxel::Blueschist,
        Voxel::Calcite,
        Voxel::Dripstone,
        Voxel::Limestone,
        Voxel::Ochrestone,
        Voxel::Rhodonite,
        Voxel::Serpentinite,
        Voxel::RedMoss,
        Voxel::Basalt,
        Voxel::Cactus,
        Voxel::OakWood,
        Voxel::OakWoodLog,
        Voxel::BirchWood,
        Voxel::BirchWoodLog,
        Voxel::PineWood,
        Voxel::PineWoodLog,
        Voxel::OakLeaves,
        Voxel::BirchLeaves,
        Voxel::PineLeaves,
        Voxel::RainwoodWood,
        Voxel::RainwoodWoodLog,
        Voxel::RainwoodLeaves,
        Voxel::Terracotta,
        Voxel::Occupied,
        Voxel::WaterOccupied,
        Voxel::SnowyGrass,
        Voxel::RootedDirt,
        Voxel::OakPlanks,
        Voxel::BirchPlanks,
        Voxel::PinePlanks,
        Voxel::RainwoodPlanks,
    ];

    /// The base texture name under `assets/textures/blocks/` without extension.
    pub fn texture_name(self) -> Option<&'static str> {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => None,

            // Woods & Foliage
            Self::OakWood | Self::OakWoodLog => Some("tree_oakwood"),
            Self::BirchWood | Self::BirchWoodLog => Some("tree_birchwood"),
            Self::PineWood | Self::PineWoodLog => Some("tree_pinewood"),
            Self::RainwoodWood | Self::RainwoodWoodLog => Some("tree_rainwood"),
            Self::OakPlanks => Some("tree_oakwood_planks"),
            Self::BirchPlanks => Some("tree_birchwood_planks"),
            Self::PinePlanks => Some("tree_pinewood_planks"),
            Self::RainwoodPlanks => Some("tree_rainwood_planks"),
            Self::OakLeaves => Some("tree_oakwood_leaves"),
            Self::BirchLeaves => Some("tree_birchwood_leaves"),
            Self::PineLeaves => Some("tree_pinewood_leaves"),
            Self::RainwoodLeaves => Some("tree_rainwood_leaves"),

            // Natural terrain
            Self::Grass => Some("terr_grass"),
            Self::SnowyGrass => Some("terr_snow"),
            Self::Dirt => Some("terr_dirt"),
            Self::Stone => Some("rock_stone"),
            Self::Sand => Some("terr_sand"),
            Self::Clay => Some("terr_clay"),
            Self::Terracotta => Some("terr_terracotta"),
            Self::Gravel => Some("terr_gravel"),
            Self::Moss => Some("terr_moss"),
            Self::Mud => Some("terr_mud"),
            Self::PackedDirt => Some("terr_packed_dirt"),
            Self::RootedDirt => Some("terr_rooted_dirt"),
            Self::PackedMud => Some("terr_packed_mud"),
            Self::Mulch => Some("terr_mulch"),
            Self::Snow => Some("terr_snow"),
            Self::RedSand => Some("terr_red_sand"),
            Self::Ice => Some("terr_ice"),
            Self::PackedIce => Some("terr_packed_ice"),

            // Rock & Minerals
            Self::Cobblestone => Some("rock_cobblestone"),
            Self::MossyCobblestone => Some("rock_mossy_cobblestone"),
            Self::MossyStone => Some("rock_mossy_stone"),
            Self::Slate => Some("rock_slate"),
            Self::Cobbleslate => Some("rock_cobbleslate"),
            Self::Blackstone => Some("rock_blackstone"),
            Self::Cobbleblackstone => Some("rock_cobbleblackstone"),
            Self::Flint => Some("rock_flint"),
            Self::Magma => Some("rock_magma"),
            Self::Andesite => Some("rock_andesite"),
            Self::Diorite => Some("rock_diorite"),
            Self::Granite => Some("rock_granite"),
            Self::Dreadstone => Some("rock_dreadstone"),
            Self::Tuff => Some("rock_tuff"),
            Self::Sandstone => Some("rock_sandstone"),
            Self::RedSandstone => Some("rock_red_sandstone"),
            Self::Blueschist => Some("rock_blueschist"),
            Self::Calcite => Some("rock_calcite"),
            Self::Dripstone => Some("rock_dripstone"),
            Self::Limestone => Some("rock_limestone"),
            Self::Ochrestone => Some("rock_ochrestone"),
            Self::Rhodonite => Some("rock_rhodonite"),
            Self::Serpentinite => Some("rock_serpentinite"),
            Self::RedMoss => Some("terr_red_moss"),
            Self::Basalt => Some("rock_basalt"),
            Self::Cactus => Some("tree_cactus_side"),

            // Fluids
            Self::Water => Some("liqd_water_still"),
            Self::WaterFlowing => Some("liqd_water_flow"),
            Self::Lava => Some("liqd_lava_still"),

            // Lights
            Self::Light | Self::LightWarm => Some("emit_warm_light"),
            Self::LightCold => Some("emit_cold_light"),
            Self::LightRed => Some("emit_red_light"),
            Self::LightGreen => Some("emit_green_light"),
            Self::LightBlue => Some("emit_blue_light"),
        }
    }

    /// Optional texture override for the side faces (+X, -X, +Z, -Z).
    /// If None, falls back to `texture_name()`.
    pub fn side_texture_name(self) -> Option<&'static str> {
        match self {
            Self::Basalt => Some("rock_basalt_side"),
            Self::Cactus => Some("tree_cactus_side"),
            Self::Mulch => Some("terr_mulch_side"),
            Self::Grass => Some("terr_grass_side"),
            Self::SnowyGrass => Some("terr_snowy_grass_side"),
            _ => None,
        }
    }

    /// Optional texture override for the top face (+Y).
    /// If None, falls back to `texture_name()`.
    pub fn top_texture_name(self) -> Option<&'static str> {
        match self {
            Self::OakWoodLog => Some("tree_oakwood_log"),
            Self::BirchWoodLog => Some("tree_birchwood_log"),
            Self::PineWoodLog => Some("tree_pinewood_log"),
            Self::RainwoodWoodLog => Some("tree_rainwood_log"),
            Self::Cactus => Some("tree_cactus_top"),
            Self::SnowyGrass => Some("terr_snow"),
            _ => None,
        }
    }

    /// Optional texture override for the bottom face (-Y).
    /// If None, falls back to `texture_name()`.
    pub fn bottom_texture_name(self) -> Option<&'static str> {
        match self {
            Self::OakWoodLog => Some("tree_oakwood_log"),
            Self::BirchWoodLog => Some("tree_birchwood_log"),
            Self::PineWoodLog => Some("tree_pinewood_log"),
            Self::RainwoodWoodLog => Some("tree_rainwood_log"),
            Self::Cactus => Some("tree_cactus_bottom"),
            Self::Mulch => Some("terr_dirt"),
            Self::Grass => Some("terr_dirt"),
            Self::SnowyGrass => Some("terr_dirt"),
            _ => None,
        }
    }

    /// Whether this voxel receives custom biome or foliage tinting.
    pub fn is_tinted(self) -> bool {
        matches!(
            self,
            Self::Grass
                | Self::Water
                | Self::WaterFlowing
                | Self::WaterOccupied
                | Self::OakLeaves
                | Self::BirchLeaves
                | Self::PineLeaves
                | Self::RainwoodLeaves
        )
    }

    /// Color tint applied to vertices for biome / atmospheric coloring.
    pub fn tint_color(self) -> [f32; 4] {
        match self {
            Self::Grass => [0.55, 0.94, 0.42, 1.0],
            Self::Water | Self::WaterFlowing | Self::WaterOccupied => [0.35, 0.65, 0.92, 1.0],
            Self::OakLeaves => [0.60, 1.15, 0.35, 1.0],
            Self::BirchLeaves => [0.85, 1.25, 0.40, 1.0],
            Self::PineLeaves => [0.40, 0.90, 0.55, 1.0],
            Self::RainwoodLeaves => [0.45, 1.10, 0.65, 1.0],
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
            Self::OakWood | Self::OakWoodLog | Self::OakPlanks => [133, 94, 56, 255],
            Self::BirchWood | Self::BirchWoodLog | Self::BirchPlanks => [225, 222, 210, 255],
            Self::PineWood | Self::PineWoodLog | Self::PinePlanks => [74, 48, 28, 255],
            Self::RainwoodWood | Self::RainwoodWoodLog | Self::RainwoodPlanks => [118, 76, 52, 255],
            Self::OakLeaves => [87, 166, 46, 255],
            Self::BirchLeaves => [133, 199, 56, 255],
            Self::PineLeaves => [46, 107, 66, 255],
            Self::RainwoodLeaves => [60, 135, 55, 255],
            Self::Terracotta => [165, 95, 65, 255],
            Self::Grass => [110, 180, 80, 255],
            Self::SnowyGrass => [240, 245, 255, 255],
            Self::Dirt | Self::PackedDirt | Self::RootedDirt => [107, 66, 33, 255],
            Self::Stone => [122, 128, 133, 255],
            Self::Cobblestone => [110, 110, 115, 255],
            Self::MossyCobblestone | Self::MossyStone => [95, 120, 85, 255],
            Self::Slate | Self::Cobbleslate => [80, 85, 95, 255],
            Self::Blackstone | Self::Cobbleblackstone => [45, 42, 48, 255],
            Self::Flint => [55, 55, 60, 255],
            Self::Sand => [209, 194, 128, 255],
            Self::Clay => [150, 155, 175, 255],
            Self::Gravel => [130, 125, 125, 255],
            Self::Moss => [80, 130, 60, 255],
            Self::Mud | Self::PackedMud => [85, 60, 45, 255],
            Self::Mulch => [90, 55, 35, 255],
            Self::Snow => [240, 245, 255, 255],
            Self::Magma => [180, 70, 30, 255],
            Self::Andesite => [136, 136, 136, 255],
            Self::Diorite => [180, 180, 185, 255],
            Self::Granite => [150, 105, 90, 255],
            Self::Dreadstone => [35, 30, 40, 255],
            Self::Tuff => [105, 108, 100, 255],
            Self::Sandstone => [215, 205, 150, 255],
            Self::RedSandstone => [185, 95, 45, 255],
            Self::RedSand => [190, 100, 50, 255],
            Self::Ice => [140, 185, 235, 220],
            Self::PackedIce => [160, 200, 245, 255],
            Self::Blueschist => [75, 95, 115, 255],
            Self::Calcite => [220, 220, 225, 255],
            Self::Dripstone => [135, 105, 90, 255],
            Self::Limestone => [195, 185, 165, 255],
            Self::Ochrestone => [185, 135, 55, 255],
            Self::Rhodonite => [195, 110, 135, 255],
            Self::Serpentinite => [70, 115, 85, 255],
            Self::RedMoss => [175, 45, 45, 255],
            Self::Basalt => [75, 75, 80, 255],
            Self::Cactus => [85, 135, 45, 255],
            Self::Water | Self::WaterFlowing => [60, 140, 220, 255],
            Self::Lava => [230, 100, 20, 255],
            Self::Light | Self::LightWarm => [255, 199, 64, 255],
            Self::LightCold => [200, 230, 255, 255],
            Self::LightRed => [255, 64, 64, 255],
            Self::LightGreen => [64, 255, 64, 255],
            Self::LightBlue => [64, 128, 255, 255],
        }
    }

    pub fn is_empty(self) -> bool {
        self == Self::Air
    }

    pub fn is_water(self) -> bool {
        matches!(self, Self::Water | Self::WaterFlowing | Self::WaterOccupied)
    }

    pub fn is_fluid(self) -> bool {
        self.is_water() || self == Self::Lava
    }

    pub fn is_collidable(self) -> bool {
        !matches!(
            self,
            Self::Air | Self::Water | Self::WaterFlowing | Self::Lava
        )
    }

    pub fn is_transparent(self) -> bool {
        self.is_water()
    }

    pub fn is_leaves(self) -> bool {
        matches!(
            self,
            Self::OakLeaves | Self::BirchLeaves | Self::PineLeaves | Self::RainwoodLeaves
        )
    }

    /// Solid opaque blocks that completely occlude light and adjacent faces (not leaves, water, or air).
    pub fn is_solid_opaque(self) -> bool {
        !self.is_empty()
            && !self.is_water()
            && !self.is_leaves()
            && self != Self::Occupied
            && self != Self::WaterOccupied
    }

    pub fn is_light(self) -> bool {
        matches!(
            self,
            Self::Light
                | Self::LightWarm
                | Self::LightCold
                | Self::LightRed
                | Self::LightGreen
                | Self::LightBlue
        )
    }

    pub fn light_color(self) -> Color {
        match self {
            Self::Light | Self::LightWarm => Color::srgb(1.0, 0.82, 0.42),
            Self::LightCold => Color::srgb(0.80, 0.90, 1.0),
            Self::LightRed => Color::srgb(1.0, 0.25, 0.25),
            Self::LightGreen => Color::srgb(0.25, 1.0, 0.25),
            Self::LightBlue => Color::srgb(0.25, 0.50, 1.0),
            _ => Color::WHITE,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Air => "Air",
            Self::Grass => "Grass",
            Self::SnowyGrass => "Snowy Grass",
            Self::Dirt => "Dirt",
            Self::PackedDirt => "Packed Dirt",
            Self::RootedDirt => "Rooted Dirt",
            Self::Stone => "Stone",
            Self::Cobblestone => "Cobblestone",
            Self::MossyCobblestone => "Mossy Cobblestone",
            Self::MossyStone => "Mossy Stone",
            Self::Slate => "Slate",
            Self::Cobbleslate => "Cobbleslate",
            Self::Blackstone => "Blackstone",
            Self::Cobbleblackstone => "Cobbleblackstone",
            Self::Flint => "Flint",
            Self::Sand => "Sand",
            Self::Clay => "Clay",
            Self::Terracotta => "Terracotta",
            Self::Gravel => "Gravel",
            Self::Moss => "Moss",
            Self::Mud => "Mud",
            Self::PackedMud => "Packed Mud",
            Self::Mulch => "Mulch",
            Self::Snow => "Snow",
            Self::Magma => "Magma",
            Self::Andesite => "Andesite",
            Self::Diorite => "Diorite",
            Self::Granite => "Granite",
            Self::Dreadstone => "Dreadstone",
            Self::Tuff => "Tuff",
            Self::Sandstone => "Sandstone",
            Self::RedSandstone => "Red Sandstone",
            Self::RedSand => "Red Sand",
            Self::Ice => "Ice",
            Self::PackedIce => "Packed Ice",
            Self::Blueschist => "Blueschist",
            Self::Calcite => "Calcite",
            Self::Dripstone => "Dripstone",
            Self::Limestone => "Limestone",
            Self::Ochrestone => "Ochrestone",
            Self::Rhodonite => "Rhodonite",
            Self::Serpentinite => "Serpentinite",
            Self::RedMoss => "Red Moss",
            Self::Basalt => "Basalt",
            Self::Cactus => "Cactus",
            Self::OakWood => "Oak Wood",
            Self::OakWoodLog => "Oak Log",
            Self::OakPlanks => "Oak Planks",
            Self::BirchWood => "Birch Wood",
            Self::BirchWoodLog => "Birch Log",
            Self::BirchPlanks => "Birch Planks",
            Self::PineWood => "Pine Wood",
            Self::PineWoodLog => "Pine Log",
            Self::PinePlanks => "Pine Planks",
            Self::RainwoodWood => "Rainwood Wood",
            Self::RainwoodWoodLog => "Rainwood Log",
            Self::RainwoodPlanks => "Rainwood Planks",
            Self::OakLeaves => "Oak Leaves",
            Self::BirchLeaves => "Birch Leaves",
            Self::PineLeaves => "Pine Leaves",
            Self::RainwoodLeaves => "Rainwood Leaves",
            Self::Water => "Water",
            Self::WaterFlowing => "Flowing Water",
            Self::Lava => "Lava",
            Self::Light | Self::LightWarm => "Warm Light",
            Self::LightCold => "Cold Light",
            Self::LightRed => "Red Light",
            Self::LightGreen => "Green Light",
            Self::LightBlue => "Blue Light",
            Self::Occupied => "Occupied",
            Self::WaterOccupied => "Waterlogged Occupied",
        }
    }

    /// Whether this voxel is completely unbreakable (like bedrock).
    pub fn is_unbreakable(self) -> bool {
        self == Self::Dreadstone
    }
}
