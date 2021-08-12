use duit_core::spec::widgets::ButtonSpec;
use glam::Vec2;

use crate::{
    widget::{Context, LayoutStrategy},
    Color, Widget, WidgetData,
};

pub struct Button {}

impl Button {
    pub fn from_spec(_spec: &ButtonSpec) -> Self {
        Self {}
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
            LayoutStrategy::Shrink {
                padding: style.padding,
            },
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
}
