/// Simple terrain generator

use dotrix_core::assets::Mesh;
use dotrix_math::{ InnerSpace, Vec3 };

use crate::{ Component, Generator, HeightMap, VecXZ };

#[derive(Default)]
pub struct Simple {
    pub heights: HeightMap,
    pub max_height: f32,
    pub offset: VecXZ<i32>,
    pub dirty: bool,
}

impl Generator for Simple {
    fn get(
        &self,
        component: Component,
        position: VecXZ<i32>,
        scale: u32,
        unit_size: f32
    ) -> Option<Mesh> {
        let vertices_per_side = component.vertices_per_side();
        let units_per_side = vertices_per_side - 1;
        let half_units_per_side = units_per_side as i32 / 2;

        // TODO: calculate offset from position

        let capacity = (vertices_per_side * vertices_per_side) as usize;
        let mut positions = Vec::with_capacity(capacity);
        let mut uvs = Vec::with_capacity(capacity);
        let mut normals = vec![[0.0, 0.0, 0.0]; capacity];
        let mut indices = Vec::with_capacity(3 * 2 * units_per_side * units_per_side);

        // Map position
        let map_x = (position.x - scale as i32) * half_units_per_side;
        let map_z = (position.z - scale as i32) * half_units_per_side;

        let height_map_x0 = map_x - self.offset.x;
        let height_map_z0 = map_z - self.offset.z;

        let x0 = map_x as f32 * unit_size as f32;
        let z0 = map_z as f32 * unit_size as f32;

        println!("  Get mesh for Position: ({}; {}), Map: ({}; {}), World: ({:?}; {:?}), HeightMap: ({:?}; {:?})",
            position.x, position.z, map_x, map_z, x0, z0,
height_map_x0,
height_map_z0,
            );

        for z in 0..vertices_per_side {
            let scale_z = z as i32 * scale as i32;
            let world_z = z0 + scale_z as f32 * unit_size;
            let height_map_z = height_map_z0 + scale_z;
            for x in 0..vertices_per_side {
                let scale_x = x as i32 * scale as i32;
                let world_x = x0 + scale_x as f32 * unit_size;
                let height_map_x = height_map_x0 + scale_x;
                let height = if height_map_x > 0 && height_map_z > 0 {
                    self.heights.get(
                        (height_map_x as i32) as u32,
                        (height_map_z as i32) as u32
                    )
                } else { None };

                let world_y = self.max_height * height.unwrap_or(0.0);

                positions.push([world_x, world_y, world_z]);
                uvs.push([
                    x as f32 / units_per_side as f32,
                    z as f32 / units_per_side as f32,
                ]);
            }
        }

        for z in 0..units_per_side {
            let i = (z * vertices_per_side) as u32;
            for x in 0..units_per_side {
                let i00 = i + x as u32;
                let i10 = i00 + 1;
                let i01 = i00 + vertices_per_side as u32;
                let i11 = i01 + 1;

                indices.push(i10);
                indices.push(i00);
                indices.push(i01);
                indices.push(i10);
                indices.push(i01);
                indices.push(i11);
            }
        }
        let indices_count = indices.len();
        for i in (0..indices_count).step_by(3) {
            let i0 = indices[i] as usize;
            let i1 = indices[i + 1] as usize;
            let i2 = indices[i + 2] as usize;
            // get the face
            let p0 = Vec3::from(positions[i0]);
            let p1 = Vec3::from(positions[i1]);
            let p2 = Vec3::from(positions[i2]);

            let n1 = p1 - p0;
            let n2 = p2 - p0;
            let normal = n1.cross(n2).normalize();

            normals[i0] = (Vec3::from(normals[i0]) + normal).into();
            normals[i1] = (Vec3::from(normals[i1]) + normal).into();
            normals[i2] = (Vec3::from(normals[i2]) + normal).into();
        }

        for normal in normals.iter_mut() {
            let normalized = Vec3::from(*normal).normalize();
            normal[0] = normalized.x;
            normal[1] = normalized.y;
            normal[2] = normalized.z;
        }

        let mut mesh = Mesh::default();
        mesh.with_vertices(&positions);
        mesh.with_vertices(&normals);
        mesh.with_vertices(&uvs);
        mesh.with_indices(&indices);

        Some(mesh)
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self) {
        self.dirty = true;
    }
}

impl From<HeightMap> for Simple {
    fn from(heights: HeightMap) -> Self {
        let height_map_size = heights.size();
        let offset = VecXZ::new(
            -(height_map_size.x as i32) / 2,
            -(height_map_size.z as i32) / 2
        );

        Self {
            heights,
            offset,
            max_height: 100.0,
            dirty: true
        }
    }
}