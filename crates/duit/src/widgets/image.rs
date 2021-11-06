use duit_core::spec::widgets::ImageSpec;
use dume::TextureId;
use glam::{vec2, Vec2};

use crate::{
    widget::{Context, HitTestResult},
    Widget, WidgetData,
};

pub struct Image {
    texture: Option<TextureId>,
    texture_name: Option<String>,
    width: Option<f32>,
    zoom_to_fill: bool,
}

impl Image {
    pub fn from_spec(spec: &ImageSpec) -> Self {
        Self {
            texture: None,
            texture_name: spec.image.clone(),
            width: spec.size,
            zoom_to_fill: spec.zoom_to_fill,
        }
    }

    pub fn set_image(&mut self, sprite_name: impl Into<String>) -> &mut Self {
        self.texture_name = Some(sprite_name.into());
        self.texture = None;
        self
    }

    fn update_texture(&mut self, cx: &mut Context) -> TextureId {
        match self.texture {
            Some(s) => s,
            None => {
                self.texture = Some(
                    cx.canvas
                        .context()
                        .texture_for_name(self.texture_name.as_ref().unwrap())
                        .expect("missing texture"),
                );
                self.texture.unwrap()
            }
        }
    }
}

impl Widget for Image {
    type Style = ();

    fn base_class(&self) -> &str {
        "image"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let mut width = match self.width {
            Some(w) => w,
            None => max_size.x,
        };
        let texture = self.update_texture(&mut cx);
        let dimensions = cx.canvas.context().texture_dimensions(texture);
        let aspect_ratio = dimensions.x as f32 / dimensions.y as f32;
        let height = width / aspect_ratio;

        if self.zoom_to_fill && max_size.x / aspect_ratio < max_size.y {
            width += (max_size.y - max_size.x / aspect_ratio) * aspect_ratio;
        }

        data.set_size(vec2(width, height));

        data.for_each_child(|child| child.layout(&mut cx, vec2(width, height)));
    }

    fn paint(&mut self, _style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        cx.canvas.draw_sprite(
            self.texture.expect("sprite ID not set by layout()"),
            Vec2::ZERO,
            data.size().x,
        );
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
