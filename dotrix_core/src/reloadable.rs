//! When an asset changes it may need to reload
//! and rebind to the pipeline. This Trait is used
//! for signalling such a reload.
//!

use crate::renderer::Renderer;

#[derive(Debug, Clone, Copy)]
/// Denotes the kinds of reload/changes of data that can occur
pub enum ReloadKind {
    /// No change occured in the last load
    NoChange,
    /// Data was updated but a rebind was not required
    Update,
    /// Data was updated in a way that requires a rebind
    Reload,
}

#[derive(Debug, Default)]
/// Used to track the state of reloads
pub struct ReloadState {
    /// The last cycle that an update occured
    ///
    /// This is used to signal that update is required
    /// in assets that are not processed every frame
    pub last_update_cycle: usize,
    /// The last cycle that an reload occured
    ///
    /// This is used to signal that reload is required
    /// in assets that are not processed every frame
    pub last_reload_cycle: usize,
    /// Requires updated this frame
    ///
    /// Unlike `[last_update_cycle]` this does not need to keep track
    /// or the current cycle count and can be used in places that
    /// don't have access to the `[Renderer]`
    pub requires_update: bool,
    /// Requires reload this frame
    ///
    /// Unlike `[last_update_cycle]` this does not need to keep track
    /// or the current cycle count and can be used in places that
    /// don't have access to the `[Renderer]`
    pub requires_reload: bool,
}

/// Describes an object that has gpu represntable data
/// that may update and needs periodic (re-)loading
///
/// A  loading function should call `[flag_updated]` and `[flag_reload]`
/// depending on if an update occured or if a reload is required
pub trait Reloadable {
    /// Get a mutable ref to the `[ReloadState]` that holds relevent reload
    /// data
    fn get_reload_state_mut(&mut self) -> &mut ReloadState;

    /// Get a ref to the `[ReloadState]` that holds relevent reload
    /// data
    fn get_reload_state(&self) -> &ReloadState;

    /// Flag as having data updates in a non rebinding required way
    /// as happening this frame
    fn flag_updated(&mut self) {
        self.get_reload_state_mut().requires_update = true;
    }

    /// Flag as having data updates that require a rebind
    /// as happening this frame
    fn flag_reload(&mut self) {
        self.get_reload_state_mut().requires_reload = true;
    }

    /// Should be called each frame to correctly track reload state
    fn cycle(&mut self, renderer: &Renderer) {
        let reload_state = self.get_reload_state_mut();
        if reload_state.requires_update {
            reload_state.last_update_cycle = renderer.cycle();
            reload_state.requires_update = false;
        }
        if reload_state.requires_reload {
            reload_state.last_reload_cycle = renderer.cycle();
            reload_state.requires_reload = false;
        }
    }

    /// Get the kind of changes that has occured since a certain cycle
    fn changes_since(&self, cycle: usize) -> ReloadKind {
        let reload_state = self.get_reload_state();

        if reload_state.last_reload_cycle >= cycle {
            ReloadKind::Reload
        } else if reload_state.last_update_cycle >= cycle {
            ReloadKind::Update
        } else {
            ReloadKind::NoChange
        }
    }
}
