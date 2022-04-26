//! The global data and settings for the AO calculations
//!

use dotrix_core::renderer::{Pipeline, Renderer, Texture as TextureBuffer};

#[derive(Default)]
pub(crate) struct SdfAoInit {
    pub(crate) init_pipeline: Pipeline,
}

/// Global Data for depth calculations.
pub struct SdfAo {
    // The size of the buffer
    pub(crate) buffer_size: [u32; 2],
    /// The ping buffer
    pub(crate) ping_buffer: TextureBuffer,
    /// The pong buffer
    pub(crate) pong_buffer: TextureBuffer,
    /// Final ao texture
    pub(crate) ao_buffer: TextureBuffer,
    /// The number of AO sampling rays to cast
    pub samples: u32,
    /// The number of steps to march on the AO sampling ray
    pub steps: u32,
    /// The size of each step to march on the AO sampling ray
    pub step_size: f32,
    /// Scale factor for the depth trace
    ///
    /// This controls the number of rays that will be cast
    /// A number of 1. will mean the same scale as that in `[crate::SdfCalc::working_scale]`
    pub working_scale: f32,
}

impl Default for SdfAo {
    fn default() -> Self {
        Self {
            buffer_size: Default::default(),
            ping_buffer: {
                let mut buffer = TextureBuffer::new("SdfAoPing")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::R32Float;
                buffer
            },
            pong_buffer: {
                let mut buffer = TextureBuffer::new("SdfAoPong")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::R32Float;
                buffer
            },
            ao_buffer: {
                let mut buffer = TextureBuffer::new("SdfAo").use_as_storage().allow_write();
                buffer.format = wgpu::TextureFormat::R32Float;
                buffer
            },
            samples: 16,
            steps: 8,
            step_size: 0.1,
            working_scale: 0.25,
        }
    }
}

impl SdfAo {
    pub fn load(&mut self, renderer: &Renderer, buffer_size: [u32; 2]) -> bool {
        let reload = buffer_size[0] != self.buffer_size[0] || buffer_size[1] != self.buffer_size[1];
        if reload {
            self.ping_buffer.unload();
            self.pong_buffer.unload();
            self.ao_buffer.unload();
            self.buffer_size = buffer_size;

            let data: Vec<u8> =
                vec![[f32::MAX.to_le_bytes()]; buffer_size[0] as usize * buffer_size[1] as usize]
                    .iter()
                    .flatten()
                    .flatten()
                    .copied()
                    .collect();

            renderer.update_or_load_texture(
                &mut self.ping_buffer,
                buffer_size[0],
                buffer_size[1],
                &[data.as_slice()],
            );
            renderer.update_or_load_texture(
                &mut self.pong_buffer,
                buffer_size[0],
                buffer_size[1],
                &[data.as_slice()],
            );
            renderer.update_or_load_texture(
                &mut self.ao_buffer,
                buffer_size[0],
                buffer_size[1],
                &[data.as_slice()],
            );
        }

        reload
    }
}
