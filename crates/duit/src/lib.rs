mod spec;
mod style;
mod ui;
mod widget;
pub mod widgets;
mod window;
mod event;
mod color;

pub use spec::InstanceHandle;
pub use style::StyleError;
pub use ui::Ui;
pub use color::Color;
pub use widget::{Widget, WidgetData, WidgetHandle, WidgetPodHandle, WidgetState};
pub use window::WindowPositioner;
pub use event::Event;

pub use duit_core::{Align, spec::{Spec, SpecError, ValidationError}};

pub use glam::Vec2;
pub use dume_renderer::Rect;
