use cgmath::{Vector2, num_traits::clamp};
use std::collections::HashMap;

use winit::event::{
    ElementState,
    Event,
    KeyboardInput,
    MouseScrollDelta,
    WindowEvent,
};

pub type KeyCode = winit::event::VirtualKeyCode;

pub type MouseButton = winit::event::MouseButton;

/// Information about KeyboardKey or MouseButton.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, )] // TODO: add support for serialization
pub enum Button {
    Key(KeyCode), // TODO: consider support for Key{scancode: u32}?
    Mouse(MouseButton),
}

/// State of a button.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum State {
    Activated,
    Hold,
    Deactivated,
}

/// Input Service
pub struct Input {
    mapper: Box<dyn std::any::Any + Send + Sync>,
    states: HashMap<Button, State>,
    mouse_scroll_delta: f32,
    mouse_position: Vector2<f32>,
    last_mouse_position: Vector2<f32>,
    window_size: Vector2<f32>, // TODO: move to other struct or service
}

impl Input {
    pub fn new(mapper: Box<dyn std::any::Any + Send + Sync>) -> Self {
        Self {
            mapper,
            states: HashMap::new(),
            mouse_scroll_delta: 0.0,
            mouse_position: Vector2::new(0.0, 0.0),
            last_mouse_position: Vector2::new(0.0, 0.0),
            window_size: Vector2::new(0.0, 0.0),
        }
    }

    /// Returns the status of the mapped action.
    pub fn action_state<T>(&self, action: T) -> Option<State>
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        if let Some(button) = self.action_mapped(action) {
            if let Some(state) = self.states.get(button) {
                return Some(*state);
            }
        }
        None
    }


    /// Checks if mapped action button is pressed.
    pub fn is_action_activated<T>(&self, action: T) -> bool
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        self.action_state(action)
            .map(|state| state == State::Activated)
            .unwrap_or(false)
    }

    /// Checks if mapped action button is released.
    pub fn is_action_deactivated<T>(&self, action: T) -> bool
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        self.action_state(action)
            .map(|state| state == State::Deactivated)
            .unwrap_or(false)
    }

    /// Checks if mapped button is pressed or hold.
    pub fn is_action_hold<T>(&self, action: T) -> bool
    where
        Self: ActionMapper<T>,
        T: Copy + Eq + std::hash::Hash,
    {
        self.action_state(action)
            .map(|state| state == State::Hold || state == State::Activated)
            .unwrap_or(false)
    }

    /// Get input amapper reference
    pub fn mapper<T: 'static + Send + Sync>(&self) -> &T {
        self.mapper.downcast_ref::<T>().unwrap()
    }

    /// Get mutual mapper reference
    pub fn mapper_mut<T: 'static + Send + Sync>(&mut self) -> &mut T {
        self.mapper.downcast_mut::<T>().unwrap()
    }

    /// Set window size
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_size = Vector2::new(width, height);
    }

    /// Mouse scroll delta. Value should can be positive (up) or negative (down), usually -1 and 1
    /// (but should be smaller on higher, depends on OS and device).
    pub fn mouse_scroll(&self) -> f32 {
        self.mouse_scroll_delta
    }

    /// Current mouse position in pixel coordinates. The top-left of the window is at (0, 0).
    pub fn mouse_position(&self) -> Vector2<f32> {
        self.mouse_position
    }

    /// Difference of the mouse position from the last frame in pixel coordinates. The top-left of
    /// the window is at (0, 0).
    pub fn mouse_delta(&self) -> Vector2<f32> {
        self.mouse_position - self.last_mouse_position
    }

    /// Normalized mouse position. The top-left of the window is at (0, 0), bottom-right at (1, 1).
    pub fn mouse_position_normalized(&self) -> Vector2<f32> {
        Vector2::new(
            clamp(self.mouse_position.x / self.window_size.x, 0.0, 1.0),
            clamp(self.mouse_position.y / self.window_size.y, 0.0, 1.0),
        )
    }

    /// This method should be called in application.rs after render, so States from events will be
    /// properly updated.
    pub fn reset(&mut self) {
        self.last_mouse_position = self.mouse_position;
        self.mouse_scroll_delta = 0.0;

        self.states.retain(|_btn, state| match state {
            State::Activated => {
                *state = State::Hold;
                true
            }
            State::Deactivated => false,
            _ => true,
        });
    }

    /// This method should be called in application.rs in event_loop, so systems will get actual
    /// Input data.
    pub fn on_event(&mut self, event: &winit::event::Event<'_, ()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => self.on_keyboard_event(input),
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => self.on_mouse_click_event(*state, *button),
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => self.on_cursor_moved_event(position),
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => self.on_mouse_wheel_event(&delta),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => self.window_size = Vector2::new(size.width as f32, size.height as f32),
            _ => {}
        }
    }

    /// Handle cursor moved event from winit.
    fn on_cursor_moved_event(&mut self, position: &winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = Vector2 {
            x: position.x as f32,
            y: position.y as f32,
        };
    }

    /// Handle keyboard event from winit.
    fn on_keyboard_event(&mut self, input: &KeyboardInput) {
        if let Some(key_code) = input.virtual_keycode {
            self.on_button_state(Button::Key(key_code), input.state);
        }
    }

    fn on_button_state(&mut self, btn: Button, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if *self.states.get(&btn).unwrap_or(&State::Deactivated) == State::Deactivated {
                    self.states.insert(btn, State::Activated);
                }
            },
            ElementState::Released => {
                self.states.insert(btn, State::Deactivated);
            },
        }
    }

    /// Handle mouse click events from winit.
    fn on_mouse_click_event(&mut self, state: ElementState, button: winit::event::MouseButton) {
        self.on_button_state(Button::Mouse(button), state);
    }

    /// Handle mouse wheel event from winit.
    fn on_mouse_wheel_event(&mut self, delta: &MouseScrollDelta) {
        let change = match delta {
            MouseScrollDelta::LineDelta(_x, y) => *y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };
        self.mouse_scroll_delta += change;
    }


}

/// Trait for Input Mapper, needs to be implemented.
pub trait ActionMapper<T: Copy + Eq + std::hash::Hash> {
    fn action_mapped(&self, action: T) -> Option<&Button>;
}

/// Mapper for input
pub struct Mapper<T> {
    map: HashMap<T, Button>,
}

impl<T> Mapper<T>
where
    T: Copy + Eq + std::hash::Hash
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Add a new action to mapper. If action already exists, it will be overridden.
    pub fn add_action(&mut self, action: T, button: Button) {
        self.map.insert(action, button);
    }

    /// Add multiple actions to mapper. Existing actions will be overridden.
    pub fn add_actions(&mut self, actions: Vec<(T, Button)>) {
        for (action, button) in actions.iter() {
            self.map.insert(*action, *button);
        }
    }

    /// Get button that is binded to that action.
    pub fn get_button(&self, action: T) -> Option<&Button> {
        self.map.get(&action)
    }

    /// Remove action from mapper.
    pub fn remove_action(&mut self, action: T) {
        self.map.remove(&action);
    }

    /// Remove multiple actions from mapper.
    pub fn remove_actions(&mut self, actions: Vec<T>) {
        for action in actions.iter() {
            self.map.remove(action);
        }
    }

    /// Removes all actions and set new ones.
    pub fn set(&mut self, actions: Vec<(T, Button)>) {
        self.map.clear();
        self.add_actions(actions);
    }
}

impl<T: Copy + Eq + std::hash::Hash> Default for Mapper<T> {
    fn default() -> Self {
        Self::new()
    }
}
