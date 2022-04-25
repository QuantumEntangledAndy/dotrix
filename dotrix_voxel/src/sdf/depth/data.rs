//! Holds per object data required for the depth trace
//!
//!

use super::service::*;
use dotrix_core::{
    ecs::Entity,
    renderer::{Buffer, Pipeline, Renderer},
};

pub struct DepthCalc {
    /// Pipeline for renderering this SDF
    pub pipeline: Pipeline,
    pub data: Buffer,
}

impl Default for DepthCalc {
    fn default() -> Self {
        Self {
            pipeline: Default::default(),
            data: Buffer::uniform("Depth Data"),
        }
    }
}

impl DepthCalc {
    pub fn load(&mut self, renderer: &Renderer, depth: &SdfDepth, entity: &Entity) {
        let data = DepthCalcGpu {
            object_id: u64::from(entity) as u32,
            max_iterations: depth.max_iteration,
            padding: Default::default(),
        };

        renderer.load_buffer(&mut self.data, bytemuck::cast_slice(&[data]))
    }
}

/// Gpu representation of per run depth calculation data
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(super) struct DepthCalcGpu {
    object_id: u32,
    max_iterations: u32,
    padding: [u32; 2],
}

unsafe impl bytemuck::Zeroable for DepthCalcGpu {}
unsafe impl bytemuck::Pod for DepthCalcGpu {}
