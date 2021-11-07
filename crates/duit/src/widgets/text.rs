use duit_core::spec::widgets::TextSpec;
use dume::{Align, Baseline, TextBlob, TextOptions, TextSection};
use glam::Vec2;

use crate::{
    color::Color,
    widget::{Context, Widget, WidgetData},
};

pub struct Text {
    text: dume::Text,
    paragraph: Option<TextBlob>,
}

impl Text {
    pub fn from_spec(spec: &TextSpec) -> Self {
        let initial_text = match spec {
            TextSpec::Simple(text) => text.as_str(),
            TextSpec::Complex { text, .. } => text.as_ref().map(String::as_str).unwrap_or_default(),
        };
        Self {
            text: dume::Text::from_sections([TextSection::Text {
                text: initial_text.into(),
                style: Default::default(),
            }]),
            paragraph: None,
        }
    }

    pub fn new(text: dume::Text) -> Self {
        Self {
            text,
            paragraph: None,
        }
    }

    pub fn set_text(&mut self, text: dume::Text) -> &mut Self {
        self.text = text;
        self.paragraph = None;
        self
    }

    fn create_paragraph(
        &mut self,
        style: &Style,
        cx: &mut Context,
        max_size: Vec2,
    ) -> &mut TextBlob {
        let dume_text = &mut self.text;
        dume_text.set_default_size(style.default_size);
        dume_text.set_default_color(style.default_color.into());
        dume_text.set_default_font_family(style.default_font_family.clone().into());

        let mut blob = cx.canvas.context().create_text_blob(
            dume_text,
            TextOptions {
                wrap_lines: true,
                baseline: Baseline::Top,
                align_h: Align::Start,
                align_v: Align::Start,
            },
        );
        cx.canvas.context().resize_text_blob(&mut blob, max_size);
        self.paragraph = Some(blob);
        self.paragraph.as_mut().unwrap()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    default_color: Color,
    default_size: f32,
    default_font_family: String,
}

impl Widget for Text {
    type Style = Style;

    fn base_class(&self) -> &str {
        "text"
    }

    fn style_changed(&mut self, _style: &Self::Style, _data: &mut WidgetData, _cx: Context) {
        // Re-create the paragraph so text style is updated.
        self.paragraph = None;
    }

    fn layout(
        &mut self,
        style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let paragraph = match &mut self.paragraph {
            Some(p) => {
                cx.canvas.context().resize_text_blob(p, max_size);
                p
            }
            None => self.create_paragraph(style, &mut cx, max_size),
        };

        data.set_size(paragraph.size());
    }

    fn paint(&mut self, _style: &Self::Style, _data: &mut WidgetData, cx: Context) {
        cx.canvas.draw_text(
            self.paragraph
                .as_ref()
                .expect("paragraph not created in layout()"),
            Vec2::ZERO,
            1.,
        );
    }
}
