//! The service that controls the general behaviour of the SDF
//! calcalation
//!

use dotrix_core::Application;

use super::depth::SdfDepth;

pub struct SdfCalc {
    /// Depth calcaulation specific settings
    pub depth: SdfDepth,
    /// The scale at which the computation operates at fractions of
    /// screen size.
    ///
    /// Making this smaller will increase render speed at a loss of
    /// percision
    ///
    /// Values greater than 1.0 will mean multiple rays per screen pixel
    /// which is often superflous
    ///
    /// Regardless of working scale the final image will be resized to
    /// screen buffer size with an appropiate scaling filter
    pub working_scale: f32,
}

impl Default for SdfCalc {
    fn default() -> Self {
        Self {
            depth: Default::default(),
            working_scale: 0.2,
        }
    }
}

pub(super) fn extension(app: &mut Application) {
    app.add_service(SdfCalc::default());
}
