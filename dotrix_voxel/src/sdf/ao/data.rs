//! The gpu data related to AO calculations.
//!

use super::service::*;
use dotrix_core::renderer::{Buffer, Pipeline, Renderer};

pub struct AoCalc {
    /// Pipeline for renderering this SDF
    pub pipeline: Pipeline,
    pub data: Buffer,
}

impl Default for AoCalc {
    fn default() -> Self {
        Self {
            pipeline: Default::default(),
            data: Buffer::uniform("Ao Data"),
        }
    }
}

impl AoCalc {
    pub fn load(&mut self, renderer: &Renderer, ao: &SdfAo, number_of_occulders: u32) {
        let data = AoCalcGpu {
            samples: ao.samples,
            steps: ao.steps,
            step_size: ao.step_size,
            number_of_occulders,
        };

        renderer.load_buffer(&mut self.data, bytemuck::cast_slice(&[data]))
    }
}

/// Gpu representation of per run depth calculation data
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(super) struct AoCalcGpu {
    samples: u32,
    steps: u32,
    number_of_occulders: u32,
    step_size: f32,
}

unsafe impl bytemuck::Zeroable for AoCalcGpu {}
unsafe impl bytemuck::Pod for AoCalcGpu {}
