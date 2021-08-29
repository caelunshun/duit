use std::any::Any;

use duit_core::spec::widgets::ButtonSpec;
use glam::Vec2;
use winit::event::MouseButton;

use crate::{
    widget::{Context, LayoutStrategy},
    Color, Event, Widget, WidgetData,
};

pub struct Button {
    on_click: Option<Box<dyn FnMut() -> Box<dyn Any>>>,
}

impl Button {
    pub fn from_spec(_spec: &ButtonSpec) -> Self {
        Self::new()
    }

    pub fn new() -> Self {
        Self {
            on_click: None,
        }
    }

    /// Causes a message to be sent when the button is clicked.
    ///
    /// If an `on_click` message is already set, it is overriden.
    pub fn on_click<Message: 'static>(
        &mut self,
        mut message: impl FnMut() -> Message + 'static,
    ) -> &mut Self {
        self.on_click = Some(Box::new(move || Box::new(message())));
        self
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    padding: f32,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
    background_color: Color,
}

impl Widget for Button {
    type Style = Style;

    fn base_class(&self) -> &str {
        "button"
    }

    fn layout(
        &mut self,
        style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        data.lay_out_child(
            LayoutStrategy::Shrink,
            style.padding,
            &mut cx,
            max_size,
        );
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        let canvas = &mut cx.canvas;

        canvas
            .begin_path()
            .rounded_rect(Vec2::ZERO, data.size(), style.border_radius)
            .solid_color(style.background_color.into())
            .fill();
        canvas
            .solid_color(style.border_color.into())
            .stroke_width(style.border_width)
            .stroke();

        data.paint_children(&mut cx);
    }

    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        if let Some(on_click) = self.on_click.as_mut() {
            if let Event::MousePress {
                button: MouseButton::Left,
                pos,
            } = event
            {
                if data.bounds().contains(*pos) {
                    cx.send_message((*on_click)());
                }
            }
        }
    }
}
