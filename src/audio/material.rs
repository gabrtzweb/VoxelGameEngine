use bevy::prelude::*;
use crate::world::Voxel;

/// Categorized material acoustic identity for tactile footstep and block interaction audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SoundMaterial {
    Grass,
    Dirt,
    Stone,
    Wood,
    Sand,
    Gravel,
    Snow,
    Leaves,
    Mud,
    Slate,
    Glass,
    Water,
}

impl SoundMaterial {
    /// Maps any voxel type to its corresponding acoustic material profile.
    pub fn from_voxel(voxel: Voxel) -> Self {
        if voxel.is_water() || voxel.is_fluid() {
            return Self::Water;
        }

        if voxel.is_leaves_const() || matches!(
            voxel,
            Voxel::BullKelp
                | Voxel::Delesseria
                | Voxel::Kelp
                | Voxel::RedAlgae
                | Voxel::Sargassum
                | Voxel::Seaweed
                | Voxel::Seagrass
                | Voxel::BrainCoralFan
                | Voxel::BubbleCoralFan
                | Voxel::DeadCoralFan
                | Voxel::DyingCoralFan
                | Voxel::FireCoralFan
                | Voxel::HeartCoralFan
                | Voxel::HornCoralFan
                | Voxel::LeafCoralFan
                | Voxel::ReedCoralFan
                | Voxel::SpikeCoralFan
                | Voxel::StaghornCoralFan
                | Voxel::SunCoralFan
                | Voxel::ThornCoralFan
                | Voxel::TubeCoralFan
                | Voxel::WaxCoralFan
                | Voxel::ZigzagCoralFan
        ) {
            return Self::Leaves;
        }

        if voxel.has_grass_turf() || matches!(
            voxel,
            Voxel::MossAmber
                | Voxel::MossCave
                | Voxel::MossCrimson
                | Voxel::MossForest
                | Voxel::FungiAmanita
                | Voxel::FungiAmanitaCap
                | Voxel::FungiChanterelle
                | Voxel::FungiChanterelleCap
                | Voxel::FungiIndigo
                | Voxel::FungiIndigoCap
                | Voxel::FungiInside
                | Voxel::FungiMycena
                | Voxel::FungiMycenaCap
                | Voxel::FungiPorcini
                | Voxel::FungiPorciniCap
                | Voxel::FungiRussula
                | Voxel::FungiRussulaCap
                | Voxel::FungiShiitake
                | Voxel::FungiShiitakeCap
                | Voxel::FungiStem
                | Voxel::FungiVerdigris
                | Voxel::FungiVerdigrisCap
        ) {
            return Self::Grass;
        }

        let name = voxel.label();

        if name.contains("Log")
            || name.contains("Planks")
            || name.contains("Wood")
            || name.contains("Bamboo")
            || name.contains("Dam")
        {
            return Self::Wood;
        }

        if name.contains("Glass") || name.contains("Crystal") {
            return Self::Glass;
        }

        if name.contains("Snow") || name.contains("Ice") || name.contains("Frost") {
            return Self::Snow;
        }

        if name.contains("Sand") {
            return Self::Sand;
        }

        if name.contains("Gravel") {
            return Self::Gravel;
        }

        if name.contains("Mud") {
            return Self::Mud;
        }

        if name.contains("Slate") || name.contains("Shingles") {
            return Self::Slate;
        }

        if name.contains("Dirt")
            || name.contains("Coarse")
            || name.contains("Rooted")
            || name.contains("Mulch")
            || name.contains("Tilled")
            || name.contains("Loam")
            || name.contains("Peat")
            || name.contains("Silt")
            || name.contains("Laterite")
            || name.contains("Caliche")
            || name.contains("Clay")
            || name.contains("Ash")
            || name.contains("PathDirty")
        {
            return Self::Dirt;
        }

        Self::Stone
    }

    /// Block destruction sound asset paths.
    pub fn break_sounds(&self) -> &'static [&'static str] {
        match self {
            Self::Grass => &[
                "sounds/blocks/grass/break1.ogg",
                "sounds/blocks/grass/break2.ogg",
                "sounds/blocks/grass/break3.ogg",
                "sounds/blocks/grass/break4.ogg",
            ],
            Self::Dirt => &[
                "sounds/blocks/rooted_dirt/break1.ogg",
                "sounds/blocks/rooted_dirt/break2.ogg",
                "sounds/blocks/rooted_dirt/break3.ogg",
                "sounds/blocks/rooted_dirt/break4.ogg",
            ],
            Self::Stone => &[
                "sounds/blocks/stone/break1.ogg",
                "sounds/blocks/stone/break2.ogg",
                "sounds/blocks/stone/break3.ogg",
                "sounds/blocks/stone/break4.ogg",
            ],
            Self::Wood => &[
                "sounds/blocks/wood/break1.ogg",
                "sounds/blocks/wood/break2.ogg",
                "sounds/blocks/wood/break3.ogg",
                "sounds/blocks/wood/break4.ogg",
            ],
            Self::Sand => &[
                "sounds/blocks/sand/break1.ogg",
                "sounds/blocks/sand/break2.ogg",
                "sounds/blocks/sand/break3.ogg",
                "sounds/blocks/sand/break4.ogg",
            ],
            Self::Gravel => &[
                "sounds/blocks/gravel/break1.ogg",
                "sounds/blocks/gravel/break2.ogg",
                "sounds/blocks/gravel/break3.ogg",
                "sounds/blocks/gravel/break4.ogg",
            ],
            Self::Snow => &[
                "sounds/blocks/snow/break1.ogg",
                "sounds/blocks/snow/break2.ogg",
                "sounds/blocks/snow/break3.ogg",
                "sounds/blocks/snow/break4.ogg",
            ],
            Self::Leaves => &[
                "sounds/footsteps/leaves/leaves_through1.ogg",
                "sounds/footsteps/leaves/leaves_through2.ogg",
                "sounds/footsteps/leaves/leaves_through3.ogg",
                "sounds/footsteps/leaves/leaves_through4.ogg",
            ],
            Self::Mud => &[
                "sounds/blocks/rooted_dirt/break1.ogg",
                "sounds/blocks/rooted_dirt/break2.ogg",
                "sounds/blocks/rooted_dirt/break3.ogg",
                "sounds/blocks/rooted_dirt/break4.ogg",
            ],
            Self::Slate => &[
                "sounds/blocks/slate/break1.ogg",
                "sounds/blocks/slate/break2.ogg",
                "sounds/blocks/slate/break3.ogg",
                "sounds/blocks/slate/break4.ogg",
            ],
            Self::Glass => &[
                "sounds/blocks/glass/hit1.ogg",
                "sounds/blocks/glass/hit2.ogg",
                "sounds/blocks/glass/hit3.ogg",
                "sounds/blocks/glass/hit4.ogg",
            ],
            Self::Water => &[
                "sounds/liquids/splash.ogg",
                "sounds/liquids/splash2.ogg",
            ],
        }
    }

    /// Block placement sound asset paths.
    pub fn place_sounds(&self) -> &'static [&'static str] {
        match self {
            Self::Slate => &[
                "sounds/blocks/slate/place1.ogg",
                "sounds/blocks/slate/place2.ogg",
                "sounds/blocks/slate/place3.ogg",
                "sounds/blocks/slate/place4.ogg",
                "sounds/blocks/slate/place5.ogg",
                "sounds/blocks/slate/place6.ogg",
            ],
            _ => self.hit_sounds(),
        }
    }

    /// Block impact / hit sound asset paths.
    pub fn hit_sounds(&self) -> &'static [&'static str] {
        match self {
            Self::Grass => &[
                "sounds/blocks/grass/hit1.ogg",
                "sounds/blocks/grass/hit2.ogg",
                "sounds/blocks/grass/hit3.ogg",
                "sounds/blocks/grass/hit4.ogg",
            ],
            Self::Dirt => &[
                "sounds/blocks/rooted_dirt/hit1.ogg",
                "sounds/blocks/rooted_dirt/hit2.ogg",
                "sounds/blocks/rooted_dirt/hit3.ogg",
                "sounds/blocks/rooted_dirt/hit4.ogg",
            ],
            Self::Stone => &[
                "sounds/blocks/stone/hit1.ogg",
                "sounds/blocks/stone/hit2.ogg",
                "sounds/blocks/stone/hit3.ogg",
                "sounds/blocks/stone/hit4.ogg",
                "sounds/blocks/stone/hit5.ogg",
                "sounds/blocks/stone/hit6.ogg",
                "sounds/blocks/stone/hit7.ogg",
                "sounds/blocks/stone/hit8.ogg",
            ],
            Self::Wood => &[
                "sounds/blocks/wood/hit1.ogg",
                "sounds/blocks/wood/hit2.ogg",
                "sounds/blocks/wood/hit3.ogg",
                "sounds/blocks/wood/hit4.ogg",
            ],
            Self::Sand => &[
                "sounds/blocks/sand/hit1.ogg",
                "sounds/blocks/sand/hit2.ogg",
                "sounds/blocks/sand/hit3.ogg",
                "sounds/blocks/sand/hit4.ogg",
            ],
            Self::Gravel => &[
                "sounds/blocks/gravel/break1.ogg",
                "sounds/blocks/gravel/break2.ogg",
                "sounds/blocks/gravel/break3.ogg",
                "sounds/blocks/gravel/break4.ogg",
            ],
            Self::Snow => &[
                "sounds/blocks/snow/hit1.ogg",
                "sounds/blocks/snow/hit2.ogg",
                "sounds/blocks/snow/hit3.ogg",
                "sounds/blocks/snow/hit4.ogg",
            ],
            Self::Leaves => &[
                "sounds/footsteps/leaves/leaves_through1.ogg",
                "sounds/footsteps/leaves/leaves_through2.ogg",
                "sounds/footsteps/leaves/leaves_through3.ogg",
            ],
            Self::Mud => &[
                "sounds/footsteps/mud/mud_walk1.ogg",
                "sounds/footsteps/mud/mud_walk2.ogg",
                "sounds/footsteps/mud/mud_walk3.ogg",
            ],
            Self::Slate => &[
                "sounds/blocks/slate/hit1.ogg",
                "sounds/blocks/slate/hit2.ogg",
                "sounds/blocks/slate/hit3.ogg",
                "sounds/blocks/slate/hit4.ogg",
            ],
            Self::Glass => &[
                "sounds/blocks/glass/hit1.ogg",
                "sounds/blocks/glass/hit2.ogg",
                "sounds/blocks/glass/hit3.ogg",
                "sounds/blocks/glass/hit4.ogg",
            ],
            Self::Water => &[
                "sounds/liquids/splash.ogg",
                "sounds/liquids/splash2.ogg",
            ],
        }
    }

    /// Footstep walking sound asset paths.
    pub fn walk_sounds(&self) -> &'static [&'static str] {
        match self {
            Self::Grass => &[
                "sounds/footsteps/grass/grass_walk1.ogg",
                "sounds/footsteps/grass/grass_walk2.ogg",
                "sounds/footsteps/grass/grass_walk3.ogg",
                "sounds/footsteps/grass/grass_walk4.ogg",
                "sounds/footsteps/grass/grass_walk5.ogg",
                "sounds/footsteps/grass/grass_walk6.ogg",
                "sounds/footsteps/grass/grass_walk7.ogg",
                "sounds/footsteps/grass/grass_walk8.ogg",
                "sounds/footsteps/grass/grass_walk9.ogg",
                "sounds/footsteps/grass/grass_walk10.ogg",
            ],
            Self::Dirt => &[
                "sounds/footsteps/dirt/dirt_walk1.ogg",
                "sounds/footsteps/dirt/dirt_walk2.ogg",
                "sounds/footsteps/dirt/dirt_walk3.ogg",
                "sounds/footsteps/dirt/dirt_walk4.ogg",
                "sounds/footsteps/dirt/dirt_walk5.ogg",
                "sounds/footsteps/dirt/dirt_walk6.ogg",
                "sounds/footsteps/dirt/dirt_walk7.ogg",
                "sounds/footsteps/dirt/dirt_walk8.ogg",
                "sounds/footsteps/dirt/dirt_walk9.ogg",
                "sounds/footsteps/dirt/dirt_walk10.ogg",
                "sounds/footsteps/dirt/dirt_walk11.ogg",
            ],
            Self::Stone => &[
                "sounds/footsteps/stone/stone_walk1.ogg",
                "sounds/footsteps/stone/stone_walk2.ogg",
                "sounds/footsteps/stone/stone_walk3.ogg",
                "sounds/footsteps/stone/stone_walk4.ogg",
                "sounds/footsteps/stone/stone_walk5.ogg",
                "sounds/footsteps/stone/stone_walk6.ogg",
                "sounds/footsteps/stone/stone_walk7.ogg",
                "sounds/footsteps/stone/stone_walk8.ogg",
                "sounds/footsteps/stone/stone_walk9.ogg",
                "sounds/footsteps/stone/stone_walk10.ogg",
                "sounds/footsteps/stone/stone_walk11.ogg",
            ],
            Self::Wood => &[
                "sounds/footsteps/wood/wood_walk1.ogg",
                "sounds/footsteps/wood/wood_walk2.ogg",
                "sounds/footsteps/wood/wood_walk3.ogg",
                "sounds/footsteps/wood/wood_walk4.ogg",
                "sounds/footsteps/wood/wood_walk5.ogg",
                "sounds/footsteps/wood/wood_walk6.ogg",
                "sounds/footsteps/wood/wood_walk7.ogg",
                "sounds/footsteps/wood/wood_walk8.ogg",
                "sounds/footsteps/wood/wood_walk9.ogg",
                "sounds/footsteps/wood/wood_walk10.ogg",
                "sounds/footsteps/wood/wood_walk11.ogg",
            ],
            Self::Sand => &[
                "sounds/footsteps/sand/sand_walk1.ogg",
                "sounds/footsteps/sand/sand_walk2.ogg",
                "sounds/footsteps/sand/sand_walk3.ogg",
                "sounds/footsteps/sand/sand_walk4.ogg",
                "sounds/footsteps/sand/sand_walk5.ogg",
                "sounds/footsteps/sand/sand_walk6.ogg",
                "sounds/footsteps/sand/sand_walk7.ogg",
                "sounds/footsteps/sand/sand_walk8.ogg",
                "sounds/footsteps/sand/sand_walk9.ogg",
                "sounds/footsteps/sand/sand_walk10.ogg",
                "sounds/footsteps/sand/sand_walk11.ogg",
            ],
            Self::Gravel => &[
                "sounds/footsteps/gravel/gravel_walk1.ogg",
                "sounds/footsteps/gravel/gravel_walk2.ogg",
                "sounds/footsteps/gravel/gravel_walk3.ogg",
                "sounds/footsteps/gravel/gravel_walk4.ogg",
                "sounds/footsteps/gravel/gravel_walk5.ogg",
                "sounds/footsteps/gravel/gravel_walk6.ogg",
                "sounds/footsteps/gravel/gravel_walk7.ogg",
                "sounds/footsteps/gravel/gravel_walk8.ogg",
                "sounds/footsteps/gravel/gravel_walk9.ogg",
                "sounds/footsteps/gravel/gravel_walk10.ogg",
                "sounds/footsteps/gravel/gravel_walk11.ogg",
            ],
            Self::Snow => &[
                "sounds/footsteps/snow/snow_walk1.ogg",
                "sounds/footsteps/snow/snow_walk2.ogg",
                "sounds/footsteps/snow/snow_walk3.ogg",
                "sounds/footsteps/snow/snow_walk4.ogg",
                "sounds/footsteps/snow/snow_walk5.ogg",
                "sounds/footsteps/snow/snow_walk6.ogg",
                "sounds/footsteps/snow/snow_walk7.ogg",
                "sounds/footsteps/snow/snow_walk8.ogg",
                "sounds/footsteps/snow/snow_walk9.ogg",
                "sounds/footsteps/snow/snow_walk10.ogg",
                "sounds/footsteps/snow/snow_walk11.ogg",
            ],
            Self::Leaves => &[
                "sounds/footsteps/leaves/leaves_through1.ogg",
                "sounds/footsteps/leaves/leaves_through2.ogg",
                "sounds/footsteps/leaves/leaves_through3.ogg",
                "sounds/footsteps/leaves/leaves_through4.ogg",
                "sounds/footsteps/leaves/leaves_through5.ogg",
                "sounds/footsteps/leaves/leaves_through6.ogg",
                "sounds/footsteps/leaves/leaves_through7.ogg",
            ],
            Self::Mud => &[
                "sounds/footsteps/mud/mud_walk1.ogg",
                "sounds/footsteps/mud/mud_walk2.ogg",
                "sounds/footsteps/mud/mud_walk3.ogg",
                "sounds/footsteps/mud/mud_walk4.ogg",
                "sounds/footsteps/mud/mud_walk5.ogg",
                "sounds/footsteps/mud/mud_walk6.ogg",
            ],
            Self::Slate => &[
                "sounds/footsteps/slate/step1.ogg",
                "sounds/footsteps/slate/step2.ogg",
                "sounds/footsteps/slate/step3.ogg",
                "sounds/footsteps/slate/step4.ogg",
                "sounds/footsteps/slate/step5.ogg",
                "sounds/footsteps/slate/step6.ogg",
            ],
            Self::Glass => &[
                "sounds/blocks/glass/hit1.ogg",
                "sounds/blocks/glass/hit2.ogg",
                "sounds/blocks/glass/hit3.ogg",
                "sounds/blocks/glass/hit4.ogg",
            ],
            Self::Water => &[
                "sounds/footsteps/water/water_through1.ogg",
                "sounds/footsteps/water/water_through2.ogg",
                "sounds/footsteps/water/water_through3.ogg",
                "sounds/footsteps/water/water_through4.ogg",
                "sounds/footsteps/water/water_through5.ogg",
                "sounds/footsteps/water/water_through6.ogg",
                "sounds/footsteps/water/water_through7.ogg",
                "sounds/footsteps/water/water_through8.ogg",
                "sounds/footsteps/water/water_through9.ogg",
                "sounds/footsteps/water/water_through10.ogg",
                "sounds/footsteps/water/water_through11.ogg",
            ],
        }
    }

    /// Footstep running sound asset paths.
    pub fn run_sounds(&self) -> &'static [&'static str] {
        match self {
            Self::Grass => &[
                "sounds/footsteps/grass/grass_run1.ogg",
                "sounds/footsteps/grass/grass_run2.ogg",
                "sounds/footsteps/grass/grass_run3.ogg",
                "sounds/footsteps/grass/grass_run4.ogg",
            ],
            Self::Dirt => &[
                "sounds/footsteps/dirt/dirt_run1.ogg",
                "sounds/footsteps/dirt/dirt_run2.ogg",
                "sounds/footsteps/dirt/dirt_run3.ogg",
                "sounds/footsteps/dirt/dirt_run4.ogg",
                "sounds/footsteps/dirt/dirt_run5.ogg",
                "sounds/footsteps/dirt/dirt_run6.ogg",
                "sounds/footsteps/dirt/dirt_run7.ogg",
                "sounds/footsteps/dirt/dirt_run8.ogg",
                "sounds/footsteps/dirt/dirt_run9.ogg",
                "sounds/footsteps/dirt/dirt_run10.ogg",
                "sounds/footsteps/dirt/dirt_run11.ogg",
            ],
            Self::Stone => &[
                "sounds/footsteps/stone/stone_run1.ogg",
                "sounds/footsteps/stone/stone_run2.ogg",
                "sounds/footsteps/stone/stone_run3.ogg",
                "sounds/footsteps/stone/stone_run4.ogg",
                "sounds/footsteps/stone/stone_run5.ogg",
                "sounds/footsteps/stone/stone_run6.ogg",
                "sounds/footsteps/stone/stone_run7.ogg",
                "sounds/footsteps/stone/stone_run8.ogg",
                "sounds/footsteps/stone/stone_run9.ogg",
                "sounds/footsteps/stone/stone_run10.ogg",
                "sounds/footsteps/stone/stone_run11.ogg",
            ],
            Self::Sand => &[
                "sounds/footsteps/sand/sand_run1.ogg",
                "sounds/footsteps/sand/sand_run2.ogg",
                "sounds/footsteps/sand/sand_run3.ogg",
                "sounds/footsteps/sand/sand_run4.ogg",
                "sounds/footsteps/sand/sand_run5.ogg",
                "sounds/footsteps/sand/sand_run6.ogg",
                "sounds/footsteps/sand/sand_run7.ogg",
                "sounds/footsteps/sand/sand_run8.ogg",
                "sounds/footsteps/sand/sand_run9.ogg",
                "sounds/footsteps/sand/sand_run10.ogg",
                "sounds/footsteps/sand/sand_run11.ogg",
            ],
            Self::Gravel => &[
                "sounds/footsteps/gravel/gravel_run1.ogg",
                "sounds/footsteps/gravel/gravel_run2.ogg",
                "sounds/footsteps/gravel/gravel_run3.ogg",
                "sounds/footsteps/gravel/gravel_run4.ogg",
                "sounds/footsteps/gravel/gravel_run5.ogg",
                "sounds/footsteps/gravel/gravel_run6.ogg",
                "sounds/footsteps/gravel/gravel_run7.ogg",
                "sounds/footsteps/gravel/gravel_run8.ogg",
                "sounds/footsteps/gravel/gravel_run9.ogg",
                "sounds/footsteps/gravel/gravel_run10.ogg",
                "sounds/footsteps/gravel/gravel_run11.ogg",
            ],
            Self::Snow => &[
                "sounds/footsteps/snow/snow_run1.ogg",
                "sounds/footsteps/snow/snow_run2.ogg",
                "sounds/footsteps/snow/snow_run3.ogg",
                "sounds/footsteps/snow/snow_run4.ogg",
                "sounds/footsteps/snow/snow_run5.ogg",
                "sounds/footsteps/snow/snow_run6.ogg",
                "sounds/footsteps/snow/snow_run7.ogg",
                "sounds/footsteps/snow/snow_run8.ogg",
                "sounds/footsteps/snow/snow_run9.ogg",
                "sounds/footsteps/snow/snow_run10.ogg",
                "sounds/footsteps/snow/snow_run11.ogg",
            ],
            _ => self.walk_sounds(),
        }
    }

    /// Landing sound asset paths after falling or jumping.
    pub fn land_sounds(&self) -> &'static [&'static str] {
        match self {
            Self::Grass => &[
                "sounds/footsteps/grass/grass_land1.ogg",
                "sounds/footsteps/grass/grass_land2.ogg",
                "sounds/footsteps/grass/grass_land3.ogg",
                "sounds/footsteps/grass/grass_land4.ogg",
            ],
            Self::Dirt => &[
                "sounds/footsteps/dirt/dirt_land1.ogg",
                "sounds/footsteps/dirt/dirt_land2.ogg",
                "sounds/footsteps/dirt/dirt_land3.ogg",
                "sounds/footsteps/dirt/dirt_land4.ogg",
            ],
            Self::Gravel => &[
                "sounds/footsteps/gravel/gravel_land1.ogg",
                "sounds/footsteps/gravel/gravel_land2.ogg",
            ],
            Self::Water => &[
                "sounds/liquids/splash.ogg",
                "sounds/liquids/splash2.ogg",
            ],
            _ => self.run_sounds(),
        }
    }
}
