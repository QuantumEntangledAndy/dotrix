//! Compute the SDF ambient occulsion in a compute shader
//!
use super::camera::CameraBuffer;
use crate::{Grid, SdfCalc, SdfDepth, TexSdf, VOXEL_TEMPLATES};
use dotrix_core::{
    assets::Shader,
    ecs::{Const, Mut, System},
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

const INIT_PIPELINE_LABEL: &str = "dotrix_voxel::sdf::ao_init";
const PIPELINE_LABEL: &str = "dotrix_voxel::sdf::ao";
const PIXELS_PER_WORKGROUP: [usize; 3] = [16, 16, 1];

fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let templates = &VOXEL_TEMPLATES;
    let mut context = Context::new();
    context.insert("dotrix_voxel_camera_group", &0);
    context.insert("dotrix_voxel_camera_binding", &0);
    context.insert("map_data_group", &1);
    context.insert("map_data_binding", &4);
    context.insert("sdf_tex_group", &1);
    context.insert("sdf_tex_binding", &5);
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: templates
            .render("dotrix_voxel/ao/ao.wgsl", &context)
            .unwrap(),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);

    let mut shader = Shader {
        name: String::from(INIT_PIPELINE_LABEL),
        code: templates
            .render("dotrix_voxel/ao/init.wgsl", &context)
            .unwrap(),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, INIT_PIPELINE_LABEL);
}

fn compute(
    sdf_calc: Const<SdfCalc>,
    sdf_depth: Const<SdfDepth>,
    mut sdf_ao: Mut<SdfAo>,
    mut sdf_ao_init: Mut<SdfAoInit>,
    mut renderer: Mut<Renderer>,
    world: Const<World>,
    assets: Const<Assets>,
    window: Const<Window>,
    globals: Const<Globals>,
) {
    let working_scale = sdf_calc.working_scale * sdf_ao.working_scale;
    let buffer_size = {
        let ws = window.inner_size();
        [
            (ws[0] as f32 * working_scale) as u32,
            (ws[1] as f32 * working_scale) as u32,
        ]
    };
    let rebind = sdf_ao.load(&renderer, buffer_size);

    let workgroup_size_x = (buffer_size[0] as f32 / PIXELS_PER_WORKGROUP[0] as f32).ceil() as u32;
    let workgroup_size_y = (buffer_size[0] as f32 / PIXELS_PER_WORKGROUP[1] as f32).ceil() as u32;
    let workgroup_size_z = 1;
    if rebind {
        sdf_ao_init.init_pipeline.bindings.unload();
    }
    if sdf_ao_init.init_pipeline.shader.is_null() {
        sdf_ao_init.init_pipeline.shader = assets
            .find::<Shader>(INIT_PIPELINE_LABEL)
            .unwrap_or_default();
    }
    if !sdf_ao_init.init_pipeline.cycle(&renderer) {
        return;
    }
    if !sdf_ao_init.init_pipeline.ready(&renderer) {
        if let Some(shader) = assets.get(sdf_ao_init.init_pipeline.shader) {
            renderer.bind(
                &mut sdf_ao_init.init_pipeline,
                PipelineLayout::Compute {
                    label: String::from(INIT_PIPELINE_LABEL),
                    shader,
                    bindings: &[BindGroup::new(
                        "Globals",
                        vec![
                            Binding::StorageTexture(
                                "SdfPing",
                                Stage::Compute,
                                &sdf_ao.ping_buffer,
                                Access::WriteOnly,
                            ),
                            Binding::StorageTexture(
                                "SdfPing",
                                Stage::Compute,
                                &sdf_ao.ping_buffer,
                                Access::WriteOnly,
                            ),
                            Binding::StorageTexture(
                                "SdfNormal",
                                Stage::Compute,
                                &sdf_ao.ao_buffer,
                                Access::WriteOnly,
                            ),
                        ],
                    )],
                    options: ComputeOptions::default(),
                },
            );
        }
    }
    if !sdf_ao_init.init_pipeline.ready(&renderer) {
        return;
    }
    renderer.compute(
        &mut sdf_ao_init.init_pipeline,
        &ComputeArgs {
            work_groups: WorkGroups {
                x: workgroup_size_x,
                y: workgroup_size_y,
                z: workgroup_size_z,
            },
        },
    );

    let (mut ping, mut pong) = (&sdf_ao.ping_buffer, &sdf_ao.pong_buffer);

    let number_of_occulders = world.query::<(&Grid, &mut TexSdf, &Transform)>().count();

    for (grid, sdf, object_2_world) in world.query::<(&Grid, &mut TexSdf, &Transform)>() {
        if rebind {
            sdf.ao.pipeline.bindings.unload();
        }

        if sdf.ao.pipeline.shader.is_null() {
            sdf.ao.pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }
        if !sdf.ao.pipeline.cycle(&renderer) {
            continue;
        }

        // Perform data updates
        sdf.ao.load(&renderer, &sdf_ao, number_of_occulders as u32);
        sdf.update(&renderer, grid, object_2_world);

        if !sdf.ao.pipeline.ready(&renderer) {
            // Rebind required
            if let Some(shader) = assets.get(sdf.ao.pipeline.shader) {
                // Shader ready Bind it

                // Get data
                let camera_buffer = globals
                    .get::<CameraBuffer>()
                    .expect("CameraBuffer buffer must be loaded");

                renderer.bind(
                    &mut sdf.ao.pipeline,
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
                                    Binding::Uniform("Data", Stage::Compute, &sdf.ao.data),
                                    Binding::Uniform("OBB", Stage::Compute, &sdf.obb_data),
                                    Binding::Texture(
                                        "Depth",
                                        Stage::Compute,
                                        &sdf_depth.depth_buffer,
                                    ),
                                    Binding::Texture(
                                        "Normals",
                                        Stage::Compute,
                                        &sdf_depth.normal_buffer,
                                    ),
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
                                        "AoBuffer",
                                        Stage::Compute,
                                        &sdf_ao.ao_buffer,
                                        Access::WriteOnly,
                                    ),
                                ],
                            ),
                        ],
                        options: ComputeOptions::default(),
                    },
                );
                (ping, pong) = (&sdf_ao.ping_buffer, &sdf_ao.pong_buffer);
            }
        }

        renderer.compute(
            &mut sdf.ao.pipeline,
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
    app.add_service(SdfAo::default());
    app.add_service(SdfAoInit::default());
    app.add_system(System::from(startup));
    app.add_system(System::from(compute));
}
