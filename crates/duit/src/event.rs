use glam::{vec2, Vec2};

use winit::event::{KeyboardInput, WindowEvent};
#[doc(inline)]
pub use winit::event::{ModifiersState, MouseButton, VirtualKeyCode};

/// An event delivered to an element.
///
/// All elements are notified of all events.
#[derive(Debug, Copy, Clone)]
pub enum Event {
    /// A mouse click.
    MousePress { pos: Vec2, button: MouseButton },
    /// A mouse release.
    MouseRelease { pos: Vec2, button: MouseButton },
    /// The mouse moved.
    MouseMove { pos: Vec2 },
    /// A key press.
    KeyPress { key: VirtualKeyCode },
    /// A key release.
    KeyRelease { key: VirtualKeyCode },
    /// Received a character from the keyboard.
    Character(char),
    /// Scrolling along an axis.
    Scroll { offset: Vec2, mouse_pos: Vec2 },
}

impl Event {
    /// Applies a translation to any coordinates in this event.
    pub fn translated(&self, delta: Vec2) -> Self {
        match *self {
            Event::MousePress { pos, button } => Event::MousePress {
                pos: pos + delta,
                button,
            },
            Event::MouseRelease { pos, button } => Event::MouseRelease {
                pos: pos + delta,
                button,
            },
            Event::MouseMove { pos } => Event::MouseMove { pos: pos + delta },
            Event::Scroll { offset, mouse_pos } => Event::Scroll {
                offset,
                mouse_pos: mouse_pos + delta,
            },
            // events that contain no coordinates (keyboard events)
            e => e,
        }
    }
}

#[derive(Default)]
pub(crate) struct EventTracker {
    cursor_position: Vec2,
}

impl EventTracker {
    pub fn handle_event(&mut self, event: &WindowEvent, scale_factor: f64) -> Option<Event> {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(virtual_keycode),
                        ..
                    },
                ..
            } => Some(match state {
                winit::event::ElementState::Pressed => Event::KeyPress {
                    key: *virtual_keycode,
                },

                winit::event::ElementState::Released => Event::KeyRelease {
                    key: *virtual_keycode,
                },
            }),
            WindowEvent::MouseInput { state, button, .. } => Some(match state {
                winit::event::ElementState::Pressed => Event::MousePress {
                    pos: self.cursor_position,
                    button: *button,
                },
                winit::event::ElementState::Released => Event::MouseRelease {
                    pos: self.cursor_position,
                    button: *button,
                },
            }),
            WindowEvent::CursorMoved { position, .. } => {
                let position = position.to_logical::<f64>(scale_factor);
                self.cursor_position = vec2(position.x as f32, position.y as f32);
                Some(Event::MouseMove {
                    pos: self.cursor_position,
                })
            }
            WindowEvent::ReceivedCharacter(c) => Some(Event::Character(*c)),
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Some(Event::Scroll {
                    offset: vec2(x * 14., y * 14.),
                    mouse_pos: self.cursor_position,
                }),
                winit::event::MouseScrollDelta::PixelDelta(delta) => Some(Event::Scroll {
                    offset: vec2(
                        delta.to_logical(scale_factor).x,
                        delta.to_logical(scale_factor).y,
                    ),
                    mouse_pos: self.cursor_position,
                }),
            },
            _ => None,
        }
    }
}
