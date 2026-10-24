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
    Pebble = 6,
    PebbleCluster = 7,
    CrossBillboard = 8,
    Bamboo = 9,
    Cocoon = 10,
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
    /// All 11 registered block shapes in the engine.
    pub const fn all() -> [BlockShape; 11] {
        [
            BlockShape::Full,
            BlockShape::Slab,
            BlockShape::Stair,
            BlockShape::Column,
            BlockShape::Torch,
            BlockShape::Basket,
            BlockShape::Pebble,
            BlockShape::PebbleCluster,
            BlockShape::CrossBillboard,
            BlockShape::Bamboo,
            BlockShape::Cocoon,
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
            Self::Pebble => "Pebble",
            Self::PebbleCluster => "Pebble Cluster",
            Self::CrossBillboard => "Cross-Billboard",
            Self::Bamboo => "Bamboo",
            Self::Cocoon => "Cocoon",
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
            Self::Pebble => "PEBBLE",
            Self::PebbleCluster => "PEBBLE_CLUSTER",
            Self::CrossBillboard => "CROSS",
            Self::Bamboo => "BAMBOO",
            Self::Cocoon => "COCOON",
        }
    }

    pub const fn orientation_count(self) -> u8 {
        match self {
            Self::Full => 3,
            Self::Basket | Self::CrossBillboard | Self::Bamboo | Self::Cocoon => 1,
            Self::Slab => 6,
            Self::Stair => 8,
            Self::Column => 6,
            Self::Torch => 5,
            Self::Pebble | Self::PebbleCluster => 4,
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
            Self::Basket | Self::CrossBillboard | Self::Bamboo | Self::Cocoon => "Standard",
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
            Self::Pebble | Self::PebbleCluster => match orientation % 4 {
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
            Self::Full | Self::Basket => ([Vec3::ZERO, Vec3::ONE], None),
            Self::CrossBillboard => (
                [
                    Vec3::new(0.125, 0.0, 0.125),
                    Vec3::new(0.875, 0.8, 0.875),
                ],
                None,
            ),
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
            Self::Pebble => {
                let b = rotate_aabb_y(
                    Vec3::new(7.0 / 16.0, 0.0, 5.0 / 16.0),
                    Vec3::new(11.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0),
                    orientation,
                );
                (b, None)
            }
            Self::PebbleCluster => {
                let b1 = rotate_aabb_y(
                    Vec3::new(7.0 / 16.0, 0.0, 8.0 / 16.0),
                    Vec3::new(12.0 / 16.0, 2.0 / 16.0, 13.0 / 16.0),
                    orientation,
                );
                let b2 = rotate_aabb_y(
                    Vec3::new(4.0 / 16.0, 0.0, 3.0 / 16.0),
                    Vec3::new(12.0 / 16.0, 1.0 / 16.0, 10.0 / 16.0),
                    orientation,
                );
                (b1, Some(b2))
            }
            Self::Bamboo => {
                let min = Vec3::new(6.5 / 16.0, 0.0, 6.5 / 16.0);
                let max = Vec3::new(9.5 / 16.0, 1.0, 9.5 / 16.0);
                ([min, max], None)
            }
            Self::Cocoon => {
                let min = Vec3::new(2.0 / 16.0, 0.0, 2.0 / 16.0);
                let max = Vec3::new(14.0 / 16.0, 14.0 / 16.0, 14.0 / 16.0);
                ([min, max], None)
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
