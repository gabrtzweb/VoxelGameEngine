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
}

impl Voxel {
    /// All voxels that map to a texture and are loaded into the terrain texture array.
    pub const ALL: [Voxel; 41] = [
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
        Voxel::Occupied,
        Voxel::WaterOccupied,
    ];

    /// The base texture name under `assets/textures/blocks/` without extension.
    pub fn texture_name(self) -> Option<&'static str> {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => None,

            // Natural terrain
            Self::Grass => Some("terr_grass"),
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

    /// Color tint applied to vertices for biome / atmospheric coloring.
    pub fn tint_color(self) -> [f32; 4] {
        match self {
            Self::Grass => [0.58, 0.90, 0.44, 1.0],
            Self::Water | Self::WaterFlowing | Self::WaterOccupied => [0.40, 0.80, 1.0, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn tint_color_at(self, _world_voxel: IVec3) -> [f32; 4] {
        self.tint_color()
    }

    /// Fallback 1x1 solid RGBA pixel if the texture file is not found on disk.
    pub fn fallback_color(self) -> [u8; 4] {
        match self {
            Self::Air | Self::Occupied | Self::WaterOccupied => [0, 0, 0, 0],
            Self::Grass => [110, 180, 80, 255],
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
            Self::Grass
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
            Self::Sandstone | Self::RedSandstone => 1.5,
            Self::Stone
            | Self::Cobblestone
            | Self::MossyCobblestone
            | Self::MossyStone
            | Self::Andesite
            | Self::Diorite
            | Self::Granite
            | Self::Tuff => 2.0,
            Self::Slate | Self::Cobbleslate | Self::Blackstone | Self::Cobbleblackstone => 2.5,
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
            | Self::Flint
            | Self::Magma
            | Self::Andesite
            | Self::Diorite
            | Self::Granite
            | Self::Tuff
            | Self::Sandstone
            | Self::RedSandstone
            | Self::Ice
            | Self::PackedIce => ToolType::Pickaxe,

            Self::Dirt
            | Self::Grass
            | Self::Sand
            | Self::Gravel
            | Self::Clay
            | Self::Mud
            | Self::PackedDirt
            | Self::PackedMud
            | Self::Snow
            | Self::RedSand => ToolType::Shovel,

            Self::Mulch => ToolType::Axe,

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
}
