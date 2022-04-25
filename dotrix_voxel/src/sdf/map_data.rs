//! Gpu represntation of Grid data needed to compute Map
//!
//! This is mostly various matrix tranfroms that are best precomputed
//!

use crate::Grid;
use dotrix_core::{
    renderer::{Buffer, Renderer},
    Transform,
};
use dotrix_math::*;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(crate) struct MapGpu {
    cube_transform: [[f32; 4]; 4],
    // Inverse cube_transform
    inv_cube_transform: [[f32; 4]; 4],
    // World transform of the voxel grid
    world_transform: [[f32; 4]; 4],
    // Inverse World transform of the voxel grid
    inv_world_transform: [[f32; 4]; 4],
    // Dimensions of the voxel
    grid_dimensions: [f32; 4],
    // Scale in world space
    world_scale: [f32; 4],
}

unsafe impl bytemuck::Zeroable for MapGpu {}
unsafe impl bytemuck::Pod for MapGpu {}

impl MapGpu {
    pub fn load(renderer: &Renderer, buffer: &mut Buffer, grid: &Grid, transform: &Transform) {
        let grid_size = grid.get_size();
        let scale = Mat4::from_nonuniform_scale(grid_size[0], grid_size[1], grid_size[2]);
        let world_transform_mat4: Mat4 = transform.matrix();
        let mut world_transform_tl: Mat4 = world_transform_mat4;
        world_transform_tl.x[3] = 0.;
        world_transform_tl.y[3] = 0.;
        world_transform_tl.z[3] = 0.;
        world_transform_tl.w[0] = 0.;
        world_transform_tl.w[1] = 0.;
        world_transform_tl.w[2] = 0.;
        world_transform_tl.w[3] = 1.;
        // let normal_transform: Mat4 = world_transform_tl
        //     .invert()
        //     .unwrap_or_else(Mat4::identity)
        //     .transpose();
        // let inv_normal_transform: Mat4 = world_transform_tl.transpose();
        let world_scale: [f32; 3] = transform.scale.into();
        let uniform = Self {
            cube_transform: scale.into(),
            inv_cube_transform: scale.invert().unwrap_or_else(Mat4::identity).into(),
            world_transform: world_transform_mat4.into(),
            inv_world_transform: world_transform_mat4
                .invert()
                .unwrap_or_else(Mat4::identity)
                .into(),
            // normal_transform: normal_transform.into(),
            // inv_normal_transform: inv_normal_transform.into(),
            grid_dimensions: [grid_size[0], grid_size[1], grid_size[2], 1.],
            world_scale: [world_scale[0], world_scale[1], world_scale[2], 1.],
        };
        // println!("grid_dimensions: {:?}", uniform.grid_dimensions);
        // println!("cube_transform: {:?}", uniform.cube_transform);
        // println!("inv_cube_transform: {:?}", uniform.inv_cube_transform);
        renderer.load_buffer(buffer, bytemuck::cast_slice(&[uniform]));
    }
}
