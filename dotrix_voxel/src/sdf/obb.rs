//! Oriented Bounding Box
//!
//! This is a bouding box that has been rotated into another frame
//!
use dotrix_core::renderer::{Buffer, Renderer};
use dotrix_math::*;

/// Oriented boudning box
pub struct Obb {
    /// The box center
    pub center: Vec3,
    /// The half widths/height/depth of the box
    pub half_widths: Vec3,
    /// The three axis that define the orientation
    pub axis: Mat3,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub(super) struct ObbGpu {
    axis: [[f32; 4]; 4], // Use 4x4 due to required 16byte alignments
    center: [f32; 4],
    half_widths: [f32; 4],
}

unsafe impl bytemuck::Zeroable for ObbGpu {}
unsafe impl bytemuck::Pod for ObbGpu {}

impl Obb {
    pub fn from_transform(transform: Mat4, half_widths: Vec3) -> Self {
        let axis_transorm = transform
            .invert()
            .unwrap_or_else(Mat4::identity)
            .transpose();
        let x_axis = (axis_transorm * Vec4::from([1., 0., 0., 0.]))
            .truncate()
            .normalize();
        let y_axis = (axis_transorm * Vec4::from([0., 1., 0., 0.]))
            .truncate()
            .normalize();
        let z_axis = (axis_transorm * Vec4::from([0., 0., 1., 0.]))
            .truncate()
            .normalize();

        let center = (transform * Vec4::from([0., 0., 0., 1.])).truncate();

        Self {
            center,
            half_widths,
            axis: Mat3::from_cols(x_axis, y_axis, z_axis),
        }
    }

    pub fn load(&self, renderer: &Renderer, buffer: &mut Buffer) {
        let gpu_rep = ObbGpu {
            center: self.center.extend(0.).into(),
            half_widths: self.half_widths.extend(0.).into(),
            axis: Mat4::from(self.axis).into(),
        };

        renderer.load_buffer(buffer, bytemuck::cast_slice(&[gpu_rep]));
    }
}
