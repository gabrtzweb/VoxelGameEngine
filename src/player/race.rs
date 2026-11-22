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
    Orc,
    Dragonborn,
    Tiefling,
}

impl CharacterRace {
    pub const ALL: [CharacterRace; 6] = [
        CharacterRace::Human,
        CharacterRace::Elf,
        CharacterRace::Dwarf,
        CharacterRace::Orc,
        CharacterRace::Dragonborn,
        CharacterRace::Tiefling,
    ];

    /// The 3 classic playable core races (each with 3 playable subraces).
    pub const PLAYABLE: [CharacterRace; 3] = [
        CharacterRace::Human,
        CharacterRace::Elf,
        CharacterRace::Dwarf,
    ];

    pub fn subraces(&self) -> [CharacterSubrace; 3] {
        match self {
            Self::Human => [
                CharacterSubrace::Nordling,
                CharacterSubrace::Sunlander,
                CharacterSubrace::CoastalHuman,
            ],
            Self::Elf => [
                CharacterSubrace::WoodElf,
                CharacterSubrace::DarkElf,
                CharacterSubrace::HighElf,
            ],
            Self::Dwarf => [
                CharacterSubrace::MountainDwarf,
                CharacterSubrace::DeepDwarf,
                CharacterSubrace::HillDwarf,
            ],
            _ => [
                CharacterSubrace::Nordling,
                CharacterSubrace::Sunlander,
                CharacterSubrace::CoastalHuman,
            ],
        }
    }

    pub fn default_subrace(&self) -> CharacterSubrace {
        self.subraces()[0]
    }

    pub fn desc_key(&self) -> &'static str {
        match self {
            Self::Human => "menu.race.human.desc",
            Self::Elf => "menu.race.elf.desc",
            Self::Dwarf => "menu.race.dwarf.desc",
            _ => "menu.race.human.desc",
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Elf => "elf",
            Self::Dwarf => "dwarf",
            Self::Orc => "orc",
            Self::Dragonborn => "dragonborn",
            Self::Tiefling => "tiefling",
        }
    }

    pub fn loc_key(&self) -> &'static str {
        match self {
            Self::Human => "menu.race.human",
            Self::Elf => "menu.race.elf",
            Self::Dwarf => "menu.race.dwarf",
            Self::Orc => "menu.race.orc",
            Self::Dragonborn => "menu.race.dragonborn",
            Self::Tiefling => "menu.race.tiefling",
        }
    }

    pub fn default_name(&self) -> &'static str {
        match self {
            Self::Human => "Humano",
            Self::Elf => "Elfo",
            Self::Dwarf => "Anão",
            Self::Orc => "Orc",
            Self::Dragonborn => "Draconato",
            Self::Tiefling => "Tiefling",
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
        }
    }

    /// Primary skin tone for head and face
    pub fn skin_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.92, 0.76, 0.65),
            Self::Elf => Color::srgb(0.96, 0.90, 0.82),
            Self::Dwarf => Color::srgb(0.86, 0.68, 0.52),
            Self::Orc => Color::srgb(0.36, 0.58, 0.28),
            Self::Dragonborn => Color::srgb(0.85, 0.20, 0.15),
            Self::Tiefling => Color::srgb(0.70, 0.22, 0.34),
        }
    }

    /// Torso / chest clothing or scales color
    pub fn torso_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.22, 0.45, 0.78),
            Self::Elf => Color::srgb(0.18, 0.55, 0.32),
            Self::Dwarf => Color::srgb(0.48, 0.45, 0.42),
            Self::Orc => Color::srgb(0.34, 0.24, 0.18),
            Self::Dragonborn => Color::srgb(0.70, 0.14, 0.12),
            Self::Tiefling => Color::srgb(0.20, 0.16, 0.25),
        }
    }

    /// Arms color
    pub fn arms_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.92, 0.76, 0.65),
            Self::Elf => Color::srgb(0.96, 0.90, 0.82),
            Self::Dwarf => Color::srgb(0.86, 0.68, 0.52),
            Self::Orc => Color::srgb(0.36, 0.58, 0.28),
            Self::Dragonborn => Color::srgb(0.85, 0.20, 0.15),
            Self::Tiefling => Color::srgb(0.70, 0.22, 0.34),
        }
    }

    /// Legs color
    pub fn legs_color(&self) -> Color {
        match self {
            Self::Human => Color::srgb(0.24, 0.24, 0.30),
            Self::Elf => Color::srgb(0.40, 0.32, 0.22),
            Self::Dwarf => Color::srgb(0.32, 0.26, 0.22),
            Self::Orc => Color::srgb(0.22, 0.20, 0.20),
            Self::Dragonborn => Color::srgb(0.48, 0.12, 0.12),
            Self::Tiefling => Color::srgb(0.16, 0.14, 0.20),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaceAttributes {
    pub max_health: u32,
    pub max_stamina: u32,
    pub defense: u32,
    pub move_speed_multiplier: f32,
    pub swim_speed_multiplier: f32,
    pub jump_height_blocks: f32,
    pub footstep_noise_multiplier: f32,
    pub mining_speed_multiplier: f32,
    pub reach_bonus_blocks: f32,
    pub cave_vision_bonus: f32,
    pub fall_damage_reduction: f32,
    pub knockback_resistance: f32,
    pub cold_resistance: f32,
    pub heat_resistance: f32,
    pub xp_gain_multiplier: f32,
    pub carry_slots: u32,
}

impl RaceAttributes {
    /// Computes vertical launch velocity required to clear `jump_height_blocks` under `gravity`.
    pub fn jump_speed(&self, gravity: f32) -> f32 {
        (2.0 * gravity * (self.jump_height_blocks + 0.08)).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum CharacterSubrace {
    #[default]
    Nordling,
    Sunlander,
    CoastalHuman,
    WoodElf,
    DarkElf,
    HighElf,
    MountainDwarf,
    DeepDwarf,
    HillDwarf,
}

impl CharacterSubrace {
    pub const ALL: [CharacterSubrace; 9] = [
        Self::Nordling,
        Self::Sunlander,
        Self::CoastalHuman,
        Self::WoodElf,
        Self::DarkElf,
        Self::HighElf,
        Self::MountainDwarf,
        Self::DeepDwarf,
        Self::HillDwarf,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::Nordling => "nordling",
            Self::Sunlander => "sunlander",
            Self::CoastalHuman => "coastal_human",
            Self::WoodElf => "wood_elf",
            Self::DarkElf => "dark_elf",
            Self::HighElf => "high_elf",
            Self::MountainDwarf => "mountain_dwarf",
            Self::DeepDwarf => "deep_dwarf",
            Self::HillDwarf => "hill_dwarf",
        }
    }

    pub fn parent_race(&self) -> CharacterRace {
        match self {
            Self::Nordling | Self::Sunlander | Self::CoastalHuman => CharacterRace::Human,
            Self::WoodElf | Self::DarkElf | Self::HighElf => CharacterRace::Elf,
            Self::MountainDwarf | Self::DeepDwarf | Self::HillDwarf => CharacterRace::Dwarf,
        }
    }

    /// Underlying rig archetype used by `src/player/model.rs`.
    pub fn rig_race(&self) -> CharacterRace {
        self.parent_race()
    }

    pub fn loc_key(&self) -> &'static str {
        match self {
            Self::Nordling => "menu.subrace.nordling",
            Self::Sunlander => "menu.subrace.sunlander",
            Self::CoastalHuman => "menu.subrace.coastal_human",
            Self::WoodElf => "menu.subrace.wood_elf",
            Self::DarkElf => "menu.subrace.dark_elf",
            Self::HighElf => "menu.subrace.high_elf",
            Self::MountainDwarf => "menu.subrace.mountain_dwarf",
            Self::DeepDwarf => "menu.subrace.deep_dwarf",
            Self::HillDwarf => "menu.subrace.hill_dwarf",
        }
    }

    pub fn desc_key(&self) -> &'static str {
        match self {
            Self::Nordling => "menu.subrace.nordling.desc",
            Self::Sunlander => "menu.subrace.sunlander.desc",
            Self::CoastalHuman => "menu.subrace.coastal_human.desc",
            Self::WoodElf => "menu.subrace.wood_elf.desc",
            Self::DarkElf => "menu.subrace.dark_elf.desc",
            Self::HighElf => "menu.subrace.high_elf.desc",
            Self::MountainDwarf => "menu.subrace.mountain_dwarf.desc",
            Self::DeepDwarf => "menu.subrace.deep_dwarf.desc",
            Self::HillDwarf => "menu.subrace.hill_dwarf.desc",
        }
    }

    pub fn trait_keys(&self) -> [(&'static str, &'static str); 2] {
        match self {
            Self::Nordling => [
                ("menu.subrace.nordling.trait1.name", "menu.subrace.nordling.trait1.desc"),
                ("menu.subrace.nordling.trait2.name", "menu.subrace.nordling.trait2.desc"),
            ],
            Self::Sunlander => [
                ("menu.subrace.sunlander.trait1.name", "menu.subrace.sunlander.trait1.desc"),
                ("menu.subrace.sunlander.trait2.name", "menu.subrace.sunlander.trait2.desc"),
            ],
            Self::CoastalHuman => [
                ("menu.subrace.coastal_human.trait1.name", "menu.subrace.coastal_human.trait1.desc"),
                ("menu.subrace.coastal_human.trait2.name", "menu.subrace.coastal_human.trait2.desc"),
            ],
            Self::WoodElf => [
                ("menu.subrace.wood_elf.trait1.name", "menu.subrace.wood_elf.trait1.desc"),
                ("menu.subrace.wood_elf.trait2.name", "menu.subrace.wood_elf.trait2.desc"),
            ],
            Self::DarkElf => [
                ("menu.subrace.dark_elf.trait1.name", "menu.subrace.dark_elf.trait1.desc"),
                ("menu.subrace.dark_elf.trait2.name", "menu.subrace.dark_elf.trait2.desc"),
            ],
            Self::HighElf => [
                ("menu.subrace.high_elf.trait1.name", "menu.subrace.high_elf.trait1.desc"),
                ("menu.subrace.high_elf.trait2.name", "menu.subrace.high_elf.trait2.desc"),
            ],
            Self::MountainDwarf => [
                ("menu.subrace.mountain_dwarf.trait1.name", "menu.subrace.mountain_dwarf.trait1.desc"),
                ("menu.subrace.mountain_dwarf.trait2.name", "menu.subrace.mountain_dwarf.trait2.desc"),
            ],
            Self::DeepDwarf => [
                ("menu.subrace.deep_dwarf.trait1.name", "menu.subrace.deep_dwarf.trait1.desc"),
                ("menu.subrace.deep_dwarf.trait2.name", "menu.subrace.deep_dwarf.trait2.desc"),
            ],
            Self::HillDwarf => [
                ("menu.subrace.hill_dwarf.trait1.name", "menu.subrace.hill_dwarf.trait1.desc"),
                ("menu.subrace.hill_dwarf.trait2.name", "menu.subrace.hill_dwarf.trait2.desc"),
            ],
        }
    }

    pub fn default_name(&self) -> &'static str {
        match self {
            Self::Nordling => "Northern Human",
            Self::Sunlander => "Southern Human",
            Self::CoastalHuman => "Coastal Human",
            Self::WoodElf => "Wood Elf",
            Self::DarkElf => "Dark Elf",
            Self::HighElf => "High Elf",
            Self::MountainDwarf => "Mountain Dwarf",
            Self::DeepDwarf => "Deep Dwarf",
            Self::HillDwarf => "Hill Dwarf",
        }
    }

    pub fn display_name(&self, loc: Option<&Localization>) -> String {
        loc.map_or_else(
            || self.default_name().to_string(),
            |l| l.t(self.loc_key()).to_string(),
        )
    }

    pub fn icon_asset_path(&self) -> String {
        format!("textures/interfaces/icons/subrace_{}.png", self.id())
    }

    pub fn attributes(&self) -> RaceAttributes {
        match self {
            Self::Nordling => RaceAttributes {
                max_health: 110,
                max_stamina: 110,
                defense: 6,
                move_speed_multiplier: 1.00,
                swim_speed_multiplier: 1.00,
                jump_height_blocks: 1.25,
                footstep_noise_multiplier: 1.00,
                mining_speed_multiplier: 1.00,
                reach_bonus_blocks: 0.00,
                cave_vision_bonus: 0.00,
                fall_damage_reduction: 0.05,
                knockback_resistance: 0.15,
                cold_resistance: 0.40,
                heat_resistance: 0.00,
                xp_gain_multiplier: 1.05,
                carry_slots: 29,
            },
            Self::Sunlander => RaceAttributes {
                max_health: 100,
                max_stamina: 105,
                defense: 5,
                move_speed_multiplier: 1.04,
                swim_speed_multiplier: 1.02,
                jump_height_blocks: 1.30,
                footstep_noise_multiplier: 0.90,
                mining_speed_multiplier: 1.00,
                reach_bonus_blocks: 0.00,
                cave_vision_bonus: 0.00,
                fall_damage_reduction: 0.10,
                knockback_resistance: 0.00,
                cold_resistance: 0.00,
                heat_resistance: 0.40,
                xp_gain_multiplier: 1.12,
                carry_slots: 29,
            },
            Self::CoastalHuman => RaceAttributes {
                max_health: 105,
                max_stamina: 110,
                defense: 5,
                move_speed_multiplier: 1.02,
                swim_speed_multiplier: 1.30,
                jump_height_blocks: 1.28,
                footstep_noise_multiplier: 0.95,
                mining_speed_multiplier: 1.00,
                reach_bonus_blocks: 0.00,
                cave_vision_bonus: 0.05,
                fall_damage_reduction: 0.10,
                knockback_resistance: 0.05,
                cold_resistance: 0.15,
                heat_resistance: 0.15,
                xp_gain_multiplier: 1.08,
                carry_slots: 30,
            },
            Self::WoodElf => RaceAttributes {
                max_health: 90,
                max_stamina: 115,
                defense: 4,
                move_speed_multiplier: 1.10,
                swim_speed_multiplier: 1.05,
                jump_height_blocks: 2.05,
                footstep_noise_multiplier: 0.55,
                mining_speed_multiplier: 0.95,
                reach_bonus_blocks: 0.00,
                cave_vision_bonus: 0.15,
                fall_damage_reduction: 0.35,
                knockback_resistance: 0.00,
                cold_resistance: 0.10,
                heat_resistance: 0.05,
                xp_gain_multiplier: 1.00,
                carry_slots: 27,
            },
            Self::DarkElf => RaceAttributes {
                max_health: 90,
                max_stamina: 110,
                defense: 5,
                move_speed_multiplier: 1.07,
                swim_speed_multiplier: 1.00,
                jump_height_blocks: 2.00,
                footstep_noise_multiplier: 0.65,
                mining_speed_multiplier: 1.05,
                reach_bonus_blocks: 0.00,
                cave_vision_bonus: 0.85,
                fall_damage_reduction: 0.30,
                knockback_resistance: 0.00,
                cold_resistance: 0.05,
                heat_resistance: 0.15,
                xp_gain_multiplier: 1.00,
                carry_slots: 27,
            },
            Self::HighElf => RaceAttributes {
                max_health: 95,
                max_stamina: 115,
                defense: 4,
                move_speed_multiplier: 1.06,
                swim_speed_multiplier: 1.02,
                jump_height_blocks: 2.00,
                footstep_noise_multiplier: 0.75,
                mining_speed_multiplier: 1.00,
                reach_bonus_blocks: 0.25,
                cave_vision_bonus: 0.25,
                fall_damage_reduction: 0.30,
                knockback_resistance: 0.00,
                cold_resistance: 0.15,
                heat_resistance: 0.15,
                xp_gain_multiplier: 1.10,
                carry_slots: 28,
            },
            Self::MountainDwarf => RaceAttributes {
                max_health: 125,
                max_stamina: 100,
                defense: 9,
                move_speed_multiplier: 0.94,
                swim_speed_multiplier: 0.85,
                jump_height_blocks: 1.10,
                footstep_noise_multiplier: 1.15,
                mining_speed_multiplier: 1.15,
                reach_bonus_blocks: 0.25,
                cave_vision_bonus: 0.45,
                fall_damage_reduction: 0.00,
                knockback_resistance: 0.35,
                cold_resistance: 0.25,
                heat_resistance: 0.10,
                xp_gain_multiplier: 1.00,
                carry_slots: 31,
            },
            Self::DeepDwarf => RaceAttributes {
                max_health: 120,
                max_stamina: 105,
                defense: 8,
                move_speed_multiplier: 0.95,
                swim_speed_multiplier: 0.88,
                jump_height_blocks: 1.10,
                footstep_noise_multiplier: 1.05,
                mining_speed_multiplier: 1.25,
                reach_bonus_blocks: 0.50,
                cave_vision_bonus: 0.65,
                fall_damage_reduction: 0.00,
                knockback_resistance: 0.25,
                cold_resistance: 0.05,
                heat_resistance: 0.45,
                xp_gain_multiplier: 1.00,
                carry_slots: 32,
            },
            Self::HillDwarf => RaceAttributes {
                max_health: 120,
                max_stamina: 110,
                defense: 7,
                move_speed_multiplier: 0.97,
                swim_speed_multiplier: 0.92,
                jump_height_blocks: 1.15,
                footstep_noise_multiplier: 1.00,
                mining_speed_multiplier: 1.12,
                reach_bonus_blocks: 0.25,
                cave_vision_bonus: 0.35,
                fall_damage_reduction: 0.05,
                knockback_resistance: 0.20,
                cold_resistance: 0.20,
                heat_resistance: 0.20,
                xp_gain_multiplier: 1.06,
                carry_slots: 33,
            },
        }
    }

    pub fn skin_color(&self) -> Color {
        match self {
            Self::Nordling => Color::srgb(0.94, 0.80, 0.70),
            Self::Sunlander => Color::srgb(0.69, 0.48, 0.32),
            Self::CoastalHuman => Color::srgb(0.84, 0.63, 0.45),
            Self::WoodElf => Color::srgb(0.96, 0.90, 0.82),
            Self::DarkElf => Color::srgb(0.28, 0.24, 0.35),
            Self::HighElf => Color::srgb(0.93, 0.94, 0.98),
            Self::MountainDwarf => Color::srgb(0.86, 0.68, 0.52),
            Self::DeepDwarf => Color::srgb(0.52, 0.54, 0.58),
            Self::HillDwarf => Color::srgb(0.86, 0.66, 0.47),
        }
    }

    pub fn torso_color(&self) -> Color {
        match self {
            Self::Nordling => Color::srgb(0.22, 0.42, 0.68),
            Self::Sunlander => Color::srgb(0.70, 0.28, 0.18),
            Self::CoastalHuman => Color::srgb(0.14, 0.36, 0.52),
            Self::WoodElf => Color::srgb(0.18, 0.55, 0.32),
            Self::DarkElf => Color::srgb(0.20, 0.18, 0.24),
            Self::HighElf => Color::srgb(0.20, 0.34, 0.64),
            Self::MountainDwarf => Color::srgb(0.48, 0.45, 0.42),
            Self::DeepDwarf => Color::srgb(0.26, 0.28, 0.34),
            Self::HillDwarf => Color::srgb(0.24, 0.44, 0.24),
        }
    }

    pub fn arms_color(&self) -> Color {
        self.skin_color()
    }

    pub fn legs_color(&self) -> Color {
        match self {
            Self::Nordling => Color::srgb(0.22, 0.24, 0.30),
            Self::Sunlander => Color::srgb(0.32, 0.24, 0.18),
            Self::CoastalHuman => Color::srgb(0.18, 0.24, 0.32),
            Self::WoodElf => Color::srgb(0.40, 0.32, 0.22),
            Self::DarkElf => Color::srgb(0.14, 0.14, 0.18),
            Self::HighElf => Color::srgb(0.26, 0.28, 0.42),
            Self::MountainDwarf => Color::srgb(0.32, 0.26, 0.22),
            Self::DeepDwarf => Color::srgb(0.18, 0.19, 0.22),
            Self::HillDwarf => Color::srgb(0.34, 0.26, 0.18),
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

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct SelectedSubrace(pub CharacterSubrace);

impl Default for SelectedSubrace {
    fn default() -> Self {
        Self(CharacterSubrace::Nordling)
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct CharacterReady(pub bool);
