#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::Arc;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use super::chunk::{BlockShape, CHUNK_VOLUME, Chunk, ChunkStorage, Voxel};

const CHUNK_MAGIC: [u8; 4] = *b"VCHK";
const CHUNK_FORMAT_VERSION: u8 = 1;

/// High-performance chunk caching layer to eliminate redundant terrain generation
/// when moving back and forth across chunk boundaries or streaming render distances.
#[derive(Resource)]
pub struct ChunkCache {
    /// In-memory LRU cache of recently unloaded chunks.
    entries: HashMap<IVec3, Arc<Chunk>>,
    /// Eviction order queue tracking access recency.
    lru_order: VecDeque<IVec3>,
    /// Maximum number of chunks kept in memory cache (e.g. 2048 chunks ~ 3-4 MB total).
    capacity: usize,
    /// Statistics: total cache hits since startup.
    hits: u64,
    /// Statistics: total cache misses since startup.
    misses: u64,
}

impl Default for ChunkCache {
    fn default() -> Self {
        Self::new(2048)
    }
}

impl ChunkCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::default(),
            lru_order: VecDeque::with_capacity(capacity),
            capacity,
            hits: 0,
            misses: 0,
        }
    }

    /// Retrieves an `Arc<Chunk>` from cache if present.
    pub fn get(&mut self, coordinate: &IVec3) -> Option<Arc<Chunk>> {
        if let Some(chunk) = self.entries.get(coordinate) {
            self.hits += 1;
            Some(chunk.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Takes a chunk out of the cache (removing it so it can be moved into VoxelWorld).
    pub fn take(&mut self, coordinate: &IVec3) -> Option<Arc<Chunk>> {
        if let Some(chunk) = self.entries.remove(coordinate) {
            self.hits += 1;
            Some(chunk)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Inserts an unloaded chunk into the cache, evicting the oldest if capacity is exceeded.
    pub fn insert(&mut self, coordinate: IVec3, chunk: Arc<Chunk>) {
        if self.entries.contains_key(&coordinate) {
            self.entries.insert(coordinate, chunk);
            return;
        }

        while self.entries.len() >= self.capacity {
            if let Some(oldest) = self.lru_order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }

        self.lru_order.push_back(coordinate);
        self.entries.insert(coordinate, chunk);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn hits(&self) -> u64 {
        self.hits
    }

    #[inline]
    pub fn misses(&self) -> u64 {
        self.misses
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
    }
}

/// Serializes a Chunk into a compact, zero-overhead binary payload.
pub fn serialize_chunk(chunk: &Chunk) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(256);
    buffer.extend_from_slice(&CHUNK_MAGIC);
    buffer.push(CHUNK_FORMAT_VERSION);
    buffer.push(chunk.is_subterranean() as u8);

    match chunk.storage() {
        ChunkStorage::Uniform(v) => {
            buffer.push(0); // Tag 0 = Uniform
            buffer.push(*v as u8);
        }
        ChunkStorage::Paletted {
            palette,
            palette_len,
            indices,
        } => {
            buffer.push(1); // Tag 1 = Paletted
            buffer.push(*palette_len);
            for i in 0..(*palette_len as usize) {
                buffer.push(palette[i] as u8);
            }
            buffer.extend_from_slice(indices.as_slice());
        }
        ChunkStorage::Dense(voxels) => {
            buffer.push(2); // Tag 2 = Dense
            for &v in voxels.iter() {
                buffer.push(v as u8);
            }
        }
    }

    // Dynamic shapes
    let shapes = chunk.shapes();
    buffer.extend_from_slice(&(shapes.len() as u16).to_le_bytes());
    for (&idx, &(shape, orientation)) in shapes.iter() {
        buffer.extend_from_slice(&(idx as u16).to_le_bytes());
        buffer.push(shape as u8);
        buffer.push(orientation);
    }

    // Dynamic extra slabs
    let extra_slabs = chunk.extra_slabs();
    buffer.extend_from_slice(&(extra_slabs.len() as u16).to_le_bytes());
    for (&idx, &(voxel, orientation)) in extra_slabs.iter() {
        buffer.extend_from_slice(&(idx as u16).to_le_bytes());
        buffer.push(voxel as u8);
        buffer.push(orientation);
    }

    // Dynamic fluid levels
    let fluid_levels = chunk.fluid_levels();
    buffer.extend_from_slice(&(fluid_levels.len() as u16).to_le_bytes());
    for (&idx, &level) in fluid_levels.iter() {
        buffer.extend_from_slice(&(idx as u16).to_le_bytes());
        buffer.push(level);
    }

    buffer
}

/// Deserializes a binary payload back into a full `Chunk`.
pub fn deserialize_chunk(bytes: &[u8]) -> Option<Chunk> {
    if bytes.len() < 7 {
        return None;
    }
    if &bytes[0..4] != &CHUNK_MAGIC {
        return None;
    }
    let version = bytes[4];
    if version != CHUNK_FORMAT_VERSION {
        return None;
    }
    let is_subterranean = bytes[5] != 0;
    let storage_tag = bytes[6];
    let mut offset = 7;

    let mut chunk = match storage_tag {
        0 => {
            if offset >= bytes.len() {
                return None;
            }
            let voxel_id = bytes[offset];
            offset += 1;
            let voxel = if voxel_id == 0 {
                Voxel::Air
            } else {
                *Voxel::ALL
                    .iter()
                    .find(|&&v| v as u8 == voxel_id)
                    .unwrap_or(&Voxel::Air)
            };
            Chunk::filled(voxel)
        }
        1 => {
            if offset >= bytes.len() {
                return None;
            }
            let palette_len = bytes[offset] as usize;
            offset += 1;
            if palette_len > 16 || offset + palette_len + (CHUNK_VOLUME / 2) > bytes.len() {
                return None;
            }
            let mut palette = [Voxel::Air; 16];
            for i in 0..palette_len {
                let vid = bytes[offset + i];
                palette[i] = if vid == 0 {
                    Voxel::Air
                } else {
                    *Voxel::ALL
                        .iter()
                        .find(|&&v| v as u8 == vid)
                        .unwrap_or(&Voxel::Air)
                };
            }
            offset += palette_len;
            let indices_bytes = &bytes[offset..offset + (CHUNK_VOLUME / 2)];
            offset += CHUNK_VOLUME / 2;

            let mut voxels = Vec::with_capacity(CHUNK_VOLUME);
            for i in 0..CHUNK_VOLUME {
                let packed = indices_bytes[i >> 1];
                let shift = (i & 1) * 4;
                let pal_idx = ((packed >> shift) & 0x0F) as usize;
                voxels.push(if pal_idx < palette_len {
                    palette[pal_idx]
                } else {
                    Voxel::Air
                });
            }
            Chunk::from_voxels(voxels)
        }
        2 => {
            if offset + CHUNK_VOLUME > bytes.len() {
                return None;
            }
            let mut voxels = Vec::with_capacity(CHUNK_VOLUME);
            for i in 0..CHUNK_VOLUME {
                let vid = bytes[offset + i];
                let v = if vid == 0 {
                    Voxel::Air
                } else {
                    *Voxel::ALL
                        .iter()
                        .find(|&&vx| vx as u8 == vid)
                        .unwrap_or(&Voxel::Air)
                };
                voxels.push(v);
            }
            offset += CHUNK_VOLUME;
            Chunk::from_voxels(voxels)
        }
        _ => return None,
    };

    chunk.set_subterranean(is_subterranean);

    // Dynamic shapes
    if offset + 2 <= bytes.len() {
        let count = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;
        for _ in 0..count {
            if offset + 4 > bytes.len() {
                break;
            }
            let idx = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            let shape_id = bytes[offset + 2];
            let orientation = bytes[offset + 3];
            offset += 4;
            let shape = match shape_id {
                1 => BlockShape::Slab,
                2 => BlockShape::Stair,
                3 => BlockShape::Column,
                4 => BlockShape::Torch,
                5 => BlockShape::Basket,
                _ => BlockShape::Full,
            };
            chunk.set_shape_raw(idx, shape, orientation);
        }
    }

    // Dynamic extra slabs
    if offset + 2 <= bytes.len() {
        let count = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;
        for _ in 0..count {
            if offset + 4 > bytes.len() {
                break;
            }
            let idx = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            let vid = bytes[offset + 2];
            let orientation = bytes[offset + 3];
            offset += 4;
            let voxel = *Voxel::ALL
                .iter()
                .find(|&&vx| vx as u8 == vid)
                .unwrap_or(&Voxel::Air);
            chunk.set_extra_slab_raw(idx, (voxel, orientation));
        }
    }

    // Dynamic fluid levels
    if offset + 2 <= bytes.len() {
        let count = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;
        for _ in 0..count {
            if offset + 3 > bytes.len() {
                break;
            }
            let idx = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            let level = bytes[offset + 2];
            offset += 3;
            chunk.set_fluid_level_raw(idx, level);
        }
    }

    Some(chunk)
}
