use super::{
    Access, Context, GpuBuffer, GpuTexture, IdOrBuffer, IdOrTexture, PipelineInstance,
    RendererError, Sampler,
};
use crate::{assets::Asset, Assets};

/// Rendering stage
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// Vertex shader stage
    Vertex,
    /// Fragment shader stage
    Fragment,
    /// Compute shader stage
    Compute,
    /// Any stage
    All,
}

impl From<&Stage> for wgpu::ShaderStages {
    fn from(obj: &Stage) -> Self {
        match obj {
            Stage::All => wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            Stage::Vertex => wgpu::ShaderStages::VERTEX,
            Stage::Fragment => wgpu::ShaderStages::FRAGMENT,
            Stage::Compute => wgpu::ShaderStages::COMPUTE,
        }
    }
}

/// Binding types (Label, Stage, Buffer)
pub enum MaybeBinding<'a, Buffer, Texture>
where
    Buffer: IdOrBuffer<'a>,
    Buffer::BaseType: GpuBuffer,
    Texture: IdOrTexture<'a>,
    Texture::BaseType: GpuTexture,
{
    /// Uniform binding
    Uniform(&'a str, Stage, Buffer),
    /// Texture binding
    Texture(&'a str, Stage, Texture),
    /// Cube Texture binding
    TextureCube(&'a str, Stage, Texture),
    /// 2D Texture Array binding
    TextureArray(&'a str, Stage, Texture),
    /// 3D Texture binding
    Texture3D(&'a str, Stage, Texture),
    /// Storage texture binding
    StorageTexture(&'a str, Stage, Texture, Access),
    /// Storage texture cube binding
    StorageTextureCube(&'a str, Stage, Texture, Access),
    /// Storage 2D texture array binding
    StorageTextureArray(&'a str, Stage, Texture, Access),
    /// Storage texture binding 3D
    StorageTexture3D(&'a str, Stage, Texture, Access),
    /// Texture sampler binding
    Sampler(&'a str, Stage, &'a Sampler),
    /// Storage binding
    Storage(&'a str, Stage, Buffer),
}

impl<'a, Buffer, Texture> MaybeBinding<'a, Buffer, Texture>
where
    Buffer: 'a + IdOrBuffer<'a>,
    Buffer::BaseType: GpuBuffer,
    Texture: 'a + IdOrTexture<'a>,
    Texture::BaseType: GpuTexture,
{
    /// Resolve all assets or Error
    pub fn make_concrete<T>(
        &'a self,
        assets: &'a Assets,
    ) -> Result<ConcreteBinding<'a, Buffer::BaseType, Texture::BaseType>, RendererError<T>>
    where
        T: Asset,
        RendererError<T>: std::convert::From<RendererError<Buffer::BaseType>>,
        RendererError<T>: std::convert::From<RendererError<Texture::BaseType>>,
    {
        Ok(match self {
            MaybeBinding::Uniform(label, stage, maybe) => {
                ConcreteBinding::Uniform(label, *stage, maybe.get_asset(assets)?)
            }
            MaybeBinding::Storage(label, stage, maybe) => {
                ConcreteBinding::Storage(label, *stage, maybe.get_asset(assets)?)
            }
            MaybeBinding::Texture(label, stage, maybe) => {
                ConcreteBinding::Texture(label, *stage, maybe.get_asset(assets)?)
            }
            MaybeBinding::TextureCube(label, stage, maybe) => {
                ConcreteBinding::TextureCube(label, *stage, maybe.get_asset(assets)?)
            }
            MaybeBinding::TextureArray(label, stage, maybe) => {
                ConcreteBinding::TextureArray(label, *stage, maybe.get_asset(assets)?)
            }
            MaybeBinding::Texture3D(label, stage, maybe) => {
                ConcreteBinding::Texture3D(label, *stage, maybe.get_asset(assets)?)
            }
            MaybeBinding::StorageTexture(label, stage, maybe, access) => {
                ConcreteBinding::StorageTexture(label, *stage, maybe.get_asset(assets)?, *access)
            }
            MaybeBinding::StorageTextureCube(label, stage, maybe, access) => {
                ConcreteBinding::StorageTextureCube(
                    label,
                    *stage,
                    maybe.get_asset(assets)?,
                    *access,
                )
            }
            MaybeBinding::StorageTextureArray(label, stage, maybe, access) => {
                ConcreteBinding::StorageTextureArray(
                    label,
                    *stage,
                    maybe.get_asset(assets)?,
                    *access,
                )
            }
            MaybeBinding::StorageTexture3D(label, stage, maybe, access) => {
                ConcreteBinding::StorageTexture3D(label, *stage, maybe.get_asset(assets)?, *access)
            }
            MaybeBinding::Sampler(label, stage, concrete) => {
                ConcreteBinding::Sampler(label, *stage, concrete)
            }
        })
    }
}

/// Bind Group holding bindings
pub struct MaybeBindGroup<'a, Buffer, Texture>
where
    Buffer: IdOrBuffer<'a>,
    Buffer::BaseType: GpuBuffer,
    Texture: IdOrTexture<'a>,
    Texture::BaseType: GpuTexture,
{
    /// Text label of the Bind group
    pub label: &'a str,
    /// List of bindings
    pub bindings: Vec<MaybeBinding<'a, Buffer, Texture>>,
}

impl<'a, Buffer, Texture> MaybeBindGroup<'a, Buffer, Texture>
where
    Buffer: 'a + IdOrBuffer<'a>,
    Buffer::BaseType: GpuBuffer,
    Texture: 'a + IdOrTexture<'a>,
    Texture::BaseType: GpuTexture,
{
    /// Resolve all assets or Error
    pub fn make_concrete<T>(
        &'a self,
        assets: &'a Assets,
    ) -> Result<ConcreteBindGroup<'a, Buffer::BaseType, Texture::BaseType>, RendererError<T>>
    where
        T: Asset,
        RendererError<T>: std::convert::From<RendererError<Buffer::BaseType>>,
        RendererError<T>: std::convert::From<RendererError<Texture::BaseType>>,
    {
        Ok(ConcreteBindGroup {
            label: self.label,
            bindings: self
                .bindings
                .iter()
                .map(|maybe| maybe.make_concrete(assets))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Binding with all Id's resolved to Assets
pub enum ConcreteBinding<'a, Buffer, Texture>
where
    Buffer: GpuBuffer,
    Texture: GpuTexture,
{
    /// Uniform binding
    Uniform(&'a str, Stage, &'a Buffer),
    /// Texture binding
    Texture(&'a str, Stage, &'a Texture),
    /// Cube Texture binding
    TextureCube(&'a str, Stage, &'a Texture),
    /// 2D Texture Array binding
    TextureArray(&'a str, Stage, &'a Texture),
    /// 3D Texture binding
    Texture3D(&'a str, Stage, &'a Texture),
    /// Storage texture binding
    StorageTexture(&'a str, Stage, &'a Texture, Access),
    /// Storage texture cube binding
    StorageTextureCube(&'a str, Stage, &'a Texture, Access),
    /// Storage 2D texture array binding
    StorageTextureArray(&'a str, Stage, &'a Texture, Access),
    /// Storage texture binding 3D
    StorageTexture3D(&'a str, Stage, &'a Texture, Access),
    /// Texture sampler binding
    Sampler(&'a str, Stage, &'a Sampler),
    /// Storage binding
    Storage(&'a str, Stage, &'a Buffer),
}

/// Bind Group holding bindings with Assets and not Ids
pub struct ConcreteBindGroup<'a, Buffer, Texture>
where
    Buffer: GpuBuffer,
    Texture: GpuTexture,
{
    /// Text label of the Bind group
    pub label: &'a str,
    /// List of bindings
    pub bindings: Vec<ConcreteBinding<'a, Buffer, Texture>>,
}

impl<'a, Buffer, Texture> ConcreteBindGroup<'a, Buffer, Texture>
where
    Buffer: GpuBuffer,
    Texture: GpuTexture,
{
    /// Constructs new Bind Group
    pub fn new(label: &'a str, bindings: Vec<ConcreteBinding<'a, Buffer, Texture>>) -> Self {
        Self { label, bindings }
    }

    /// Constructs WGPU BindGroupLayout for the `BindGroup`
    pub fn layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entries = self
            .bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| match binding {
                ConcreteBinding::Uniform(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                ConcreteBinding::Texture(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                ConcreteBinding::TextureCube(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::Cube,
                    },
                    count: None,
                },
                ConcreteBinding::TextureArray(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                ConcreteBinding::Texture3D(_, stage, texture) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: texture.get_texture().sample_type(),
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
                ConcreteBinding::StorageTexture(_, stage, texture, access) => {
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::StorageTexture {
                            access: access.into(),
                            format: texture.get_texture().format,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    }
                }
                ConcreteBinding::StorageTextureCube(_, stage, texture, access) => {
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::StorageTexture {
                            access: access.into(),
                            format: texture.get_texture().format,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                        },
                        count: None,
                    }
                }
                ConcreteBinding::StorageTextureArray(_, stage, texture, access) => {
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::StorageTexture {
                            access: access.into(),
                            format: texture.get_texture().format,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    }
                }
                ConcreteBinding::StorageTexture3D(_, stage, texture, access) => {
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::StorageTexture {
                            access: access.into(),
                            format: texture.get_texture().format,
                            view_dimension: wgpu::TextureViewDimension::D3,
                        },
                        count: None,
                    }
                }
                ConcreteBinding::Sampler(_, stage, _) => wgpu::BindGroupLayoutEntry {
                    binding: index as u32,
                    visibility: stage.into(),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                ConcreteBinding::Storage(_, stage, storage) => {
                    let read_only = storage.get_buffer().can_write();
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: stage.into(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                }
            })
            .collect::<Vec<_>>();

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(self.label),
            entries: entries.as_slice(),
        })
    }
}

/// Pipeline Bindings
#[derive(Default)]
pub struct Bindings {
    /// List of `wgpu::BindGroup`
    pub wgpu_bind_groups: Vec<wgpu::BindGroup>,
}

impl Bindings {
    pub(crate) fn load<Buffer, Texture>(
        &mut self,
        ctx: &Context,
        pipeline_instance: &PipelineInstance,
        bind_groups: &[ConcreteBindGroup<Buffer, Texture>],
    ) where
        Buffer: GpuBuffer,
        Texture: GpuTexture,
    {
        let wgpu_bind_groups_layout = match pipeline_instance {
            PipelineInstance::Render(render) => &render.wgpu_bind_groups_layout,
            PipelineInstance::Compute(compute) => &compute.wgpu_bind_groups_layout,
        };
        self.wgpu_bind_groups = wgpu_bind_groups_layout
            .iter()
            .enumerate()
            .map(|(group, wgpu_bind_group_layout)| {
                ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: wgpu_bind_group_layout,
                    entries: bind_groups[group]
                        .bindings
                        .iter()
                        .enumerate()
                        .map(|(binding, entry)| wgpu::BindGroupEntry {
                            binding: binding as u32,
                            resource: match entry {
                                ConcreteBinding::Uniform(_, _, uniform) => {
                                    uniform.get_buffer().get().as_entire_binding()
                                }
                                ConcreteBinding::Texture(_, _, texture)
                                | ConcreteBinding::TextureCube(_, _, texture)
                                | ConcreteBinding::TextureArray(_, _, texture)
                                | ConcreteBinding::Texture3D(_, _, texture)
                                | ConcreteBinding::StorageTexture(_, _, texture, _)
                                | ConcreteBinding::StorageTextureCube(_, _, texture, _)
                                | ConcreteBinding::StorageTextureArray(_, _, texture, _)
                                | ConcreteBinding::StorageTexture3D(_, _, texture, _) => {
                                    wgpu::BindingResource::TextureView(texture.get_texture().get())
                                }
                                ConcreteBinding::Sampler(_, _, sampler) => {
                                    wgpu::BindingResource::Sampler(sampler.get())
                                }
                                ConcreteBinding::Storage(_, _, storage) => {
                                    storage.get_buffer().get().as_entire_binding()
                                }
                            },
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    label: None,
                })
            })
            .collect::<Vec<_>>();
    }

    /// Returns true if bindings was loaded to GPU
    pub fn loaded(&self) -> bool {
        !self.wgpu_bind_groups.is_empty()
    }

    /// Unloads bindings from GPU
    pub fn unload(&mut self) {
        self.wgpu_bind_groups.clear();
    }
}
