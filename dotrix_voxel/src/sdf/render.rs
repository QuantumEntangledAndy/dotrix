//! This handles the rendering of the SDF
//!
//! This is done with a single triangle that is slapped on the front of the camera

//! Component and buffers
use super::SdfAo;
use dotrix_core::{
    assets::{Mesh, Shader},
    ecs::{Const, Mut, System},
    renderer::{BindGroup, Binding, Pipeline, PipelineLayout, RenderOptions, Stage},
    Application, Assets, Renderer,
};
use tera::{Context, Tera};

pub const PIPELINE_LABEL: &str = "dotrix_voxel::sdf::render";

#[derive(Default)]
pub struct PosterWall {
    pub pipeline: Pipeline,
}

/// startup system
pub fn startup(mut assets: Mut<Assets>, renderer: Const<Renderer>) {
    let mut templates = Tera::default();
    templates
        .add_raw_templates(vec![(
            "dotrix_voxel/render/render.wgsl",
            include_str!("./render/render.wgsl"),
        )])
        .unwrap();

    let context = Context::new();
    let mut shader = Shader {
        name: String::from(PIPELINE_LABEL),
        code: templates
            .render("dotrix_voxel/render/render.wgsl", &context)
            .unwrap(),
        ..Default::default()
    };
    shader.load(&renderer);

    assets.store_as(shader, PIPELINE_LABEL);

    let mut mesh = Mesh::default();
    let near_plane = 0.;
    mesh.with_vertices(&[
        [-1., -1., near_plane],
        [-1., 3., near_plane],
        [3., -1., near_plane],
    ]);
    mesh.with_indices(&[0, 2, 1]);
    mesh.load(&renderer);
    assets.store_as(mesh, PIPELINE_LABEL);
}

/// rendering system
pub fn render(
    mut renderer: Mut<Renderer>,
    mut poster_wall: Mut<PosterWall>,
    assets: Const<Assets>,
    ao_sdf: Const<SdfAo>,
) {
    if poster_wall.pipeline.shader.is_null() {
        poster_wall.pipeline.shader = assets.find::<Shader>(PIPELINE_LABEL).unwrap_or_default();
    }

    // check if model is disabled or already rendered
    if !poster_wall.pipeline.cycle(&renderer) {
        return;
    }

    let mesh =
        assets
            .get(assets.find::<Mesh>(PIPELINE_LABEL).expect(
                "PosterWall mesh must be initialized with the `poster_wall::startup` system",
            ))
            .unwrap();

    if !poster_wall.pipeline.ready(&renderer) {
        if let Some(shader) = assets.get(poster_wall.pipeline.shader) {
            renderer.bind(
                &mut poster_wall.pipeline,
                PipelineLayout::Render {
                    label: String::from(PIPELINE_LABEL),
                    mesh,
                    shader,
                    bindings: &[BindGroup::new(
                        "Locals",
                        vec![Binding::Texture(
                            "Texture",
                            Stage::Fragment,
                            &ao_sdf.ao_buffer,
                        )],
                    )],
                    options: RenderOptions::default(),
                },
            );
        }
    }
    // println!("Run Pipeline");
    renderer.draw(&mut poster_wall.pipeline, mesh, &Default::default());
}

pub(super) fn resize(mut data: Mut<PosterWall>) {
    data.pipeline.bindings.unload();
}

pub fn extension(app: &mut Application) {
    app.add_service(PosterWall::default());
    app.add_system(System::from(startup));
    app.add_system(System::from(render));
    app.add_system(System::from(resize));
}
