use std::any::Any;

use duit_core::spec::widgets::ClickableSpec;
use glam::Vec2;
use winit::event::MouseButton;

use crate::{
    widget::{Context, LayoutStrategy},
    Event, Widget, WidgetData,
};

pub struct Clickable {
    on_click: Option<Box<dyn FnMut() -> Box<dyn Any>>>,
}

impl Clickable {
    pub fn from_spec(_s: &ClickableSpec) -> Self {
        Self { on_click: None }
    }

    /// Causes a message to be sent when the widget is clicked.
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

impl Widget for Clickable {
    type Style = ();

    fn base_class(&self) -> &str {
        "clickable"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        data.lay_out_child(LayoutStrategy::Shrink,0., &mut cx, max_size);
    }

    fn paint(&mut self, _style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        data.paint_children(&mut cx);
    }

    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        if let Some(on_click) = self.on_click.as_mut() {
            if let Event::MousePress {
                button: MouseButton::Left,
                pos, ..
            } = event
            {
                if data.bounds().contains(*pos) {
                    cx.send_message((*on_click)());
                }
            }
        }

        data.pass_event_to_children(&mut cx, event);
    }
}
