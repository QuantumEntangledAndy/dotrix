//! The gpu data related to AO calculations.
//!

use super::service::*;
use dotrix_core::renderer::{Buffer, Pipeline, Renderer};

pub struct ShadowCalc {
    /// Pipeline for renderering this SDF
    pub pipeline: Pipeline,
    pub data: Buffer,
}

impl Default for ShadowCalc {
    fn default() -> Self {
        Self {
            pipeline: Default::default(),
            data: Buffer::uniform("Shadow Data"),
        }
    }
}

impl ShadowCalc {
    pub fn load(&mut self, renderer: &Renderer, shadow: &SdfShadow) {
        let data = ShadowCalcGpu {
            max_iterations: shadow.max_iterations,
            max_probe: shadow.max_probe,
            k: shadow.hardness,
            padding: Default::default(),
        };

        renderer.load_buffer(&mut self.data, bytemuck::cast_slice(&[data]))
    }
}

/// Gpu representation of per run depth calculation data
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(super) struct ShadowCalcGpu {
    max_iterations: u32,
    max_probe: f32,
    k: f32,
    padding: [f32; 1],
}

unsafe impl bytemuck::Zeroable for ShadowCalcGpu {}
unsafe impl bytemuck::Pod for ShadowCalcGpu {}
