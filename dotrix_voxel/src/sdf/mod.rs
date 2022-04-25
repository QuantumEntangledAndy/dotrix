use dotrix_core::ecs::System;
use dotrix_core::Application;

mod camera;
mod depth;
mod jump_flood;
mod map_data;
mod obb;
mod render;
mod sdf_calc;
mod tex_sdf;

use depth::*;
pub use jump_flood::*;
pub use obb::*;
pub use sdf_calc::*;
pub use tex_sdf::*;

/// Enables Voxel SDF Dotrix Extension
pub fn extension(app: &mut Application) {
    app.add_system(System::from(jump_flood::startup));
    app.add_system(System::from(jump_flood::compute));

    camera::extension(app);
    depth::extension(app);
    render::extension(app);
    sdf_calc::extension(app);
}
