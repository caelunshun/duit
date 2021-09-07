use std::{iter, time::Instant};

use duit_core::spec::widgets::TextInputSpec;
use dume::{
    font::Query,
    Align,
    Baseline::{self},
    Canvas, Paragraph, Text, TextLayout, TextSection, TextStyle,
};
use glam::{vec2, Vec2};
use winit::event::{MouseButton, VirtualKeyCode};

use crate::{widget::Context, Color, Event, Widget, WidgetData};

pub struct TextInput {
    width: Option<f32>,
    placeholder: String,
    max_len: Option<usize>,
    is_password: bool,

    placeholder_paragraph: Option<Paragraph>,

    text: String,
    text_paragraph: Option<Paragraph>,

    focused: bool,

    last_change: Instant,
    create_time: Instant,
}

impl TextInput {
    pub fn from_spec(spec: &TextInputSpec) -> Self {
        Self {
            width: spec.width,

            placeholder: spec.placeholder.clone().unwrap_or_default(),
            placeholder_paragraph: None,

            is_password: spec.is_password,
            max_len: spec.max_len,

            text: String::new(),
            text_paragraph: None,

            focused: false,

            last_change: Instant::now(),
            create_time: Instant::now(),
        }
    }

    pub fn current_input(&self) -> &str {
        &self.text
    }

    fn paragraph_to_draw(&self) -> &Paragraph {
        if self.text.is_empty() {
            self.placeholder_paragraph
                .as_ref()
                .expect("placeholder paragraph not created")
        } else {
            self.text_paragraph
                .as_ref()
                .expect("text paragraph not created")
        }
    }

    fn mark_text_dirty(&mut self) {
        // causes the paragraph to be recreated in layout()
        self.text_paragraph = None;
    }
}

fn make_password_text(text: &str) -> String {
    iter::repeat("•").take(text.chars().count()).collect()
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    background_color: Color,
    border_color: Color,
    border_width: f32,
    border_radius: f32,
    cursor_color: Color,
    cursor_width: f32,
    font: String,
    font_size: f32,
    font_color: Color,
    placeholder_font_color: Color,
    padding: f32,
}

fn create_paragraph(cv: &mut Canvas, style: &Style, color: Color, text: String) -> Paragraph {
    let text = Text::from_sections(vec![TextSection::Text {
        text,
        style: TextStyle {
            color: color.into(),
            size: style.font_size,
            font: Query {
                family: style.font.clone(),
                ..Default::default()
            },
        },
    }]);

    cv.create_paragraph(
        text,
        TextLayout {
            max_dimensions: Vec2::splat(f32::INFINITY),
            line_breaks: false,
            baseline: Baseline::Top,
            align_h: Align::Start,
            align_v: Align::Start,
        },
    )
}

impl Widget for TextInput {
    type Style = Style;

    fn base_class(&self) -> &str {
        "text_input"
    }

    fn layout(&mut self, style: &Self::Style, data: &mut WidgetData, cx: Context, max_size: Vec2) {
        if self.placeholder_paragraph.is_none() {
            self.placeholder_paragraph = Some(create_paragraph(
                cx.canvas,
                style,
                style.placeholder_font_color,
                self.placeholder.clone(),
            ));
        }

        if self.text_paragraph.is_none() {
            let text = if !self.is_password {
                self.text.clone()
            } else {
                make_password_text(&self.text)
            };

            self.text_paragraph = Some(create_paragraph(cx.canvas, style, style.font_color, text));
        }

        let width = match self.width {
            Some(x) => x,
            None => max_size.x,
        };

        let height = style.font_size + 2. * style.padding;

        data.set_size(vec2(width, height));
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        let cv = &mut cx.canvas;

        cv.begin_path()
            .rounded_rect(Vec2::ZERO, data.size(), style.border_radius)
            .solid_color(style.background_color.into())
            .fill();

        cv.solid_color(style.border_color.into())
            .stroke_width(style.border_width)
            .stroke();

        let text_pos = Vec2::new(style.padding, style.padding / 2.);

        cv.draw_paragraph(text_pos, self.paragraph_to_draw());

        // Cursor
        let time = self.create_time.elapsed().as_secs_f32();
        if self.focused
            && (self.last_change.elapsed().as_secs_f32() <= 0.75 || (time * 2.0) as u32 % 2 == 0)
        {
            let cursor_pos = text_pos
                + self
                    .text_paragraph
                    .as_ref()
                    .unwrap()
                    .lines()
                    .last()
                    .expect("no last line")
                    .end;

            cv.begin_path()
                .move_to(cursor_pos)
                .line_to(cursor_pos + vec2(0., style.font_size))
                .stroke_width(style.cursor_width)
                .solid_color(style.cursor_color.into())
                .stroke();
        }
    }

    fn handle_event(&mut self, data: &mut WidgetData, _cx: Context, event: &Event) {
        let focused = self.focused;

        match event {
            Event::MousePress {
                pos,
                button: MouseButton::Left,
            } => {
                self.focused = data.bounds().contains(*pos);
            }
            Event::KeyPress { key } => {
                if self.focused {
                    if matches!(key, VirtualKeyCode::Back | VirtualKeyCode::Delete) {
                        self.text.pop();
                        self.mark_text_dirty();
                    }
                }
            }
            Event::Character(c) if self.focused && !c.is_control() => {
                if Some(self.text.len()) != self.max_len {
                    self.text.push(*c);
                    self.mark_text_dirty();
                }
            }
            _ => {}
        }

        if focused != self.focused {
            // Update style classes
            if self.focused {
                data.add_class("focused");
            } else {
                data.remove_class("focused");
            }
        }
    }
}
