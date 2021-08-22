mod color;
mod event;
mod spec;
mod style;
mod ui;
mod widget;
pub mod widgets;
mod window;

pub use color::Color;
pub use event::Event;
pub use spec::InstanceHandle;
pub use style::StyleError;
pub use ui::{Ui, WindowId};
pub use widget::{Widget, WidgetData, WidgetHandle, WidgetPodHandle, WidgetState};
pub use window::WindowPositioner;

pub use duit_core::{
    spec::{Spec, SpecError, ValidationError},
    Align,
};

pub use dume_renderer::Rect;
pub use glam::Vec2;

pub trait RectExt {
    fn expanded(self, radius: f32) -> Self;
}

impl RectExt for Rect {
    fn expanded(self, radius: f32) -> Self {
        Rect::new(
            self.pos - Vec2::splat(radius),
            self.size + Vec2::splat(radius),
        )
    }
}
