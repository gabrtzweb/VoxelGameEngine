use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::{
    environment::weather::state::WeatherManager,
    player::PlayerCamera,
    world::{Voxel, storage::VoxelWorld},
};

/// Event dispatched when a lightning bolt strikes, carrying distance to player for acoustic delay.
#[derive(Event, Debug, Clone)]
pub struct TriggerThunderEvent {
    pub strike_pos: Vec3,
    pub distance_to_camera: f32,
}

/// Global tracking resource for lightning strike frequency and full-scene atmospheric flash.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct LightningState {
    /// Countdown in seconds until the next procedural strike during thunderstorms.
    pub strike_timer: f32,
    /// Atmospheric flash intensity [0.0..=1.0] driving full-scene ambient illumination spikes.
    pub flash_intensity: f32,
    /// Remaining duration of the active atmospheric flash.
    pub flash_timer: f32,
    /// World position of the most recent lightning ground strike.
    pub last_strike_pos: Option<Vec3>,
}

impl Default for LightningState {
    fn default() -> Self {
        Self {
            strike_timer: 5.0,
            flash_intensity: 0.0,
            flash_timer: 0.0,
            last_strike_pos: None,
        }
    }
}

/// Marker component on active lightning bolt geometry entities.
#[derive(Component)]
pub struct LightningBoltEntity {
    pub age: f32,
    pub max_age: f32,
}

/// Marker component on localized flash point lights at the strike impact point.
#[derive(Component)]
pub struct LightningImpactLight {
    pub timer: f32,
}

pub struct LightningPlugin;

impl Plugin for LightningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightningState>()
            .register_type::<LightningState>()
            .add_systems(Update, (tick_lightning, update_lightning_effects));
    }
}

/// Ticks strike timers, spawns fractal lightning bolts and triggers full-scene flashes.
pub fn tick_lightning(
    time: Res<Time>,
    weather: Option<Res<WeatherManager>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<PlayerCamera>)>,
    voxel_world: Res<VoxelWorld>,
    mut lightning_state: ResMut<LightningState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    // Decay atmosphere flash
    if lightning_state.flash_timer > 0.0 {
        lightning_state.flash_timer -= dt;
        lightning_state.flash_intensity = (lightning_state.flash_timer / 0.20).clamp(0.0, 1.0);
    } else {
        lightning_state.flash_intensity = 0.0;
    }

    let Some(ref w) = weather else {
        return;
    };

    // Only strike during thunderstorms or heavy stormy downpours
    let is_storm = w.current.is_storm() || w.thunder_intensity > 0.15;
    if !is_storm {
        lightning_state.strike_timer = 6.0;
        return;
    }

    lightning_state.strike_timer -= dt;
    if lightning_state.strike_timer > 0.0 {
        return;
    }

    let Ok(cam_transform) = camera_query.single() else {
        return;
    };
    let cam_pos = cam_transform.translation;

    // Fast PRNG calculation for random strike location within 30-70m radius
    let pseudo_rand = (time.elapsed_secs() * 1000.0).fract();
    let angle = pseudo_rand * std::f32::consts::TAU;
    let radius = 32.0 + (pseudo_rand * 37.0).fract() * 38.0;

    let strike_x = (cam_pos.x + angle.cos() * radius).floor() as i32;
    let strike_z = (cam_pos.z + angle.sin() * radius).floor() as i32;

    // Scan top-down in column to find ground impact height
    let scan_start = (cam_pos.y + 45.0).floor() as i32;
    let scan_min = (cam_pos.y - 30.0).floor() as i32;
    let mut impact_y = cam_pos.y - 10.0;

    for y in (scan_min..=scan_start).rev() {
        if let Some(voxel) = voxel_world.get_voxel(IVec3::new(strike_x, y, strike_z)) {
            if voxel != Voxel::Air && !voxel.is_empty() {
                impact_y = y as f32 + 1.0;
                break;
            }
        }
    }

    let strike_target = Vec3::new(strike_x as f32 + 0.5, impact_y, strike_z as f32 + 0.5);
    let cloud_origin = Vec3::new(
        strike_target.x + (pseudo_rand * 10.0 - 5.0),
        175.0,
        strike_target.z + ((pseudo_rand * 20.0).fract() * 10.0 - 5.0),
    );

    // 1. Build procedural fractal lightning mesh
    let bolt_mesh = generate_fractal_bolt_mesh(cloud_origin, strike_target, pseudo_rand);
    let mesh_handle = meshes.add(bolt_mesh);

    let bolt_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.6, 1.7, 2.4),
        emissive: LinearRgba::new(2.5, 2.7, 4.0, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        ..default()
    });

    commands.spawn((
        LightningBoltEntity {
            age: 0.0,
            max_age: 0.28,
        },
        Mesh3d(mesh_handle),
        MeshMaterial3d(bolt_material),
        NotShadowCaster,
        NotShadowReceiver,
        Transform::default(),
        Visibility::Visible,
    ));

    // 2. Spawn temporary ground point light at impact position
    commands.spawn((
        LightningImpactLight { timer: 0.25 },
        PointLight {
            color: Color::srgb(0.85, 0.92, 1.0),
            intensity: 350_000.0,
            range: 90.0,
            ..default()
        },
        Transform::from_translation(strike_target + Vec3::Y * 2.0),
    ));

    // 3. Trigger full-scene atmosphere flash
    lightning_state.flash_intensity = 1.0;
    lightning_state.flash_timer = 0.20;
    lightning_state.last_strike_pos = Some(strike_target);

    // 4. Acoustic thunder delay calculation (Speed of sound ~343 m/s)
    let dist = cam_pos.distance(strike_target);
    commands.trigger(TriggerThunderEvent {
        strike_pos: strike_target,
        distance_to_camera: dist,
    });

    // Reset strike countdown: strikes occur every 3.5 to 8.5 seconds during thunderstorms
    let next_interval = 3.5 + ((pseudo_rand * 43.0).fract() * 5.0);
    lightning_state.strike_timer = next_interval;
}

/// Updates lifetimes of active bolt entities and impact point lights.
pub fn update_lightning_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut bolt_query: Query<(Entity, &mut LightningBoltEntity)>,
    mut light_query: Query<(Entity, &mut LightningImpactLight)>,
) {
    let dt = time.delta_secs();

    for (entity, mut bolt) in &mut bolt_query {
        bolt.age += dt;
        if bolt.age >= bolt.max_age {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut light) in &mut light_query {
        light.timer -= dt;
        if light.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Generates a jagged ribbon mesh with midpoint displacement and branching forks.
fn generate_fractal_bolt_mesh(origin: Vec3, target: Vec3, seed: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let mut vertex_cursor = 0u32;

    // Subdivide main trunk into 14 segments
    let segments = 14;
    let mut points = Vec::with_capacity(segments + 1);
    points.push(origin);

    let step = (target - origin) / segments as f32;

    for i in 1..segments {
        let t = i as f32 / segments as f32;
        let mut pt = origin + step * (i as f32);

        // Add jagged lateral displacement
        let phase = seed * 100.0 + (i as f32) * 1.7;
        let jx = phase.sin() * 2.2 * (1.0 - (t - 0.5).abs());
        let jz = (phase * 1.3).cos() * 2.2 * (1.0 - (t - 0.5).abs());

        pt.x += jx;
        pt.z += jz;
        points.push(pt);
    }
    points.push(target);

    // Build ribbon quads along trunk points
    let trunk_width = 0.32;
    for i in 0..points.len() - 1 {
        let p0 = points[i];
        let p1 = points[i + 1];

        // 2 cross-aligned ribbons (X-plane and Z-plane) to ensure 360-degree visibility
        add_ribbon_quad(
            p0,
            p1,
            Vec3::X * trunk_width,
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut colors,
            &mut indices,
            &mut vertex_cursor,
        );
        add_ribbon_quad(
            p0,
            p1,
            Vec3::Z * trunk_width,
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut colors,
            &mut indices,
            &mut vertex_cursor,
        );

        // Branching fork at 1/3 and 2/3 of descent
        if i == 4 || i == 9 {
            let branch_dir = if i == 4 {
                Vec3::new(1.4, -0.6, 0.8)
            } else {
                Vec3::new(-1.2, -0.7, -1.0)
            };
            let b0 = p0;
            let b1 = b0 + branch_dir * 8.0;
            let b2 = b1 + branch_dir * 6.0 + Vec3::new(0.5, -4.0, 0.5);

            let branch_width = 0.16;
            add_ribbon_quad(
                b0,
                b1,
                Vec3::X * branch_width,
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
                &mut indices,
                &mut vertex_cursor,
            );
            add_ribbon_quad(
                b1,
                b2,
                Vec3::X * (branch_width * 0.6),
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut colors,
                &mut indices,
                &mut vertex_cursor,
            );
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[inline]
fn add_ribbon_quad(
    p0: Vec3,
    p1: Vec3,
    offset: Vec3,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    vertex_cursor: &mut u32,
) {
    let v0 = p0 - offset;
    let v1 = p0 + offset;
    let v2 = p1 + offset;
    let v3 = p1 - offset;

    positions.push([v0.x, v0.y, v0.z]);
    positions.push([v1.x, v1.y, v1.z]);
    positions.push([v2.x, v2.y, v2.z]);
    positions.push([v3.x, v3.y, v3.z]);

    let norm = [0.0, 1.0, 0.0];
    normals.push(norm);
    normals.push(norm);
    normals.push(norm);
    normals.push(norm);

    uvs.push([0.0, 0.0]);
    uvs.push([1.0, 0.0]);
    uvs.push([1.0, 1.0]);
    uvs.push([0.0, 1.0]);

    let col = [1.0, 1.0, 1.2, 0.95];
    colors.push(col);
    colors.push(col);
    colors.push(col);
    colors.push(col);

    indices.push(*vertex_cursor);
    indices.push(*vertex_cursor + 1);
    indices.push(*vertex_cursor + 2);
    indices.push(*vertex_cursor);
    indices.push(*vertex_cursor + 2);
    indices.push(*vertex_cursor + 3);

    *vertex_cursor += 4;
}
