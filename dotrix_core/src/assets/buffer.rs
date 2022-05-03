//! Buffer asset
//!

use crate::renderer::{Buffer as GpuBuffer, Renderer};
use crate::{ReloadState, Reloadable};
use dotrix_macro::Reloadable;

/// Buffer asset
#[derive(Reloadable)]
pub struct Buffer {
    /// Raw buffer data on the cpu
    ///
    /// This is data that is loaded from the cpu to the gpu but does not
    /// reflect any changes that may happen on the gpu
    pub data: Vec<u8>,
    /// The underlying gpu buffer
    pub buffer: GpuBuffer,
    /// The reload state holds the last cycle that certain changes were made on
    pub reload_state: ReloadState,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            data: vec![],
            buffer: GpuBuffer::new("Buffer"),
            reload_state: Default::default(),
        }
    }
}

impl Buffer {
    /// Loads the [`Buffer`] data to a buffer
    pub fn load(&mut self, renderer: &Renderer) {
        if !self.changed && self.buffer.loaded() {
            return;
        }

        renderer.load_buffer(&mut self.buffer, self.data.as_slice());
    }

    /// Unloads the [`Texture`] data from the buffer
    pub fn unload(&mut self) {
        self.flag_reload();
        self.buffer.unload();
    }
}
