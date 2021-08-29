use std::{any::Any, collections::VecDeque};

use dume_renderer::{Canvas, Rect};
use glam::Vec2;

use crate::{
    style::StyleEngine,
    widget::{Context, WidgetPodHandle},
    Event,
};

/// Computes a window's size and position
/// from the total available space.
pub trait WindowPositioner: 'static {
    fn compute_position(&self, available_space: Vec2) -> Rect;
}

/// A window contains a single root widget, a position, and a size.
///
/// Windows do not correspond to native windows. Instead, they're
/// a way to layer different parts of a UI within a single native window.
pub(crate) struct Window {
    root: WidgetPodHandle,
    positioner: Box<dyn WindowPositioner>,
    pub(crate) z_index: u64,
    hidden: bool,
}

impl Window {
    pub fn new(root: WidgetPodHandle, positioner: impl WindowPositioner, z_index: u64) -> Self {
        Self {
            root,
            positioner: Box::new(positioner),
            z_index,
            hidden: false,
        }
    }

    pub fn render(
        &mut self,
        canvas: &mut Canvas,
        style_engine: &mut StyleEngine,
        messages: &mut VecDeque<Box<dyn Any>>,
        available_space: Vec2,
    ) {
        if self.hidden {
            return;
        }

        let layout = self.positioner.compute_position(available_space);

        let mut root = self.root.borrow_mut();
        let mut cx = Context {
            canvas,
            style_engine,
            messages,
        };

        cx.canvas.translate(layout.pos);

        root.layout(&mut cx, layout.size);
        root.paint(&mut cx);
        root.paint_overlay(&mut cx);

        cx.canvas.reset_transform();
    }

    pub fn handle_event(
        &mut self,
        canvas: &mut Canvas,
        style_engine: &mut StyleEngine,
        messages: &mut VecDeque<Box<dyn Any>>,
        event: &Event,
        available_space: Vec2,
    ) {
        let mut root = self.root.borrow_mut();
        let mut cx = Context {
            canvas,
            style_engine,
            messages,
        };

        let event = event.translated(-self.positioner.compute_position(available_space).pos);

        root.handle_event(&mut cx, &event);
    }

    pub fn hide(&mut self) {
        self.hidden = true;
    }
}
