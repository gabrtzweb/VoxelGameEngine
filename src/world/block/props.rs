use super::voxel::Voxel;

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

impl Voxel {
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
