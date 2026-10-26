use bevy::{platform::collections::HashMap, prelude::*};
use crate::{
    audio::{PlaySpatialSfxEvent, material::SoundMaterial},
    gameplay::feedback::{BlockBreakEvent, BlockPlaceEvent},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum BlockSoundKind {
    Break,
    Place,
}

#[derive(Resource, Default)]
struct BlockSoundCache {
    handles: HashMap<(SoundMaterial, BlockSoundKind), Vec<Handle<AudioSource>>>,
    rng_state: u64,
}

impl BlockSoundCache {
    fn next_float(&mut self) -> f32 {
        if self.rng_state == 0 {
            self.rng_state = 0x5851_F42D_4C95_7F2D;
        }
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng_state >> 32) as u32 as f32) / 4294967295.0
    }

    fn get_or_load(
        &mut self,
        material: SoundMaterial,
        kind: BlockSoundKind,
        asset_server: &AssetServer,
    ) -> Option<Handle<AudioSource>> {
        self.handles.entry((material, kind)).or_insert_with(|| {
            let paths = match kind {
                BlockSoundKind::Break => material.break_sounds(),
                BlockSoundKind::Place => material.place_sounds(),
            };
            paths.iter().map(|p| asset_server.load(*p)).collect()
        });

        let rand = self.next_float();
        let list = self.handles.get(&(material, kind))?;
        if list.is_empty() {
            None
        } else {
            let idx = (rand * list.len() as f32) as usize % list.len();
            Some(list[idx].clone())
        }
    }
}

pub struct BlockAudioPlugin;

impl Plugin for BlockAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockSoundCache>()
            .add_observer(handle_block_break_audio)
            .add_observer(handle_block_place_audio);
    }
}

fn handle_block_break_audio(
    event: On<BlockBreakEvent>,
    asset_server: Res<AssetServer>,
    mut cache: ResMut<BlockSoundCache>,
    mut commands: Commands,
) {
    let voxel = event.voxel;
    if voxel.is_empty() {
        return;
    }

    let material = SoundMaterial::from_voxel(voxel);
    let rand_val = cache.next_float();
    let sound_opt = cache.get_or_load(material, BlockSoundKind::Break, &asset_server);

    let Some(sound) = sound_opt else {
        return;
    };

    let world_pos = event.position.as_vec3() + Vec3::splat(0.5);
    // Subtle pitch variation between 0.94 and 1.06 to prevent monotonous sound
    let speed = 0.94 + rand_val * 0.12;

    commands.trigger(
        PlaySpatialSfxEvent::new(sound, world_pos, 0.90, 28.0).with_speed(speed),
    );
}

fn handle_block_place_audio(
    event: On<BlockPlaceEvent>,
    asset_server: Res<AssetServer>,
    mut cache: ResMut<BlockSoundCache>,
    mut commands: Commands,
) {
    let voxel = event.voxel;
    if voxel.is_empty() {
        return;
    }

    let material = SoundMaterial::from_voxel(voxel);
    let rand_val = cache.next_float();
    let sound_opt = cache.get_or_load(material, BlockSoundKind::Place, &asset_server);

    let Some(sound) = sound_opt else {
        return;
    };

    let world_pos = event.position.as_vec3() + Vec3::splat(0.5);
    let speed = 0.94 + rand_val * 0.12;

    commands.trigger(
        PlaySpatialSfxEvent::new(sound, world_pos, 0.85, 24.0).with_speed(speed),
    );
}
