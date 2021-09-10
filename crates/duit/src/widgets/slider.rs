use duit_core::spec::widgets::SliderSpec;
use dume::Rect;
use glam::vec2;
use winit::event::MouseButton;

use crate::{widget::Context, Color, Event, RectExt, Widget, WidgetData};

#[derive(Debug)]
pub struct Slider {
    width: Option<f32>,

    value: f32,

    grabbed: bool,

    handle_rect: Rect,
}

impl Slider {
    pub fn from_spec(spec: &SliderSpec) -> Self {
        Self {
            width: spec.width,

            value: 0.,

            grabbed: false,

            handle_rect: Rect::default(),
        }
    }

    /// Sets the value displayed on the slider. Should
    /// be in `[0, 1]`.
    pub fn set_value(&mut self, value: f32) -> &mut Self {
        self.value = value;
        self
    }

    /// Gets the slider value between 0 and 1, inclusive.
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    line_width: f32,
    line_color: Color,

    handle_border_radius: f32,
    handle_border_width: f32,
    handle_border_color: Color,
    handle_color: Color,
    handle_width: f32,
    handle_height: f32,
}

impl Widget for Slider {
    type Style = Style;

    fn base_class(&self) -> &str {
        "slider"
    }

    fn layout(
        &mut self,
        style: &Self::Style,
        data: &mut WidgetData,
        _cx: Context,
        max_size: glam::Vec2,
    ) {
        let width = match self.width {
            Some(x) => x,
            None => max_size.x,
        };

        let height = style.handle_height;

        data.set_size(vec2(width, height));
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        let cv = &mut cx.canvas;

        // Bar
        cv.begin_path()
            .move_to(vec2(0.0, data.size().y / 2.))
            .line_to(vec2(data.size().x, data.size().y / 2.))
            .stroke_width(style.line_width)
            .solid_color(style.line_color.into())
            .stroke();

        // Handle
        let handle_pos = vec2(data.size().x * self.value, 0.);
        let handle_size = vec2(style.handle_width, style.handle_height);
        self.handle_rect = Rect::new(handle_pos, handle_size);

        cv.begin_path()
            .rounded_rect(handle_pos, handle_size, style.handle_border_radius)
            .solid_color(style.handle_color.into())
            .fill();

        cv.stroke_width(style.handle_border_width)
            .solid_color(style.handle_border_color.into())
            .stroke();
    }

    fn handle_event(&mut self, data: &mut WidgetData, _cx: Context, event: &Event) {
        let was_grabbed = self.grabbed;
        match event {
            Event::MousePress {
                pos,
                button: MouseButton::Left, ..
            } => {
                if self.handle_rect.expanded(5.).contains(*pos) {
                    self.grabbed = true;
                }
            }
            Event::MouseRelease {
                button: MouseButton::Left,
                ..
            } => self.grabbed = false,
            Event::MouseMove { pos } if self.grabbed => {
                self.value = (pos.x / data.size().x).clamp(0., 1.);
            }
            _ => {}
        }

        // Update style classes
        if was_grabbed != self.grabbed {
            if self.grabbed {
                data.add_class("grabbed");
            } else {
                data.remove_class("grabbed");
            }
        }
    }
}
