//! Holds gobal service data for the depth trace
//!

use dotrix_core::renderer::{Pipeline, Renderer, Texture as TextureBuffer};

#[derive(Default)]
pub(crate) struct SdfDepthInit {
    pub(crate) init_pipeline: Pipeline,
}

/// Global Data for depth calculations.
pub struct SdfDepth {
    // The size of the buffer
    pub(super) buffer_size: [u32; 2],
    /// The ping buffer
    pub(super) ping_buffer: TextureBuffer,
    /// The pong buffer
    pub(super) pong_buffer: TextureBuffer,
    /// Final depth texture
    pub(crate) depth_buffer: TextureBuffer,
    /// Final normals
    pub(crate) normal_buffer: TextureBuffer,
    /// The pipeline used to init the ping/pong and normal buffers
    /// with default values on the gpu
    pub max_iteration: u32,
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
            depth_buffer: {
                let mut buffer = TextureBuffer::new("SdfDepth")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::Rg32Float;
                buffer
            },
            normal_buffer: {
                let mut buffer = TextureBuffer::new("SdfNormals")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::Rgba32Float;
                buffer
            },

            max_iteration: 128,
        }
    }
}

impl SdfDepth {
    pub fn load(&mut self, renderer: &Renderer, buffer_size: [u32; 2]) -> bool {
        let reload = buffer_size[0] != self.buffer_size[0] || buffer_size[1] != self.buffer_size[1];
        if reload {
            self.ping_buffer.unload();
            self.pong_buffer.unload();
            self.depth_buffer.unload();
            self.normal_buffer.unload();
            self.buffer_size = buffer_size;

            let data: Vec<u8> = vec![
                [f32::MAX.to_le_bytes(), (-1f32).to_le_bytes()];
                buffer_size[0] as usize * buffer_size[1] as usize
            ]
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
                &mut self.depth_buffer,
                buffer_size[0],
                buffer_size[1],
                &[data.as_slice()],
            );

            let data: Vec<u8> = vec![
                [
                    0f32.to_le_bytes(),
                    0f32.to_le_bytes(),
                    0f32.to_le_bytes(),
                    0f32.to_le_bytes()
                ];
                buffer_size[0] as usize * buffer_size[1] as usize
            ]
            .iter()
            .flatten()
            .flatten()
            .copied()
            .collect();
            renderer.update_or_load_texture(
                &mut self.normal_buffer,
                buffer_size[0],
                buffer_size[1],
                &[data.as_slice()],
            );
        }

        reload
    }
}
