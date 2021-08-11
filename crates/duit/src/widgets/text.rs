use ahash::AHashMap;
use duit_core::spec::widgets::TextSpec;
use dume_renderer::{
    font::{self, Query},
    Align, Baseline, Paragraph, TextLayout, TextStyle,
};
use glam::{vec2, Vec2};

use crate::{
    color::Color,
    widget::{Context, Widget, WidgetData},
};

pub struct Text {
    queued_markup: Option<(String, AHashMap<String, String>)>,
    paragraph: Option<Paragraph>,
}

impl Text {
    pub fn from_spec(spec: &TextSpec) -> Self {
        let markup = match spec {
            TextSpec::Simple(text) => text.as_str(),
            TextSpec::Complex { text, .. } => text.as_ref().map(String::as_str).unwrap_or_default(),
        };
        Self {
            queued_markup: Some((markup.to_owned(), AHashMap::new())),
            paragraph: None,
        }
    }

    pub fn set_text(&mut self, markup: String, variables: AHashMap<String, String>) -> &mut Self {
        self.queued_markup = Some((markup, variables));
        self.paragraph = None;
        self
    }

    fn create_text(&mut self, style: &Style) -> dume_renderer::Text {
        let (markup, variables) = self.queued_markup.as_ref().unwrap();
        dume_renderer::markup::parse(
            &markup,
            TextStyle {
                color: style.default_color.into(),
                size: style.default_size,
                font: Query {
                    family: style.default_font_family.clone(),
                    style: font::Style::Normal,
                    weight: font::Weight::Normal,
                },
            },
            |var| {
                variables
                    .get(var)
                    .cloned()
                    .unwrap_or_else(|| panic!("missing text markup variable '{}'", var))
            },
        )
        .expect("failed to parse text markup")
    }

    fn create_paragraph(
        &mut self,
        style: &Style,
        cx: &mut Context,
        max_size: Vec2,
    ) -> &mut Paragraph {
        let dume_text = self.create_text(style);
        self.paragraph = Some(cx.canvas.create_paragraph(
            dume_text,
            TextLayout {
                max_dimensions: max_size,
                line_breaks: true,
                baseline: Baseline::Top,
                align_h: Align::Start,
                align_v: Align::Start,
            },
        ));
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
                cx.canvas.resize_paragraph(p, max_size);
                p
            }
            None => self.create_paragraph(style, &mut cx, max_size),
        };

        data.set_size(vec2(paragraph.width(), paragraph.height()));
    }

    fn paint(&mut self, _style: &Self::Style, _data: &mut WidgetData, cx: Context) {
        cx.canvas.draw_paragraph(
            Vec2::ZERO,
            self.paragraph
                .as_ref()
                .expect("paragraph not created in layout()"),
        );
    }
}
