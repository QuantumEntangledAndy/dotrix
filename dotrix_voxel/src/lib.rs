//! Voxel Module
//!
//! Handles general voxel related content, such as conversion to an explicit
//! mesh using marching cubes or direct rendering.
//!

use dotrix_core::Application;
use lazy_static::lazy_static;
use tera::Tera;

mod grid;
mod material_set;
mod sdf;
mod voxel;

pub use grid::Grid;
pub use material_set::*;
pub use sdf::*;
pub use voxel::Voxel;

lazy_static! {
    pub(crate) static ref VOXEL_TEMPLATES: Tera = {
        let mut templates = Tera::default();
        templates
            .add_raw_templates(vec![
                (
                    "dotrix_voxel/common/camera.inc.wgsl",
                    include_str!("./sdf/common/camera.inc.wgsl"),
                ),
                (
                    "dotrix_voxel/common/obb.inc.wgsl",
                    include_str!("./sdf/common/obb.inc.wgsl"),
                ),
                (
                    "dotrix_voxel/common/ray.inc.wgsl",
                    include_str!("./sdf/common/ray.inc.wgsl"),
                ),
                (
                    "dotrix_voxel/circle_trace/map.inc.wgsl",
                    include_str!("./sdf/circle_trace/map.inc.wgsl"),
                ),
                (
                    "dotrix_voxel/circle_trace/accelerated_raytrace.inc.wgsl",
                    include_str!("./sdf/circle_trace/accelerated_raytrace.inc.wgsl"),
                ),
                (
                    "dotrix_voxel/depth/depth.wgsl",
                    include_str!("./sdf/depth/depth.wgsl"),
                ),
                (
                    "dotrix_voxel/depth/init.wgsl",
                    include_str!("./sdf/depth/init.wgsl"),
                ),
                (
                    "dotrix_voxel/ao/init.wgsl",
                    include_str!("./sdf/ao/init.wgsl"),
                ),
                ("dotrix_voxel/ao/ao.wgsl", include_str!("./sdf/ao/ao.wgsl")),
                (
                    "dotrix_voxel/ao/hemisphere_ambient_occulsion.inc.wgsl",
                    include_str!("./sdf/ao/hemisphere_ambient_occulsion.inc.wgsl"),
                ),
            ])
            .unwrap();
        templates
    };
}

/// Enables Voxel Dotrix Extension
pub fn extension(app: &mut Application) {
    sdf::extension(app);
}
