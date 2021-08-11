//! Specs for each widget.

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
}

#[derive(Debug, Deserialize)]
pub struct ButtonSpec {
    #[serde(flatten)]
    pub base: BaseSpec,
    pub child: Box<Widget>,
}
