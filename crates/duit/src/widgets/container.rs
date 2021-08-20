use duit_core::spec::widgets::{ContainerMode, ContainerSpec};
use glam::Vec2;

use crate::{
    widget::{Context, LayoutStrategy},
    Color, Widget, WidgetData,
};

pub struct Container {
    mode: ContainerMode,
}

impl Container {
    pub fn from_spec(spec: &ContainerSpec) -> Self {
        Self { mode: spec.mode }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    border_radius: f32,
    border_width: f32,
    border_color: Color,
    background_color: Color,
}

impl Widget for Container {
    type Style = Style;

    fn base_class(&self) -> &str {
        "container"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let (strategy, padding) = match self.mode {
            ContainerMode::Shrink => (LayoutStrategy::Shrink, 0.),
            ContainerMode::FillParent => (LayoutStrategy::Fill, 0.),
            ContainerMode::FillParentAndPad(padding) => (LayoutStrategy::Fill, padding),
            ContainerMode::Pad(padding) => (LayoutStrategy::Shrink, padding),
        };
        data.lay_out_child(strategy, padding, &mut cx, max_size);
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        let canvas = &mut cx.canvas;

        canvas
            .begin_path()
            .rounded_rect(Vec2::ZERO, data.size(), style.border_radius);

        canvas.solid_color(style.background_color.into()).fill();

        canvas
            .solid_color(style.border_color.into())
            .stroke_width(style.border_width)
            .stroke();

        data.paint_children(&mut cx);
    }
}
