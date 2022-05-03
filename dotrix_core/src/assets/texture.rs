//! Texture asset
use crate::{
    renderer::{Renderer, Texture as TextureBuffer},
    ReloadKind, ReloadState, Reloadable,
};
use dotrix_macros::{Reloadable, TextureProvider};

/// Texture asset
#[derive(Reloadable, TextureProvider)]
pub struct Texture {
    /// Texture width in pixels
    pub width: u32,
    /// Texture height in pixels
    pub height: u32,
    /// Texture depth
    pub depth: u32,
    /// Raw texture data
    pub data: Vec<u8>,
    /// Texture buffer
    pub texture: TextureBuffer,
    /// Flagged on change
    pub reload_state: ReloadState,
    /// Last cycle in which the buffer data was updated
    pub last_load_cycle: usize,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            depth: 0,
            data: vec![],
            texture: TextureBuffer::new("Texture"),
            reload_state: Default::default(),
            last_load_cycle: 0,
        }
    }
}

impl Texture {
    /// Loads the [`Texture`] data to a buffer from the CPU data
    pub fn load(&mut self, renderer: &Renderer) {
        if matches!(
            self.changes_since(self.last_load_cycle),
            ReloadKind::Reload | ReloadKind::Update
        ) || !self.texture.loaded()
        {
            renderer.load_texture(
                &mut self.texture,
                self.width,
                self.height,
                &[self.data.as_slice()],
            );
            self.last_load_cycle = renderer.cycle();
        }
    }

    /// Prepare data into the cpu buffer
    ///
    /// This will flag if the buffer require reloading as appropiate
    pub fn prepare(&mut self, dimension: [u32; 2], data: Vec<u8>) {
        if dimension[0] != self.width
            || dimension[1] != self.height
            || data.len() != self.data.len()
        {
            self.unload();
        } else {
            self.flag_updated();
        }
        self.width = dimension[0];
        self.height = dimension[1];
        self.data = data;
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.texture.unload();
        self.flag_reload();
    }

    /// Fetch data from the gpu
    ///
    /// This is useful textures that are altered on the gpu
    ///
    /// This operation is slow and should mostly be
    /// used for debugging
    pub fn fetch_from_gpu(
        &mut self,
        renderer: &mut Renderer,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, wgpu::BufferAsyncError>> {
        renderer.fetch_texture(&self.texture, [self.width, self.height, self.depth])
    }
}
