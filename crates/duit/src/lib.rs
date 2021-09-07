mod color;
mod event;
mod spec;
mod style;
mod ui;
pub mod widget;
pub mod widgets;
mod window;

use std::{cell::RefCell, rc::Rc};

pub use color::Color;
pub use event::Event;
pub use spec::InstanceHandle;
pub use style::StyleError;
pub use ui::{Ui, WindowId};
use widget::WidgetPod;
pub use widget::{Widget, WidgetData, WidgetHandle, WidgetPodHandle, WidgetState};
pub use window::WindowPositioner;

pub use duit_core::{
    spec::{Spec, SpecError, ValidationError},
    Align, Axis,
};

pub use dume::Rect;
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

/// Constructs a [`WidgetPodHandle`] to the given widget.
pub fn widget(w: impl Widget) -> WidgetPodHandle {
    Rc::new(RefCell::new(WidgetPod::new(Box::new(w))))
}
