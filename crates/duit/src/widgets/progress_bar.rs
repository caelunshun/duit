use duit_core::spec::widgets::ProgressBarSpec;
use glam::{vec2, Vec2};

use crate::{widget::Context, Color, Widget, WidgetData};

pub struct ProgressBar {
    width: Option<f32>,
    height: Option<f32>,
    progress: f32,
    projected_progress: Option<f32>,
}

impl ProgressBar {
    pub fn from_spec(spec: &ProgressBarSpec) -> Self {
        Self {
            width: spec.width,
            height: spec.height,
            progress: 0.0,
            projected_progress: None,
        }
    }

    pub fn set_progress(&mut self, progress: f32) -> &mut Self {
        self.progress = progress.clamp(0., 1.);
        self
    }

    pub fn set_projected_progress(&mut self, projected_progress: f32) -> &mut Self {
        self.projected_progress = Some(projected_progress.clamp(0., 1.));
        self
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    border_radius: f32,
    border_width: f32,
    border_color: Color,
    background_color: Color,
    progress_color: Color,
    projected_progress_color: Color,
}

impl Widget for ProgressBar {
    type Style = Style;

    fn base_class(&self) -> &str {
        "progress_bar"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let width = match self.width {
            Some(w) => w,
            None => max_size.x,
        };
        let height = match self.height {
            Some(h) => h,
            None => max_size.y,
        };

        data.set_size(vec2(width, height));

        data.for_each_child(|child| child.layout(&mut cx, max_size));
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        let canvas = &mut cx.canvas;

        // Background
        canvas
            .begin_path()
            .rounded_rect(Vec2::ZERO, data.size(), style.border_radius)
            .solid_color(style.background_color.into())
            .fill();

        // Progress
        let progress_size = data.size() * vec2(self.progress, 1.0);
        canvas
            .begin_path()
            .rounded_rect(Vec2::ZERO, progress_size, style.border_radius)
            .solid_color(style.progress_color.into())
            .fill();

        // Projected progress
        if let Some(projected_progress) = self.projected_progress {
            canvas
                .begin_path()
                .rounded_rect(
                    vec2(progress_size.x, 0.0),
                    data.size() * vec2(projected_progress - self.progress, 1.0),
                    style.border_radius,
                )
                .solid_color(style.projected_progress_color.into())
                .fill();
        }

        // Border
        canvas
            .begin_path()
            .rounded_rect(Vec2::ZERO, data.size(), style.border_radius)
            .solid_color(style.border_color.into())
            .stroke_width(style.border_width)
            .stroke();

        data.paint_children(&mut cx);
    }
}
