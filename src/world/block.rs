use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum ToolType {
    #[default]
    None,
    Pickaxe,
    Shovel,
    Axe,
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
}

impl Voxel {
    /// All voxels that map to a texture and are loaded into the terrain texture array.
    pub const ALL: [Voxel; 61] = [
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
        Voxel::Occupied,
        Voxel::WaterOccupied,
        Voxel::SnowyGrass,
    ];

    /// The base texture name under `assets/textures/blocks/` without extension.
    pub fn texture_name(self) -> Option<&'static str> {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => None,

            // Woods & Foliage
            Self::OakWood | Self::OakWoodLog => Some("tree_oakwood"),
            Self::BirchWood | Self::BirchWoodLog => Some("tree_birchwood"),
            Self::PineWood | Self::PineWoodLog => Some("tree_pinewood"),
            Self::OakLeaves => Some("tree_oakwood_leaves"),
            Self::BirchLeaves => Some("tree_birchwood_leaves"),
            Self::PineLeaves => Some("tree_pinewood_leaves"),

            // Natural terrain
            Self::Grass => Some("terr_grass"),
            Self::SnowyGrass => Some("terr_snow"),
            Self::Dirt => Some("terr_dirt"),
            Self::Stone => Some("rock_stone"),
            Self::Sand => Some("terr_sand"),
            Self::Clay => Some("terr_clay"),
            Self::Gravel => Some("terr_gravel"),
            Self::Moss => Some("terr_moss"),
            Self::Mud => Some("terr_mud"),
            Self::PackedDirt => Some("terr_packed_dirt"),
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
                | Self::SnowyGrass
                | Self::Water
                | Self::WaterFlowing
                | Self::WaterOccupied
                | Self::OakLeaves
                | Self::BirchLeaves
                | Self::PineLeaves
        )
    }

    /// Color tint applied to vertices for biome / atmospheric coloring.
    pub fn tint_color(self) -> [f32; 4] {
        match self {
            Self::Grass => [0.55, 0.94, 0.42, 1.0],
            Self::SnowyGrass => [0.52, 0.80, 0.70, 1.0],
            Self::Water | Self::WaterFlowing | Self::WaterOccupied => [0.35, 0.65, 0.92, 1.0],
            Self::OakLeaves => [0.60, 1.15, 0.35, 1.0],
            Self::BirchLeaves => [0.85, 1.25, 0.40, 1.0],
            Self::PineLeaves => [0.40, 0.90, 0.55, 1.0],
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
            Self::OakWood | Self::OakWoodLog => [133, 94, 56, 255],
            Self::BirchWood | Self::BirchWoodLog => [225, 222, 210, 255],
            Self::PineWood | Self::PineWoodLog => [74, 48, 28, 255],
            Self::OakLeaves => [87, 166, 46, 255],
            Self::BirchLeaves => [133, 199, 56, 255],
            Self::PineLeaves => [46, 107, 66, 255],
            Self::Grass => [110, 180, 80, 255],
            Self::SnowyGrass => [240, 245, 255, 255],
            Self::Dirt | Self::PackedDirt => [107, 66, 33, 255],
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

    #[allow(dead_code)]
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
        matches!(self, Self::OakLeaves | Self::BirchLeaves | Self::PineLeaves)
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
            Self::BirchWood => "Birch Wood",
            Self::BirchWoodLog => "Birch Log",
            Self::PineWood => "Pine Wood",
            Self::PineWoodLog => "Pine Log",
            Self::OakLeaves => "Oak Leaves",
            Self::BirchLeaves => "Birch Leaves",
            Self::PineLeaves => "Pine Leaves",
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

    /// Hardness/durability value for breaking times.
    #[allow(dead_code)]
    pub fn durability(self) -> f32 {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => 0.0,
            Self::Dreadstone => f32::INFINITY,
            Self::OakLeaves | Self::BirchLeaves | Self::PineLeaves => 0.3,
            Self::Grass
            | Self::SnowyGrass
            | Self::Dirt
            | Self::Mud
            | Self::Sand
            | Self::Gravel
            | Self::Clay
            | Self::RedSand
            | Self::Ice => 0.6,
            Self::PackedDirt
            | Self::PackedMud
            | Self::Mulch
            | Self::Moss
            | Self::Snow
            | Self::PackedIce => 0.8,
            Self::OakWood
            | Self::OakWoodLog
            | Self::BirchWood
            | Self::BirchWoodLog
            | Self::PineWood
            | Self::PineWoodLog => 1.2,
            Self::Sandstone | Self::RedSandstone => 1.5,
            Self::Calcite => 1.5,
            Self::Stone
            | Self::Cobblestone
            | Self::MossyCobblestone
            | Self::MossyStone
            | Self::Andesite
            | Self::Diorite
            | Self::Granite
            | Self::Tuff
            | Self::Blueschist
            | Self::Dripstone
            | Self::Limestone
            | Self::Ochrestone
            | Self::Serpentinite => 2.0,
            Self::Slate
            | Self::Cobbleslate
            | Self::Blackstone
            | Self::Cobbleblackstone
            | Self::Basalt
            | Self::Rhodonite => 2.5,
            Self::Cactus => 0.4,
            Self::RedMoss => 0.6,
            Self::Flint | Self::Magma => 3.0,
            Self::Light
            | Self::LightWarm
            | Self::LightCold
            | Self::LightRed
            | Self::LightGreen
            | Self::LightBlue => 0.3,
            Self::Water | Self::WaterFlowing | Self::Lava => 100.0,
        }
    }

    /// The optimal tool type required to break/harvest the block efficiently.
    #[allow(dead_code)]
    pub fn required_tool(self) -> ToolType {
        match self {
            Self::Dreadstone => ToolType::None,

            Self::Stone
            | Self::Cobblestone
            | Self::MossyCobblestone
            | Self::MossyStone
            | Self::Slate
            | Self::Cobbleslate
            | Self::Blackstone
            | Self::Cobbleblackstone
            | Self::Basalt
            | Self::Flint
            | Self::Magma
            | Self::Andesite
            | Self::Diorite
            | Self::Granite
            | Self::Tuff
            | Self::Sandstone
            | Self::RedSandstone
            | Self::Ice
            | Self::PackedIce
            | Self::Blueschist
            | Self::Calcite
            | Self::Dripstone
            | Self::Limestone
            | Self::Ochrestone
            | Self::Rhodonite
            | Self::Serpentinite => ToolType::Pickaxe,

            Self::Dirt
            | Self::Grass
            | Self::SnowyGrass
            | Self::Sand
            | Self::Gravel
            | Self::Clay
            | Self::Mud
            | Self::PackedDirt
            | Self::PackedMud
            | Self::Snow
            | Self::RedSand
            | Self::RedMoss => ToolType::Shovel,

            Self::Mulch
            | Self::OakWood
            | Self::OakWoodLog
            | Self::BirchWood
            | Self::BirchWoodLog
            | Self::PineWood
            | Self::PineWoodLog => ToolType::Axe,

            _ => ToolType::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dreadstone_is_unbreakable_bedrock() {
        assert!(Voxel::Dreadstone.is_unbreakable());
        assert!(Voxel::Dreadstone.durability().is_infinite());
        assert!(!Voxel::Stone.is_unbreakable());
        assert!(!Voxel::Dirt.is_unbreakable());
    }

    #[test]
    fn leaf_solid_opaque_and_culling_invariants() {
        assert!(!Voxel::OakLeaves.is_solid_opaque());
        assert!(!Voxel::BirchLeaves.is_solid_opaque());
        assert!(!Voxel::PineLeaves.is_solid_opaque());
        assert!(Voxel::OakLeaves.is_leaves());
        assert!(Voxel::BirchLeaves.is_leaves());
        assert!(Voxel::PineLeaves.is_leaves());

        assert!(Voxel::OakWoodLog.is_solid_opaque());
        assert!(Voxel::BirchWoodLog.is_solid_opaque());
        assert!(Voxel::PineWoodLog.is_solid_opaque());
        assert!(Voxel::Stone.is_solid_opaque());
        assert!(!Voxel::Air.is_solid_opaque());
        assert!(!Voxel::Water.is_solid_opaque());
    }
}
