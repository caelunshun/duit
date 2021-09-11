use duit_core::spec::widgets::{ContainerMode, ContainerSpec};
use glam::Vec2;

use crate::{Color, Widget, WidgetData, widget::{Context, HitTestResult, LayoutStrategy}};

pub struct Container {
    mode: ContainerMode,
    fill_width: bool,
    fill_height: bool,
}

impl Container {
    pub fn from_spec(spec: &ContainerSpec) -> Self {
        Self {
            mode: spec.mode,
            fill_width: spec.fill_width,
            fill_height: spec.fill_height,
        }
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
        let mut size = data.lay_out_child(strategy, padding, &mut cx, max_size);
        if self.fill_width {
            size.x = max_size.x;
        }
        if self.fill_height {
            size.y = max_size.y;
        }
        data.set_size(size);
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        let canvas = &mut cx.canvas;

        let pos = match self.mode {
            ContainerMode::FillParent | ContainerMode::FillParentAndPad(_) => Vec2::ZERO,
            _ => data.child_offset(),
        };

        canvas
            .begin_path()
            .rounded_rect(pos, data.size(), style.border_radius);

        canvas.solid_color(style.background_color.into()).fill();

        canvas
            .solid_color(style.border_color.into())
            .stroke_width(style.border_width)
            .stroke();

        data.paint_children(&mut cx);
    }

    fn hit_test(&self, data: &WidgetData, pos: Vec2) -> HitTestResult {
        if data.bounds().contains(pos) {
            HitTestResult::Hit
        } else {
            HitTestResult::Missed
        }
    } 
}
