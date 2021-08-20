use duit_core::spec::widgets::ImageSpec;
use dume_renderer::SpriteId;
use glam::{vec2, Vec2};

use crate::{widget::Context, Widget, WidgetData};

pub struct Image {
    sprite: Option<SpriteId>,
    sprite_name: Option<String>,
    width: Option<f32>,
    zoom_to_fill: bool,
}

impl Image {
    pub fn from_spec(spec: &ImageSpec) -> Self {
        Self {
            sprite: None,
            sprite_name: spec.image.clone(),
            width: spec.size,
            zoom_to_fill: spec.zoom_to_fill,
        }
    }

    pub fn set_image(&mut self, sprite_name: impl Into<String>) -> &mut Self {
        self.sprite_name = Some(sprite_name.into());
        self.sprite = None;
        self
    }

    fn update_sprite(&mut self, cx: &mut Context) -> SpriteId {
        match self.sprite {
            Some(s) => s,
            None => {
                self.sprite = Some(
                    cx.canvas
                        .sprite_by_name(self.sprite_name.as_ref().unwrap())
                        .unwrap_or_else(|| {
                            panic!("missing sprite '{}'", self.sprite_name.as_ref().unwrap())
                        }),
                );
                self.sprite.unwrap()
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
        let sprite = self.update_sprite(&mut cx);
        let dimensions = cx.canvas.sprite_dimensions(sprite);
        let aspect_ratio = dimensions.x as f32 / dimensions.y as f32;
        let height = width / aspect_ratio;

        if self.zoom_to_fill && max_size.x / aspect_ratio < max_size.y {
            width += (max_size.y - max_size.x / aspect_ratio) * aspect_ratio;
        }

        data.set_size(vec2(width, height));

        data.for_each_child(|child| child.layout(&mut cx, max_size));
    }

    fn paint(&mut self, _style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        cx.canvas.draw_sprite(
            self.sprite.expect("sprite ID not set by layout()"),
            Vec2::ZERO,
            data.size().x,
        );
        data.paint_children(&mut cx);
    }
}
