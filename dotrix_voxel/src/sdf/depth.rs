//! Compute the depth of all sdfs onto a texture
//!
//! This uses circle tracing
//!
use super::camera::CameraBuffer;
use crate::{Grid, SdfCalc, TexSdf, VOXEL_TEMPLATES};
use dotrix_core::{
    assets::Shader,
    ecs::{Const, Entity, Mut, System},
    renderer::{
        Access, BindGroup, Binding, ComputeArgs, ComputeOptions, PipelineLayout, Renderer, Stage,
        WorkGroups,
    },
    Application, Assets, Globals, Transform, Window, World,
};
use tera::Context;

mod data;
mod service;

pub use self::data::*;
pub use self::service::*;

const INIT_PIPELINE_LABEL: &str = "dotrix_voxel::sdf::depth_init";
const PIPELINE_LABEL: &str = "dotrix_voxel::sdf::depth";
const PIXELS_PER_WORKGROUP: [usize; 3] = [16, 16, 1];

fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let templates = &VOXEL_TEMPLATES;
    let mut context = Context::new();
    context.insert("dotrix_voxel_camera_group", &0);
    context.insert("dotrix_voxel_camera_binding", &0);
    context.insert("map_data_group", &1);
    context.insert("map_data_binding", &2);
    context.insert("sdf_tex_group", &1);
    context.insert("sdf_tex_binding", &3);
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: templates
            .render("dotrix_voxel/depth/depth.wgsl", &context)
            .unwrap(),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);

    let mut shader = Shader {
        name: String::from(INIT_PIPELINE_LABEL),
        code: templates
            .render("dotrix_voxel/depth/init.wgsl", &context)
            .unwrap(),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, INIT_PIPELINE_LABEL);
}

fn compute(
    sdf_calc: Const<SdfCalc>,
    mut sdf_depth: Mut<SdfDepth>,
    mut sdf_depth_init: Mut<SdfDepthInit>,
    mut renderer: Mut<Renderer>,
    world: Const<World>,
    assets: Const<Assets>,
    window: Const<Window>,
    globals: Const<Globals>,
) {
    let working_scale = sdf_calc.working_scale * sdf_depth.working_scale;
    let buffer_size = {
        let ws = window.inner_size();
        [
            (ws[0] as f32 * working_scale) as u32,
            (ws[1] as f32 * working_scale) as u32,
        ]
    };
    let rebind = sdf_depth.load(&renderer, buffer_size);

    let workgroup_size_x = (buffer_size[0] as f32 / PIXELS_PER_WORKGROUP[0] as f32).ceil() as u32;
    let workgroup_size_y = (buffer_size[0] as f32 / PIXELS_PER_WORKGROUP[1] as f32).ceil() as u32;
    let workgroup_size_z = 1;
    if rebind {
        sdf_depth_init.init_pipeline.bindings.unload();
    }
    if sdf_depth_init.init_pipeline.shader.is_null() {
        sdf_depth_init.init_pipeline.shader = assets
            .find::<Shader>(INIT_PIPELINE_LABEL)
            .unwrap_or_default();
    }
    if !sdf_depth_init.init_pipeline.cycle(&renderer) {
        return;
    }
    if !sdf_depth_init.init_pipeline.ready(&renderer) {
        if let Some(shader) = assets.get(sdf_depth_init.init_pipeline.shader) {
            renderer.bind(
                &mut sdf_depth_init.init_pipeline,
                PipelineLayout::Compute {
                    label: String::from(INIT_PIPELINE_LABEL),
                    shader,
                    bindings: &[BindGroup::new(
                        "Globals",
                        vec![
                            Binding::StorageTexture(
                                "SdfPing",
                                Stage::Compute,
                                &sdf_depth.ping_buffer,
                                Access::WriteOnly,
                            ),
                            Binding::StorageTexture(
                                "SdfPing",
                                Stage::Compute,
                                &sdf_depth.ping_buffer,
                                Access::WriteOnly,
                            ),
                            Binding::StorageTexture(
                                "SdfNormal",
                                Stage::Compute,
                                &sdf_depth.normal_buffer,
                                Access::WriteOnly,
                            ),
                            Binding::StorageTexture(
                                "SdfDepth",
                                Stage::Compute,
                                &sdf_depth.depth_buffer,
                                Access::WriteOnly,
                            ),
                        ],
                    )],
                    options: ComputeOptions::default(),
                },
            );
        }
    }
    if !sdf_depth_init.init_pipeline.ready(&renderer) {
        return;
    }
    renderer.compute(
        &mut sdf_depth_init.init_pipeline,
        &ComputeArgs {
            work_groups: WorkGroups {
                x: workgroup_size_x,
                y: workgroup_size_y,
                z: workgroup_size_z,
            },
        },
    );

    let (mut ping, mut pong) = (&sdf_depth.ping_buffer, &sdf_depth.pong_buffer);

    for (grid, sdf, object_2_world, entity) in
        world.query::<(&Grid, &mut TexSdf, &Transform, &Entity)>()
    {
        if rebind {
            sdf.depth.pipeline.bindings.unload();
        }

        if sdf.depth.pipeline.shader.is_null() {
            sdf.depth.pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }
        if !sdf.depth.pipeline.cycle(&renderer) {
            continue;
        }

        // Perform data updates
        sdf.depth.load(&renderer, &sdf_depth, entity);
        sdf.update(&renderer, grid, object_2_world);

        if !sdf.depth.pipeline.ready(&renderer) {
            // Rebind required
            if let Some(shader) = assets.get(sdf.depth.pipeline.shader) {
                // Shader ready Bind it

                // Get data
                let camera_buffer = globals
                    .get::<CameraBuffer>()
                    .expect("CameraBuffer buffer must be loaded");

                renderer.bind(
                    &mut sdf.depth.pipeline,
                    PipelineLayout::Compute {
                        label: String::from(PIPELINE_LABEL),
                        shader,
                        bindings: &[
                            BindGroup::new(
                                "Globals",
                                vec![Binding::Uniform(
                                    "Camera",
                                    Stage::Compute,
                                    &camera_buffer.uniform,
                                )],
                            ),
                            BindGroup::new(
                                "Locals",
                                vec![
                                    Binding::Uniform("Data", Stage::Compute, &sdf.depth.data),
                                    Binding::Uniform("OBB", Stage::Compute, &sdf.obb_data),
                                    Binding::Uniform("Map", Stage::Compute, &sdf.map_data),
                                    Binding::Texture3D("SdfTex", Stage::Compute, &sdf.buffer),
                                ],
                            ),
                            BindGroup::new(
                                "Shared",
                                vec![
                                    Binding::Texture("Ping", Stage::Compute, ping),
                                    Binding::StorageTexture(
                                        "Pong",
                                        Stage::Compute,
                                        pong,
                                        Access::WriteOnly,
                                    ),
                                    Binding::StorageTexture(
                                        "Normals",
                                        Stage::Compute,
                                        &sdf_depth.normal_buffer,
                                        Access::WriteOnly,
                                    ),
                                    Binding::StorageTexture(
                                        "DepthBuffer",
                                        Stage::Compute,
                                        &sdf_depth.depth_buffer,
                                        Access::WriteOnly,
                                    ),
                                ],
                            ),
                        ],
                        options: ComputeOptions::default(),
                    },
                );
                (ping, pong) = (&sdf_depth.ping_buffer, &sdf_depth.pong_buffer);
            }
        }

        renderer.compute(
            &mut sdf.depth.pipeline,
            &ComputeArgs {
                work_groups: WorkGroups {
                    x: workgroup_size_x,
                    y: workgroup_size_y,
                    z: workgroup_size_z,
                },
            },
        );
    }
}

pub(super) fn extension(app: &mut Application) {
    app.add_service(SdfDepth::default());
    app.add_service(SdfDepthInit::default());
    app.add_system(System::from(startup));
    app.add_system(System::from(compute));
}