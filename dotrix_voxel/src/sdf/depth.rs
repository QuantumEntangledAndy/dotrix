//! Compute the depth of all sdfs onto a texture
//!
//! This uses circle tracing
//!
use crate::{Grid, TexSdf};
use dotrix_core::{
    assets::Shader,
    ecs::{Const, Mut, System},
    renderer::{Buffer, Pipeline, Renderer, Texture as TextureBuffer},
    Application, Assets, Globals, Transform, Window, World,
};
use tera::{Context, Tera};

// The scale at which the computation operates at fractions of
// screen size.
//
// Making this smaller will increase render speed at a loss of
// percision
//
// Values greater than 1.0 will mean multiple rays per screen pixel
// which is often superflous
//
// Regardless of working scale the final image will be resized to
// screen buffer with an appropiate scaling filter
const WORKING_SCALE: f32 = 1.0;

const PIPELINE_LABEL: &str = "dotrix_voxel::sdf::depth";

/// Data for depth calculations.
pub struct SdfDepth {
    // The size of the buffer
    buffer_size: [u32; 2],
    ping_buffer: TextureBuffer,
    pong_buffer: TextureBuffer,
}

impl Default for SdfDepth {
    fn default() -> Self {
        Self {
            buffer_size: Default::default(),
            ping_buffer: {
                let mut buffer = TextureBuffer::new("SdfDepthPing")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::Rg32Float;
                buffer
            },
            pong_buffer: {
                let mut buffer = TextureBuffer::new("SdfDepthPong")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::Rg32Float;
                buffer
            },
        }
    }
}

fn startup(renderer: Const<Renderer>, mut assets: Mut<Assets>) {
    let mut templates = Tera::default();
    templates
        .add_raw_templates(vec![
            (
                "circle_trace/depth.wgsl",
                include_str!("./depth/depth.wgsl"),
            ),
            (
                "circle_trace/map.inc.wgsl",
                include_str!("./circle_trace/map.inc.wgsl"),
            ),
            (
                "circle_trace/accelerated_raytrace.inc.wgsl",
                include_str!("./circle_trace/accelerated_raytrace.inc.wgsl"),
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
}

fn compute(
    mut sdf_depth: Mut<SdfDepth>,
    mut renderer: Mut<Renderer>,
    world: Const<World>,
    assets: Const<Assets>,
    window: Const<Window>,
) {
    let mut rebind = false;
    let buffer_size = {
        let ws = window.inner_size();
        [
            (ws[0] as f32 * WORKING_SCALE) as u32,
            (ws[1] as f32 * WORKING_SCALE) as u32,
        ]
    };
    if buffer_size[0] != sdf_depth.buffer_size[0] || buffer_size[1] != sdf_depth.buffer_size[1] {
        rebind = true;
        sdf_depth.buffer_size = buffer_size;
    }

    for (grid, sdf, world_transform) in world.query::<(&Grid, &mut TexSdf, &Transform)>() {
        if rebind {
            sdf.depth_pipeline.bindings.unload();
        }

        if sdf.depth_pipeline.shader.is_null() {
            sdf.depth_pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
        }
        if !sdf.depth_pipeline.cycle(&renderer) {
            return;
        }
    }
}

pub(super) fn extension(app: &mut Application) {
    app.add_service(SdfDepth::default());
    app.add_system(System::from(startup));
    app.add_system(System::from(compute));
}
