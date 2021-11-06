use duit_core::{spec::widgets::DividerSpec, Axis};
use glam::Vec2;

use crate::{widget::Context, Color, Widget, WidgetData};

pub struct Divider {
    axis: Axis,
    padding: f32,
}

impl Divider {
    pub fn from_spec(spec: &DividerSpec) -> Self {
        Self {
            axis: spec.axis,
            padding: spec.padding,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    pub line_width: f32,
    pub line_color: Color,
}

impl Widget for Divider {
    type Style = Style;

    fn base_class(&self) -> &str {
        "divider"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        _cx: Context,
        mut max_size: Vec2,
    ) {
        let size = 5.;
        match self.axis {
            Axis::Horizontal => max_size.y = size,
            Axis::Vertical => max_size.x = size,
        }
        max_size[self.axis as usize] -= 2. * self.padding;
        data.set_size(max_size);
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, cx: Context) {
        let mut endpoint = Vec2::ZERO;
        endpoint[self.axis as usize] = data.size()[self.axis as usize];
        cx.canvas
            .begin_path()
            .move_to(Vec2::ZERO)
            .line_to(endpoint)
            .solid_color(style.line_color)
            .stroke_width(style.line_width)
            .stroke();
    }
}
