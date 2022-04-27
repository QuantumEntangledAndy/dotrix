use dotrix_core::ecs::System;
use dotrix_core::Application;

mod ao;
mod camera;
mod depth;
mod jump_flood;
mod map_data;
mod obb;
mod render;
mod sdf_calc;
mod shadows;
mod tex_sdf;

pub use ao::*;
pub use depth::*;
pub use jump_flood::*;
pub use obb::*;
pub use sdf_calc::*;
pub use shadows::*;
pub use tex_sdf::*;

/// Enables Voxel SDF Dotrix Extension
pub fn extension(app: &mut Application) {
    app.add_system(System::from(jump_flood::startup));
    app.add_system(System::from(jump_flood::compute));

    camera::extension(app);
    depth::extension(app);
    ao::extension(app);
    shadows::extension(app);
    render::extension(app);
    sdf_calc::extension(app);

    dotrix_pbr::extension(app);
}
