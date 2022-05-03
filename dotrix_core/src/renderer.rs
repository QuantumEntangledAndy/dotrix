//! Rendering service and systems
mod access;
mod bindings;
mod buffer;
mod context;
mod mesh;
mod pipelines;
mod sampler;
mod shader;
mod texture;

use dotrix_math::{Mat4, Vec2};

use crate::{
    assets::{Asset, Shader},
    ecs::{Const, Mut},
    providers::{BufferProvider, MeshProvider, TextureProvider},
    reloadable::{ReloadKind, Reloadable},
    Assets, Color, Globals, Id, Window,
};

pub use access::Access;
pub use bindings::{
    Bindings, ConcreteBindGroup, ConcreteBinding, MaybeBindGroup, MaybeBinding, Stage,
};
pub use buffer::Buffer;
pub use context::Context;
pub use mesh::AttributeFormat;
pub use pipelines::{
    Compute, ComputeArgs, ComputeOptions, DepthBufferMode, DrawArgs, Pipeline, PipelineInstance,
    PipelineLayout, Render, RenderOptions, ScissorsRect, WorkGroups,
};
pub use sampler::Sampler;
pub use shader::ShaderModule;
pub use texture::Texture;

// Ree-export native wgpu module
pub use wgpu;

/// Conversion matrix
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

const RENDERER_STARTUP: &str =
    "Please, use `renderer::startup` as a first system on the `startup` run level";

use thiserror::Error;

#[derive(Error, Debug)]
/// Errors generated during binding/drawing/computing
pub enum RendererError<T: Asset> {
    /// When an asset returns a null id at draw/compute time
    #[error("Asset not ready")]
    AssetNotReady(Id<T>),
    /// When the draw/compute call fails due to pipeline not ready
    #[error("Pipeline not ready")]
    PipelineNotReady,
    /// When the draw/compute call fails due to shader not ready
    #[error("Shader not ready")]
    ShaderNotReady,
}

/// Collection of traits that a gpu buffer needs
pub trait GpuBuffer: Reloadable + BufferProvider + Asset {}
impl<T: Reloadable + BufferProvider + Asset> GpuBuffer for T {}
/// Collection of traits that a gpu texture needs
pub trait GpuTexture: Reloadable + TextureProvider + Asset {}
impl<T: Reloadable + TextureProvider + Asset> GpuTexture for T {}
/// Collection of traits that a gpu mesh needs
pub trait GpuMesh: Reloadable + MeshProvider + Asset {}
impl<T: Reloadable + MeshProvider + Asset> GpuMesh for T {}

/// Used to either get a Mesh directly or convert an Id to an asset
///
/// This is used so that BindGroup can accept either

macro_rules! impl_idor {
    ($i: ident, $n:ident) => {
        /// Trait used either to accept an Asset or an Id of an Asset
        pub trait $n<'a>
        where
            Self::BaseType: Asset + $i,
        {
            /// The underlying asset type
            type BaseType;

            /// Get a ref to the asset
            fn get_asset(
                &'a self,
                assets: &'a Assets,
            ) -> Result<&'a Self::BaseType, RendererError<Self::BaseType>>;
        }

        /// No Op varient when it is already an Asset
        impl<'a, T> $n<'a> for &'a T
        where
            T: Asset + $i,
            RendererError<T>: std::convert::From<RendererError<T>>,
        {
            type BaseType = T;
            fn get_asset(
                &'a self,
                _assets: &'a Assets,
            ) -> Result<&'a Self::BaseType, RendererError<Self::BaseType>> {
                Ok(self)
            }
        }

        /// When it is an Id
        impl<'a, T> $n<'a> for Id<T>
        where
            T: Asset + $i,
            RendererError<T>: std::convert::From<RendererError<T>>,
        {
            type BaseType = T;

            fn get_asset(
                &'a self,
                assets: &'a Assets,
            ) -> Result<&'a Self::BaseType, RendererError<Self::BaseType>> {
                assets
                    .get::<Self::BaseType>(*self)
                    .ok_or_else(|| RendererError::AssetNotReady(*self))
            }
        }
    };
}

impl_idor!(GpuMesh, IdOrMesh);
impl_idor!(GpuBuffer, IdOrBuffer);
impl_idor!(GpuTexture, IdOrTexture);

/// Service providing an interface to `WGPU` and `WINIT`
pub struct Renderer {
    /// Surface clear color
    pub clear_color: Color,
    /// Auto-incrementing rendering cylce
    pub cycle: usize,
    /// Antialiasing
    pub antialiasing: Antialiasing,
    /// Low-level rendering context
    pub context: Option<Context>,
    /// When dirty, renderer will try to load missing pipelines on frame binding
    pub dirty: bool,
}

impl Renderer {
    /// Sets default clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    fn context(&self) -> &Context {
        self.context.as_ref().expect(RENDERER_STARTUP)
    }

    fn context_mut(&mut self) -> &mut Context {
        self.context.as_mut().expect(RENDERER_STARTUP)
    }

    /// Returns the rendering cycle number (Experimental)
    pub fn cycle(&self) -> usize {
        self.cycle
    }

    /// Laods the vertex buffer to GPU
    /*
    pub fn load_mesh<'a>(
        &self,
        buffer: &mut VertexBuffer,
        attributes: &'a [u8],
        indices: Option<&'a [u8]>,
        count: usize,
    ) {
        buffer.load(self.context(), attributes, indices, count as u32);
    }*/

    /// Loads the texture buffer to GPU.
    /// This will recreate the texture, as a result it must be rebound on any pipelines for changes
    /// to take effect
    pub fn load_texture<'a>(
        &self,
        texture: &mut Texture,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        texture.load(self.context(), width, height, layers);
    }

    /// Load data from cpu to a texture buffer on GPU
    /// This is a noop if texture has not been loaded with `load_texture`
    /// Unexpected results/errors occur if the dimensions differs from it dimensions at load time
    pub fn update_texture<'a>(
        &self,
        texture: &mut Texture,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        texture.update(self.context(), width, height, layers);
    }

    /// This will `[update_texture]` if texture has been loaded or `[load_texture]` if not
    /// the same cavets of `[update_texture]` apply in that care must be taken not to change
    /// the dimensions between `load` and `update`
    pub fn update_or_load_texture<'a>(
        &self,
        texture: &mut Texture,
        width: u32,
        height: u32,
        layers: &'a [&'a [u8]],
    ) {
        texture.update_or_load(self.context(), width, height, layers);
    }

    /// Loads the buffer to GPU
    pub fn load_buffer<'a>(&self, buffer: &mut Buffer, data: &'a [u8]) {
        buffer.load(self.context(), data);
    }

    /// Create a buffer on GPU without data
    pub fn create_buffer(&self, buffer: &mut Buffer, size: u32, mapped: bool) {
        buffer.create(self.context(), size, mapped);
    }

    /// Loads the sampler to GPU
    pub fn load_sampler(&self, sampler: &mut Sampler) {
        sampler.load(self.context());
    }

    /// Loads the sahder module to GPU
    pub fn load_shader(&self, shader_module: &mut ShaderModule, code: &str) {
        shader_module.load(self.context(), code);
    }

    /// Copy a texture to a buffer
    pub fn copy_texture_to_buffer(
        &mut self,
        texture: &Texture,
        buffer: &Buffer,
        extent: [u32; 3],
        bytes_per_pixel: u32,
    ) {
        self.context_mut()
            .run_copy_texture_to_buffer(texture, buffer, extent, bytes_per_pixel);
    }

    /// Fetch texture from GPU
    pub fn fetch_texture(
        &mut self,
        texture: &Texture,
        dimensions: [u32; 3],
    ) -> impl std::future::Future<Output = Result<Vec<u8>, wgpu::BufferAsyncError>> {
        texture.fetch_from_gpu(dimensions, self.context_mut())
    }

    /// Forces engine to reload shaders
    pub fn reload(&mut self) {
        self.dirty = true;
    }

    /// Binds uniforms and other data to the pipeline
    ///
    /// This will also create the pipeline instance if it
    /// is not already ready
    pub fn bind<'a, 'b, Buffer, Texture, Mesh>(
        &mut self,
        pipeline: &mut Pipeline,
        layout: PipelineLayout<'a, 'b, Buffer, Texture, Mesh>,
    ) where
        &'a Mesh: GpuMesh,
        Buffer: GpuBuffer,
        Texture: GpuTexture,
    {
        if pipeline.instance.is_none() {
            let instance = layout.instance(self.context());
            pipeline.instance = Some(instance);
        }

        let instance = pipeline.instance.as_ref().unwrap();
        let mut bindings = Bindings::default();
        let bindings_layout = match layout {
            PipelineLayout::Render { bindings, .. } => bindings,
            PipelineLayout::Compute { bindings, .. } => bindings,
        };
        bindings.load(self.context(), instance, bindings_layout);
        pipeline.bindings = bindings;
    }

    /// Runs the render pipeline for a mesh
    pub fn draw<'a, 'b, T, Mesh, Buffer, Texture>(
        &mut self,
        pipeline: &mut Pipeline,
        shader: Id<Shader>,
        mesh_id: &'a Mesh,
        assets: &'a Assets,
        bind_groups: &'a [MaybeBindGroup<'a, Buffer, Texture>],
        args: DrawArgs,
    ) -> Result<(), RendererError<T>>
    where
        Mesh: 'a + IdOrMesh<'a>,
        &'a Mesh::BaseType: GpuMesh,
        Buffer: 'a + IdOrBuffer<'a>,
        Buffer::BaseType: GpuBuffer,
        Texture: 'a + IdOrTexture<'a>,
        Texture::BaseType: GpuTexture,
        T: Asset,
        RendererError<T>: std::convert::From<RendererError<Buffer::BaseType>>,
        RendererError<T>: std::convert::From<RendererError<Mesh::BaseType>>,
        RendererError<T>: std::convert::From<RendererError<Texture::BaseType>>,
        RendererError<T>: std::convert::From<RendererError<Shader>>,
    {
        let mut needs_binding = !pipeline.ready();

        let last_cycle = pipeline.get_cycle();
        pipeline.cycle(self);

        let shader = assets
            .get::<Shader>(shader)
            .ok_or(RendererError::AssetNotReady(shader))?;

        let mesh = mesh_id.get_asset(assets)?;

        if !needs_binding {
            needs_binding = matches!(mesh.changes_since(last_cycle), ReloadKind::Reload);
        }

        // Check all assets to ensure they exist and convert to assets and not IDs
        let bind_groups = bind_groups
            .iter()
            .map(|bind_group| bind_group.make_concrete(assets))
            .collect::<Result<Vec<ConcreteBindGroup<_, _>>, _>>()?;

        // Check if any asset need a reload
        for bind_group in bind_groups.iter() {
            for binding in bind_group.bindings.iter() {
                if !needs_binding {
                    let state = match binding {
                        ConcreteBinding::Uniform(_, _, reloadable)
                        | ConcreteBinding::Storage(_, _, reloadable) => {
                            reloadable.changes_since(last_cycle)
                        }
                        ConcreteBinding::Texture(_, _, reloadable)
                        | ConcreteBinding::TextureCube(_, _, reloadable)
                        | ConcreteBinding::TextureArray(_, _, reloadable)
                        | ConcreteBinding::Texture3D(_, _, reloadable)
                        | ConcreteBinding::StorageTexture(_, _, reloadable, _)
                        | ConcreteBinding::StorageTextureCube(_, _, reloadable, _)
                        | ConcreteBinding::StorageTextureArray(_, _, reloadable, _)
                        | ConcreteBinding::StorageTexture3D(_, _, reloadable, _) => {
                            reloadable.changes_since(last_cycle)
                        }
                        ConcreteBinding::Sampler(_, _, _) => ReloadKind::NoChange,
                    };

                    needs_binding = matches!(state, ReloadKind::Reload);
                }
            }
        }

        if needs_binding {
            pipeline.bindings.unload();
        }

        if !pipeline.ready() {
            if shader.loaded() {
                return Err(RendererError::ShaderNotReady);
            }
            let layout = PipelineLayout::Render {
                label: shader.name.clone(),
                mesh,
                shader,
                bindings: bind_groups.as_slice(),
                options: args.render_options.clone(),
            };
            self.bind(pipeline, layout);
        }
        self.context_mut().run_render_pipeline(
            pipeline.instance.as_ref().unwrap(),
            mesh,
            &pipeline.bindings,
            &args,
        );
        Ok(())
    }

    /// Runs the compute pipeline
    pub fn compute<'a, T, Buffer, Texture>(
        &mut self,
        _pipeline: &mut Pipeline,
        _shader: Id<Shader>,
        _assets: &Assets,
        _bind_groups: &[MaybeBindGroup<'a, Buffer, Texture>],
        _args: &ComputeArgs,
    ) -> Result<(), RendererError<T>>
    where
        Buffer: IdOrBuffer<'a>,
        Buffer::BaseType: GpuBuffer,
        Texture: IdOrTexture<'a>,
        Texture::BaseType: GpuTexture,
        T: Asset,
        RendererError<T>: std::convert::From<RendererError<Buffer::BaseType>>,
        RendererError<T>: std::convert::From<RendererError<Shader>>,
        RendererError<T>: std::convert::From<RendererError<Texture::BaseType>>,
    {
        // self.context_mut()
        //     .run_compute_pipeline(pipeline, shader, assets, bindings, args);
        Ok(())
    }

    /// Returns surface size
    pub fn surface_size(&self) -> Vec2 {
        let ctx = self.context();
        Vec2::new(ctx.sur_desc.width as f32, ctx.sur_desc.height as f32)
    }
}

/// Antialiasing modes enumeration
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Antialiasing {
    /// Enable antialiasing
    Enabled,
    /// Disable antialiasing
    Disabled,
    /// Manual control of number of samples for multisampled antialiasing
    MSAA {
        /// Number od samples for MSAA
        sample_count: u32,
    },
}

impl Antialiasing {
    /// get sample count for the antaliasing mode
    pub fn sample_count(self) -> u32 {
        match self {
            Antialiasing::Enabled => 4,
            Antialiasing::Disabled => 1,
            Antialiasing::MSAA { sample_count } => sample_count,
        }
    }
}

impl Default for Renderer {
    /// Constructs new instance of the service
    fn default() -> Self {
        Renderer {
            clear_color: Color::from([0.1, 0.2, 0.3, 1.0]),
            cycle: 1,
            context: None,
            dirty: true,
            antialiasing: Antialiasing::Enabled,
        }
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

/// Startup system
pub fn startup(mut renderer: Mut<Renderer>, mut globals: Mut<Globals>, window: Mut<Window>) {
    // get sample count
    let sample_count = renderer.antialiasing.sample_count();
    // Init context
    if renderer.context.is_none() {
        renderer.context = Some(futures::executor::block_on(context::init(
            window.get(),
            sample_count,
        )));
    }

    // Create texture sampler and store it with Globals
    let mut sampler = Sampler::default();
    renderer.load_sampler(&mut sampler);
    globals.set(sampler);
}

/// Frame binding system
pub fn bind(mut renderer: Mut<Renderer>, mut assets: Mut<Assets>) {
    let clear_color = renderer.clear_color;
    let sample_count = renderer.antialiasing.sample_count();

    // NOTE: other option here is to check sample_count != context.sample_count
    let reload_request = renderer
        .context_mut()
        .bind_frame(&clear_color, sample_count);

    if !renderer.dirty && !reload_request {
        return;
    }

    let mut loaded = true;

    for (_id, shader) in assets.iter_mut::<Shader>() {
        shader.load(&renderer);
        if !shader.loaded() {
            loaded = false;
        }
    }

    renderer.dirty = !loaded;
}

/// Frame release system
pub fn release(mut renderer: Mut<Renderer>) {
    renderer.context_mut().release_frame();
    renderer.cycle += 1;
    if renderer.cycle == 0 {
        renderer.cycle = 1;
    }
    // Check for resource cleanups and mapping callbacks
    if let Some(context) = renderer.context.as_ref() {
        context.device.poll(wgpu::Maintain::Poll);
    }
}

/// Resize handling system
pub fn resize(mut renderer: Mut<Renderer>, window: Const<Window>) {
    let size = window.inner_size();
    renderer.context_mut().resize(size.x, size.y);
}
