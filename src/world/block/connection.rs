//! Pure algorithmic connected block logic for structural blocks (fences, walls, glass panes, bars).
//! Provides bitwise direction masks and deterministic connection state topology.

use super::voxel::Voxel;

/// Cardinal direction flags representing horizontal adjacency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ConnectionMask(pub u8);

impl ConnectionMask {
    pub const NONE: Self = Self(0);
    pub const NORTH: Self = Self(1 << 0);
    pub const SOUTH: Self = Self(1 << 1);
    pub const EAST: Self = Self(1 << 2);
    pub const WEST: Self = Self(1 << 3);
    pub const UP: Self = Self(1 << 4);
    pub const DOWN: Self = Self(1 << 5);

    pub const CARDINAL_ALL: Self =
        Self(Self::NORTH.0 | Self::SOUTH.0 | Self::EAST.0 | Self::WEST.0);

    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline(always)]
    pub const fn has_north(self) -> bool {
        self.contains(Self::NORTH)
    }

    #[inline(always)]
    pub const fn has_south(self) -> bool {
        self.contains(Self::SOUTH)
    }

    #[inline(always)]
    pub const fn has_east(self) -> bool {
        self.contains(Self::EAST)
    }

    #[inline(always)]
    pub const fn has_west(self) -> bool {
        self.contains(Self::WEST)
    }

    #[inline(always)]
    pub const fn has_up(self) -> bool {
        self.contains(Self::UP)
    }

    #[inline(always)]
    pub const fn has_down(self) -> bool {
        self.contains(Self::DOWN)
    }

    /// Counts how many horizontal cardinal directions (N, S, E, W) are connected (0..=4).
    #[inline(always)]
    pub const fn cardinal_count(self) -> usize {
        ((self.0 & Self::NORTH.0) != 0) as usize
            + ((self.0 & Self::SOUTH.0) != 0) as usize
            + ((self.0 & Self::EAST.0) != 0) as usize
            + ((self.0 & Self::WEST.0) != 0) as usize
    }

    /// True if the block is an isolated post with no cardinal connections.
    #[inline(always)]
    pub const fn is_isolated_post(self) -> bool {
        self.cardinal_count() == 0
    }

    /// True if the block connects only along the North-South axis in a straight line.
    #[inline(always)]
    pub const fn is_straight_north_south(self) -> bool {
        self.cardinal_count() == 2 && self.has_north() && self.has_south()
    }

    /// True if the block connects only along the East-West axis in a straight line.
    #[inline(always)]
    pub const fn is_straight_east_west(self) -> bool {
        self.cardinal_count() == 2 && self.has_east() && self.has_west()
    }

    /// True if the block forms an L-corner (2 cardinal connections that are perpendicular).
    #[inline(always)]
    pub const fn is_corner(self) -> bool {
        self.cardinal_count() == 2
            && !self.is_straight_north_south()
            && !self.is_straight_east_west()
    }

    /// True if the block forms a 3-way T-junction.
    #[inline(always)]
    pub const fn is_t_junction(self) -> bool {
        self.cardinal_count() == 3
    }

    /// True if the block forms a 4-way cross intersection.
    #[inline(always)]
    pub const fn is_four_way(self) -> bool {
        self.cardinal_count() == 4
    }
}

/// Category of connected architectural elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConnectedCategory {
    /// Wooden or stone fences (connects to other fences and solid full blocks).
    Fence,
    /// Stone or brick walls (connects to other walls and solid full blocks).
    Wall,
    /// Glass panes (connects to other glass panes and solid full blocks).
    GlassPane,
    /// Iron bars (connects to other iron bars and solid full blocks).
    IronBars,
}

impl ConnectedCategory {
    /// Determines if a neighbor voxel can connect to a block of this category.
    pub fn can_connect_to(&self, neighbor: Option<Voxel>) -> bool {
        let Some(voxel) = neighbor else {
            return false;
        };

        if voxel.is_empty() {
            return false;
        }

        // Full solid blocks always accept connections from fences, walls, panes, and bars
        if voxel.is_solid_opaque() {
            return true;
        }

        // Check category-specific connections
        let id = voxel.id_str();
        match self {
            Self::GlassPane => id.contains("glass"),
            Self::Fence => id.contains("fence") || id.contains("gate"),
            Self::Wall => id.contains("wall"),
            Self::IronBars => id.contains("bar"),
        }
    }

    /// Pure function computing the cardinal connection mask for a block given its 4 horizontal neighbors.
    pub fn compute_cardinal_mask(
        &self,
        north: Option<Voxel>,
        south: Option<Voxel>,
        east: Option<Voxel>,
        west: Option<Voxel>,
    ) -> ConnectionMask {
        let mut mask = ConnectionMask::NONE;

        if self.can_connect_to(north) {
            mask = mask.with(ConnectionMask::NORTH);
        }
        if self.can_connect_to(south) {
            mask = mask.with(ConnectionMask::SOUTH);
        }
        if self.can_connect_to(east) {
            mask = mask.with(ConnectionMask::EAST);
        }
        if self.can_connect_to(west) {
            mask = mask.with(ConnectionMask::WEST);
        }

        mask
    }

    /// Pure function computing full 6-way connection mask [North, South, East, West, Up, Down].
    pub fn compute_6way_mask(&self, neighbors: [Option<Voxel>; 6]) -> ConnectionMask {
        let [north, south, east, west, up, down] = neighbors;
        let mut mask = self.compute_cardinal_mask(north, south, east, west);

        if self.can_connect_to(up) {
            mask = mask.with(ConnectionMask::UP);
        }
        if self.can_connect_to(down) {
            mask = mask.with(ConnectionMask::DOWN);
        }

        mask
    }
}
