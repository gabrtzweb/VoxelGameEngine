use voxel_game_engine::world::block::{ConnectedCategory, ConnectionMask, Voxel};

#[test]
fn test_connection_mask_cardinal_counts() {
    let mask_empty = ConnectionMask::NONE;
    assert_eq!(mask_empty.cardinal_count(), 0);
    assert!(mask_empty.is_isolated_post());

    let mask_one = ConnectionMask::NORTH;
    assert_eq!(mask_one.cardinal_count(), 1);
    assert!(!mask_one.is_isolated_post());

    let mask_straight = ConnectionMask::NORTH.with(ConnectionMask::SOUTH);
    assert_eq!(mask_straight.cardinal_count(), 2);
    assert!(mask_straight.is_straight_north_south());
    assert!(!mask_straight.is_straight_east_west());
    assert!(!mask_straight.is_corner());

    let mask_corner = ConnectionMask::NORTH.with(ConnectionMask::EAST);
    assert_eq!(mask_corner.cardinal_count(), 2);
    assert!(mask_corner.is_corner());
    assert!(!mask_corner.is_straight_north_south());

    let mask_t = ConnectionMask::NORTH
        .with(ConnectionMask::EAST)
        .with(ConnectionMask::WEST);
    assert_eq!(mask_t.cardinal_count(), 3);
    assert!(mask_t.is_t_junction());

    let mask_cross = ConnectionMask::CARDINAL_ALL;
    assert_eq!(mask_cross.cardinal_count(), 4);
    assert!(mask_cross.is_four_way());
}

#[test]
fn test_fence_connections_to_solid_and_air() {
    let category = ConnectedCategory::Fence;

    // Air must never connect
    assert!(!category.can_connect_to(None));
    assert!(!category.can_connect_to(Some(Voxel::Air)));

    // Full solid opaque blocks (like Andesite, Basalt) always connect
    assert!(category.can_connect_to(Some(Voxel::Andesite)));
    assert!(category.can_connect_to(Some(Voxel::Basalt)));

    // Compute cardinal mask with Andesite to the North and Air everywhere else
    let mask = category.compute_cardinal_mask(
        Some(Voxel::Andesite),
        Some(Voxel::Air),
        None,
        Some(Voxel::Air),
    );
    assert!(mask.has_north());
    assert!(!mask.has_south());
    assert!(!mask.has_east());
    assert!(!mask.has_west());
    assert_eq!(mask.cardinal_count(), 1);
}

#[test]
fn test_6way_connection_mask() {
    let category = ConnectedCategory::Wall;
    let neighbors = [
        Some(Voxel::Andesite), // North
        Some(Voxel::Andesite), // South
        None,                  // East
        None,                  // West
        Some(Voxel::Andesite), // Up
        None,                  // Down
    ];

    let mask = category.compute_6way_mask(neighbors);
    assert!(mask.has_north());
    assert!(mask.has_south());
    assert!(mask.has_up());
    assert!(!mask.has_east());
    assert!(!mask.has_west());
    assert!(!mask.has_down());
    assert!(mask.is_straight_north_south());
}
