//! Specs for each widget.

use std::slice;

use serde::Deserialize;

use crate::Align;

use super::{validate_ident, ValidationError};

#[derive(Debug, Deserialize)]
pub struct BaseSpec {
    pub id: Option<String>,
    pub flex: Option<f32>,
    #[serde(default)]
    pub classes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum Widget {
    Column(ColumnSpec),
    Row(RowSpec),
    Text(TextSpec),
    TextInput(TextInputSpec),
    Button(ButtonSpec),
    Image(ImageSpec),
    Container(ContainerSpec),
    ProgressBar(ProgressBarSpec),
    Clickable(ClickableSpec),
    Slider(SliderSpec),
}

impl Widget {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(base) = self.base_spec() {
            if let Some(id) = &base.id {
                validate_ident(id)?;
            }
        }
        Ok(())
    }

    pub fn base_spec(&self) -> Option<&BaseSpec> {
        match self {
            Widget::Column(s) => Some(&s.flex.base),
            Widget::Row(s) => Some(&s.flex.base),
            Widget::Text(t) => match t {
                TextSpec::Simple(_) => None,
                TextSpec::Complex { base, .. } => Some(base),
            },
            Widget::TextInput(s) => Some(&s.base),
            Widget::Button(s) => Some(&s.base),
            Widget::Image(s) => Some(&s.base),
            Widget::Container(s) => Some(&s.base),
            Widget::ProgressBar(s) => Some(&s.base),
            Widget::Clickable(s) => Some(&s.base),
            Widget::Slider(s) => Some(&s.base),
        }
    }

    pub fn children(&self) -> &[Widget] {
        match self {
            Widget::Column(s) => s.flex.children.as_slice(),
            Widget::Row(s) => s.flex.children.as_slice(),
            Widget::Button(s) => slice::from_ref(&*s.child),
            Widget::Image(s) => match &s.child {
                Some(c) => std::slice::from_ref(&**c),
                None => &[],
            },
            Widget::Container(s) => slice::from_ref(&*s.child),
            Widget::ProgressBar(s) => match &s.child {
                Some(c) => std::slice::from_ref(&**c),
                None => &[],
            },
            Widget::Clickable(s) => slice::from_ref(&*s.child),
            _ => &[],
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Widget::Column(_) => "Flex",
            Widget::Row(_) => "Flex",
            Widget::Text(_) => "Text",
            Widget::TextInput(_) => "TextInput",
            Widget::Button(_) => "Button",
            Widget::Image(_) => "Image",
            Widget::Container(_) => "Container",
            Widget::ProgressBar(_) => "ProgressBar",
            Widget::Clickable(_) => "Clickable",
            Widget::Slider(_) => "Slider",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ColumnSpec {
    #[serde(flatten)]
    pub flex: FlexSpec,
}

#[derive(Debug, Deserialize)]
pub struct RowSpec {
    #[serde(flatten)]
    pub flex: FlexSpec,
}

#[derive(Debug, Deserialize)]
pub struct FlexSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    #[serde(default)]
    pub align_h: Align,
    #[serde(default)]
    pub align_v: Align,
    #[serde(default)]
    pub children: Vec<Widget>,
    #[serde(default)]
    pub spacing: f32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TextSpec {
    Simple(String),
    Complex {
        #[serde(flatten)]
        base: BaseSpec,
        text: Option<String>,
        #[serde(default)]
        align_h: Align,
        #[serde(default)]
        align_v: Align,
    },
}

#[derive(Debug, Deserialize)]
pub struct TextInputSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub placeholder: Option<String>,
    pub width: Option<f32>,
    pub max_len: Option<usize>,
    #[serde(default)]
    pub is_password: bool,
}

#[derive(Debug, Deserialize)]
pub struct ButtonSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub child: Box<Widget>,
}

#[derive(Debug, Deserialize)]
pub struct ImageSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub image: Option<String>,
    pub size: Option<f32>,
    pub child: Option<Box<Widget>>,
    #[serde(default)]
    pub zoom_to_fill: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum ContainerMode {
    Shrink,
    FillParent,
    Pad(f32),
    FillParentAndPad(f32),
}

#[derive(Debug, Deserialize)]
pub struct ContainerSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub mode: ContainerMode,
    pub child: Box<Widget>,
}

#[derive(Debug, Deserialize)]
pub struct ProgressBarSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub child: Option<Box<Widget>>,
}

#[derive(Debug, Deserialize)]
pub struct ClickableSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub child: Box<Widget>,
}

#[derive(Debug, Deserialize)]
pub struct SliderSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub width: Option<f32>,
}
