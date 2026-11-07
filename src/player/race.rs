use bevy::prelude::*;

use crate::core::Localization;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaceProportions {
    /// Overall model root scale (width, height, depth)
    pub root_scale: Vec3,
    /// Head local scale
    pub head_scale: Vec3,
    /// Torso local scale
    pub torso_scale: Vec3,
    /// Arm local scale
    pub arm_scale: Vec3,
    /// Leg local scale
    pub leg_scale: Vec3,
    /// Horizontal shoulder offset multiplier (1.0 = 6.0 * PIXEL_SCALE)
    pub shoulder_spread: f32,
    /// Horizontal hip offset multiplier (1.0 = 2.0 * PIXEL_SCALE)
    pub hip_spread: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum CharacterRace {
    #[default]
    Human,
    Elf,
    Dwarf,
    Halfling,
    Orc,
    Dragonborn,
    Tiefling,
    Drow,
}

impl CharacterRace {
    pub const ALL: [CharacterRace; 8] = [
        CharacterRace::Human,
        CharacterRace::Elf,
        CharacterRace::Dwarf,
        CharacterRace::Halfling,
        CharacterRace::Orc,
        CharacterRace::Dragonborn,
        CharacterRace::Tiefling,
        CharacterRace::Drow,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Elf => "elf",
            Self::Dwarf => "dwarf",
            Self::Halfling => "halfling",
            Self::Orc => "orc",
            Self::Dragonborn => "dragonborn",
            Self::Tiefling => "tiefling",
            Self::Drow => "drow",
        }
    }

    pub fn loc_key(&self) -> &'static str {
        match self {
            Self::Human => "menu.race.human",
            Self::Elf => "menu.race.elf",
            Self::Dwarf => "menu.race.dwarf",
            Self::Halfling => "menu.race.halfling",
            Self::Orc => "menu.race.orc",
            Self::Dragonborn => "menu.race.dragonborn",
            Self::Tiefling => "menu.race.tiefling",
            Self::Drow => "menu.race.drow",
        }
    }

    pub fn default_name(&self) -> &'static str {
        match self {
            Self::Human => "Humano",
            Self::Elf => "Elfo",
            Self::Dwarf => "Anão",
            Self::Halfling => "Halfling",
            Self::Orc => "Orc",
            Self::Dragonborn => "Draconato",
            Self::Tiefling => "Tiefling",
            Self::Drow => "Drow",
        }
    }

    pub fn display_name(&self, loc: Option<&Localization>) -> String {
        loc.map_or_else(
            || self.default_name().to_string(),
            |l| l.t(self.loc_key()).to_string(),
        )
    }

    pub fn icon_asset_path(&self) -> String {
        format!("textures/interfaces/icons/race_{}.png", self.id())
    }

    /// Physical proportions and silhouette scaling per race.
    pub fn proportions(&self) -> RaceProportions {
        match self {
            Self::Human => RaceProportions {
                root_scale: Vec3::ONE,
                head_scale: Vec3::ONE,
                torso_scale: Vec3::ONE,
                arm_scale: Vec3::ONE,
                leg_scale: Vec3::ONE,
                shoulder_spread: 1.0,
                hip_spread: 1.0,
            },
            Self::Elf => RaceProportions {
                // Tall, slender, lithe silhouette
                root_scale: Vec3::new(0.92, 1.08, 0.92),
                head_scale: Vec3::new(0.95, 1.00, 0.95),
                torso_scale: Vec3::new(0.90, 1.04, 0.90),
                arm_scale: Vec3::new(0.84, 1.06, 0.84),
                leg_scale: Vec3::new(0.88, 1.06, 0.88),
                shoulder_spread: 0.92,
                hip_spread: 0.92,
            },
            Self::Dwarf => RaceProportions {
                // Short, stocky, broad-shouldered stout silhouette
                root_scale: Vec3::new(1.16, 0.78, 1.16),
                head_scale: Vec3::new(1.06, 1.08, 1.06),
                torso_scale: Vec3::new(1.24, 0.96, 1.28),
                arm_scale: Vec3::new(1.22, 0.94, 1.22),
                leg_scale: Vec3::new(1.24, 0.88, 1.24),
                shoulder_spread: 1.18,
                hip_spread: 1.15,
            },
            Self::Halfling => RaceProportions {
                // Slightly shorter and more agile than a Dwarf, without being miniature
                root_scale: Vec3::new(0.86, 0.74, 0.86),
                head_scale: Vec3::new(1.06, 1.06, 1.06),
                torso_scale: Vec3::new(0.96, 0.96, 0.98),
                arm_scale: Vec3::new(0.94, 0.95, 0.94),
                leg_scale: Vec3::new(0.96, 0.92, 0.96),
                shoulder_spread: 0.96,
                hip_spread: 0.96,
            },
            Self::Orc => RaceProportions {
                // Massive, hulking, broad-shouldered muscular brute
                root_scale: Vec3::new(1.18, 1.08, 1.18),
                head_scale: Vec3::new(1.06, 1.04, 1.08),
                torso_scale: Vec3::new(1.28, 1.06, 1.26),
                arm_scale: Vec3::new(1.30, 1.06, 1.30),
                leg_scale: Vec3::new(1.18, 1.02, 1.18),
                shoulder_spread: 1.22,
                hip_spread: 1.12,
            },
            Self::Dragonborn => RaceProportions {
                // Tall, powerful draconic build
                root_scale: Vec3::new(1.10, 1.08, 1.10),
                head_scale: Vec3::new(1.04, 1.04, 1.08),
                torso_scale: Vec3::new(1.14, 1.04, 1.16),
                arm_scale: Vec3::new(1.12, 1.04, 1.12),
                leg_scale: Vec3::new(1.12, 1.04, 1.12),
                shoulder_spread: 1.10,
                hip_spread: 1.06,
            },
            Self::Tiefling => RaceProportions {
                // Lithe arcane humanoid build
                root_scale: Vec3::new(0.98, 1.03, 0.98),
                head_scale: Vec3::new(1.00, 1.00, 1.00),
                torso_scale: Vec3::new(0.96, 1.02, 0.96),
                arm_scale: Vec3::new(0.94, 1.02, 0.94),
                leg_scale: Vec3::new(0.96, 1.02, 0.96),
                shoulder_spread: 0.96,
                hip_spread: 0.96,
            },
            Self::Drow => RaceProportions {
                // Sleek, agile dark-elven silhouette
                root_scale: Vec3::new(0.92, 1.06, 0.92),
                head_scale: Vec3::new(0.95, 1.00, 0.95),
                torso_scale: Vec3::new(0.92, 1.03, 0.92),
                arm_scale: Vec3::new(0.86, 1.05, 0.86),
                leg_scale: Vec3::new(0.88, 1.05, 0.88),
                shoulder_spread: 0.93,
                hip_spread: 0.93,
            },
        }
    }

    /// Primary skin tone for head and face
    pub fn skin_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.92, 0.76, 0.65),      // Fair peach human skin
            Self::Elf => Color::srgb(0.96, 0.90, 0.82),        // Luminous pale ivory elf skin
            Self::Dwarf => Color::srgb(0.86, 0.68, 0.52),      // Weathered ruddy tan dwarf skin
            Self::Halfling => Color::srgb(0.94, 0.75, 0.60),   // Warm golden peach halfling skin
            Self::Orc => Color::srgb(0.36, 0.58, 0.28),        // Earthy robust green orc skin
            Self::Dragonborn => Color::srgb(0.85, 0.20, 0.15), // Vivid draconic red scales
            Self::Tiefling => Color::srgb(0.70, 0.22, 0.34),   // Infernal crimson-maroon skin
            Self::Drow => Color::srgb(0.28, 0.24, 0.35),       // Dusky obsidian ash purple drow skin
        }
    }

    /// Torso / chest clothing or scales color
    pub fn torso_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.22, 0.45, 0.78),      // Adventurer blue tunic
            Self::Elf => Color::srgb(0.18, 0.55, 0.32),        // Emerald forest green tunic
            Self::Dwarf => Color::srgb(0.48, 0.45, 0.42),      // Heavy forged iron mail
            Self::Halfling => Color::srgb(0.75, 0.52, 0.25),   // Rustic amber vest
            Self::Orc => Color::srgb(0.34, 0.24, 0.18),        // Rugged dark battle leather
            Self::Dragonborn => Color::srgb(0.70, 0.14, 0.12), // Deep crimson scaly plate
            Self::Tiefling => Color::srgb(0.20, 0.16, 0.25),   // Dark obsidian robe
            Self::Drow => Color::srgb(0.20, 0.18, 0.24),       // Silver-sheened underdark silk
        }
    }

    /// Arms color
    pub fn arms_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.92, 0.76, 0.65),
            Self::Elf => Color::srgb(0.96, 0.90, 0.82),
            Self::Dwarf => Color::srgb(0.86, 0.68, 0.52),
            Self::Halfling => Color::srgb(0.94, 0.75, 0.60),
            Self::Orc => Color::srgb(0.36, 0.58, 0.28),
            Self::Dragonborn => Color::srgb(0.85, 0.20, 0.15),
            Self::Tiefling => Color::srgb(0.70, 0.22, 0.34),
            Self::Drow => Color::srgb(0.28, 0.24, 0.35),
        }
    }

    /// Legs color
    pub fn legs_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.24, 0.24, 0.30),      // Dark slate pants
            Self::Elf => Color::srgb(0.40, 0.32, 0.22),        // Woodland brown breeches
            Self::Dwarf => Color::srgb(0.32, 0.26, 0.22),      // Heavy studded leather boots
            Self::Halfling => Color::srgb(0.35, 0.45, 0.30),   // Earthy moss trousers
            Self::Orc => Color::srgb(0.22, 0.20, 0.20),        // Dark durable trousers
            Self::Dragonborn => Color::srgb(0.48, 0.12, 0.12), // Charcoal red armored greaves
            Self::Tiefling => Color::srgb(0.16, 0.14, 0.20),   // Charcoal mystic leggings
            Self::Drow => Color::srgb(0.14, 0.14, 0.18),       // Deep shadow black leggings
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct SelectedRace(pub CharacterRace);

impl Default for SelectedRace {
    fn default() -> Self {
        Self(CharacterRace::Human)
    }
}
