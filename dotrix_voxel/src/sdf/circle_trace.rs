use crate::Grid;
use crate::MaterialSet;
use crate::TexSdf;
use dotrix_core::{
    assets::{Mesh, Shader},
    ecs::{Const, Mut, System},
    renderer::{BindGroup, Binding, PipelineLayout, RenderOptions, Sampler, Stage},
    Application, Assets, Globals, Renderer, Transform, World,
};
use dotrix_math::*;
use dotrix_primitives::Cube;
use tera::{Context, Tera};

use super::camera::CameraBuffer;

const PIPELINE_LABEL: &str = "dotrix_voxel::sdf::circle_trace";

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct SdfBufferData {
    // This transform scales the 1x1x1 cube so that it totally encloses the
    // voxels
    pub cube_transform: [[f32; 4]; 4],
    // Inverse fo cube_transform
    pub inv_cube_transform: [[f32; 4]; 4],
    // World transform of the voxel grid
    pub world_transform: [[f32; 4]; 4],
    // Inverse of world_transform
    pub inv_world_transform: [[f32; 4]; 4],
    // Converts normals from object space to world space
    pub normal_transform: [[f32; 4]; 4],
    // Converts normals from world space to object space
    pub inv_normal_transform: [[f32; 4]; 4],
    // Dimensions of the voxel grid
    pub grid_dimensions: [f32; 4],
    // World space scale
    pub world_scale: [f32; 4],
}

unsafe impl bytemuck::Zeroable for SdfBufferData {}
unsafe impl bytemuck::Pod for SdfBufferData {}

pub fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut templates = Tera::default();
    templates
        .add_raw_templates(vec![
            (
                "circle_trace/render.wgsl",
                include_str!("./circle_trace/render.wgsl"),
            ),
            (
                "circle_trace/map.inc.wgsl",
                include_str!("./circle_trace/map.inc.wgsl"),
            ),
            (
                "circle_trace/accelerated_raytrace.inc.wgsl",
                include_str!("./circle_trace/accelerated_raytrace.inc.wgsl"),
            ),
            (
                "circle_trace/hemisphere_ambient_occulsion.inc.wgsl",
                include_str!("./circle_trace/hemisphere_ambient_occulsion.inc.wgsl"),
            ),
            (
                "circle_trace/lighting.inc.wgsl",
                include_str!("./circle_trace/lighting.inc.wgsl"),
            ),
            (
                "circle_trace/pbr.inc.wgsl",
                include_str!("./circle_trace/pbr.inc.wgsl"),
            ),
            (
                "circle_trace/soft_shadows_closet_approach.inc.wgsl",
                include_str!("./circle_trace/soft_shadows_closet_approach.inc.wgsl"),
            ),
            (
                "circle_trace/triplanar_surface.inc.wgsl",
                include_str!("./circle_trace/triplanar_surface.inc.wgsl"),
            ),
        ])
        .unwrap();

    let context = Context::new();
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: templates
            .render("circle_trace/render.wgsl", &context)
            .unwrap(),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);

    let mut mesh = Cube::builder(1.0).with_positions().mesh();
    mesh.load(&renderer);
    assets.store_as(mesh, PIPELINE_LABEL);
}

pub fn render(
    mut renderer: Mut<Renderer>,
    world: Const<World>,
    assets: Const<Assets>,
    globals: Const<Globals>,
) {
    let camera_buffer = globals
        .get::<CameraBuffer>()
        .expect("ProjView buffer must be loaded");

    for (grid, sdf, world_transform, material_set) in
        world.query::<(&Grid, &mut TexSdf, &Transform, &mut MaterialSet)>()
    {
        if sdf.pipeline.shader.is_null() {
            sdf.pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }
        if !sdf.pipeline.cycle(&renderer) {
            return;
        }
        let mesh = assets
            .get(
                assets
                    .find::<Mesh>(PIPELINE_LABEL)
                    .expect("Sdf mesh must be initialized with the dotrix_voxel startup system"),
            )
            .unwrap();

        let grid_size = grid.get_size();
        let scale = Mat4::from_nonuniform_scale(grid_size[0], grid_size[1], grid_size[2]);
        let world_transform_mat4: Mat4 = world_transform.matrix();
        let mut world_transform_tl: Mat4 = world_transform_mat4;
        world_transform_tl.x[3] = 0.;
        world_transform_tl.y[3] = 0.;
        world_transform_tl.z[3] = 0.;
        world_transform_tl.w[0] = 0.;
        world_transform_tl.w[1] = 0.;
        world_transform_tl.w[2] = 0.;
        world_transform_tl.w[3] = 1.;
        let normal_transform: Mat4 = world_transform_tl
            .invert()
            .unwrap_or_else(Mat4::identity)
            .transpose();
        let inv_normal_transform: Mat4 = world_transform_tl.transpose();
        let world_scale: [f32; 3] = world_transform.scale.into();
        let uniform = SdfBufferData {
            cube_transform: scale.into(),
            inv_cube_transform: scale.invert().unwrap_or_else(Mat4::identity).into(),
            world_transform: world_transform_mat4.into(),
            inv_world_transform: world_transform_mat4
                .invert()
                .unwrap_or_else(Mat4::identity)
                .into(),
            normal_transform: normal_transform.into(),
            inv_normal_transform: inv_normal_transform.into(),
            grid_dimensions: [grid_size[0], grid_size[1], grid_size[2], 1.],
            world_scale: [world_scale[0], world_scale[1], world_scale[2], 1.],
        };
        // println!("grid_dimensions: {:?}", uniform.grid_dimensions);
        // println!("cube_transform: {:?}", uniform.cube_transform);
        // println!("inv_cube_transform: {:?}", uniform.inv_cube_transform);
        renderer.load_buffer(&mut sdf.data, bytemuck::cast_slice(&[uniform]));

        let reload_required = material_set.load(&renderer, &assets);

        if reload_required {
            sdf.pipeline.bindings.unload();
        }

        if !sdf.pipeline.ready(&renderer) {
            let lights_buffer = globals
                .get::<LightStorageBuffer>()
                .expect("Light buffer must be loaded");

            let sampler = globals.get::<Sampler>().expect("Sampler must be loaded");

            if let Some(shader) = assets.get(sdf.pipeline.shader) {
                renderer.bind(
                    &mut sdf.pipeline,
                    PipelineLayout::Render {
                        label: String::from(PIPELINE_LABEL),
                        mesh,
                        shader,
                        bindings: &[
                            BindGroup::new(
                                "Globals",
                                vec![
                                    Binding::Uniform("Camera", Stage::All, &camera_buffer.uniform),
                                    Binding::Sampler("Sampler", Stage::Fragment, sampler),
                                    Binding::Storage(
                                        "Lights",
                                        Stage::Fragment,
                                        &lights_buffer.storage,
                                    ),
                                ],
                            ),
                            BindGroup::new(
                                "Locals",
                                vec![
                                    Binding::Uniform("Data", Stage::All, &sdf.data),
                                    Binding::Texture3D("Sdf", Stage::All, &sdf.buffer),
                                    Binding::Uniform(
                                        "Materials",
                                        Stage::All,
                                        material_set.get_material_buffer(),
                                    ),
                                    Binding::TextureArray(
                                        "MaterialTexture",
                                        Stage::All,
                                        material_set.get_texture_buffer(),
                                    ),
                                ],
                            ),
                        ],
                        options: RenderOptions::default(),
                    },
                );
            }
        }

        renderer.draw(&mut sdf.pipeline, mesh, &Default::default());
    }
}

pub(super) fn extension(app: &mut Application) {
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
    camera::extension(app);
    lights::extension(app);
}
