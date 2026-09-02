use super::chunk::{Chunk, Voxel, CHUNK_SIZE};

const NEIGHBOR_DIRECTIONS: [(isize, isize, isize); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

pub struct ChunkMesher;

impl ChunkMesher {
    pub fn exposed_face_count(chunk: &Chunk) -> usize {
        let mut face_count = 0;

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get(x, y, z) == Voxel::Air {
                        continue;
                    }

                    for (dx, dy, dz) in NEIGHBOR_DIRECTIONS {
                        let neighbor_x = x as isize + dx;
                        let neighbor_y = y as isize + dy;
                        let neighbor_z = z as isize + dz;

                        if Self::is_face_exposed(
                            chunk,
                            neighbor_x,
                            neighbor_y,
                            neighbor_z,
                        ) {
                            face_count += 1;
                        }
                    }
                }
            }
        }

        face_count
    }

    fn is_face_exposed(
        chunk: &Chunk,
        x: isize,
        y: isize,
        z: isize,
    ) -> bool {
        if !Chunk::is_inside(x, y, z) {
            return true;
        }

        chunk.get(x as usize, y as usize, z as usize) == Voxel::Air
    }
}