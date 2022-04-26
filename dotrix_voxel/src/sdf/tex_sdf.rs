use super::ao::AoCalc;
use super::depth::DepthCalc;
use crate::{Grid, Obb};
use dotrix_core::{
    renderer::{Buffer, Renderer, Texture as TextureBuffer},
    Transform,
};
use dotrix_math::Vec3;

/// GPU data for rendering as a texure based sdf
pub struct TexSdf {
    /// Texture buffer containing a 3d texture
    /// with r channel of the distance anf g channel of the material ID
    pub buffer: TextureBuffer,
    /// Uniform that holds oriented bounding box data
    pub obb_data: Buffer,
    // Uniform that holds the grid related data
    pub map_data: Buffer,
    /// Depth Data
    /// Per object data used in the depth calculation
    pub depth: DepthCalc,
    /// Per object data used in the ao calculation
    pub ao: AoCalc,
}

impl Default for TexSdf {
    fn default() -> Self {
        Self {
            buffer: {
                let mut buffer = TextureBuffer::new_3d("TexSDF")
                    .use_as_storage()
                    .allow_write();
                buffer.format = wgpu::TextureFormat::Rg32Float;
                buffer
            },
            depth: Default::default(),
            ao: Default::default(),
            obb_data: Buffer::uniform("TexSdf OBB"),
            map_data: Buffer::uniform("TexSdf MapData"),
        }
    }
}

impl TexSdf {
    pub fn clear_texture(&mut self, renderer: &Renderer, dimensions: &[u32; 3]) {
        let pixel_size = 4 * 2;
        let data: Vec<Vec<u8>> = vec![
            0u8;
            pixel_size
                * dimensions[0] as usize
                * dimensions[1] as usize
                * dimensions[2] as usize
        ]
        .chunks(dimensions[0] as usize * dimensions[1] as usize * pixel_size)
        .map(|chunk| chunk.to_vec())
        .collect();

        let slices: Vec<&[u8]> = data.iter().map(|chunk| chunk.as_slice()).collect();

        renderer.update_or_load_texture(
            &mut self.buffer,
            dimensions[0],
            dimensions[1],
            slices.as_slice(),
        );
    }

    /// Update as much data as possible prior to using it
    pub fn update(&mut self, renderer: &Renderer, grid: &Grid, transform: &Transform) {
        self.update_obb(renderer, grid, transform);
        self.update_map_data(renderer, grid, transform);
    }

    pub fn update_obb(&mut self, renderer: &Renderer, grid: &Grid, transform: &Transform) {
        let obb: Obb = Obb::from_transform(transform.matrix(), Vec3::from(grid.get_size()) / 2.);
        obb.load(renderer, &mut self.obb_data);
    }

    pub fn update_map_data(&mut self, renderer: &Renderer, grid: &Grid, transform: &Transform) {
        super::map_data::MapGpu::load(renderer, &mut self.map_data, grid, transform);
    }
}
