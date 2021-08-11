use serde::Deserialize;

pub use self::widgets::Widget;

pub mod widgets;

#[derive(Debug, Deserialize)]
pub struct Spec {
    pub name: String,
    pub child: Widget,
}

fn validate_ident(ident: &str) -> Result<(), ValidationError> {
    for (i, c) in ident.chars().enumerate() {
        if c.is_numeric() && i == 0 {
            return Err(ValidationError::InvalidWidgetId(ident.to_owned()));
        }

        if !c.is_alphanumeric() && c != '_' {
            return Err(ValidationError::InvalidWidgetId(ident.to_owned()));
        }
    }

    Ok(())
}

impl Spec {
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_ident(&self.name)?;
        self.child.validate()?;
        Ok(())
    }

    pub fn deserialize_from_str(s: &str) -> Result<Self, SpecError> {
        let spec: Spec = serde_yaml::from_str(s)?;
        spec.validate()?;
        Ok(spec)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("invalid widget ID '{0}'. Widget IDs must be valid Rust identifiers.")]
    InvalidWidgetId(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_root_deserializes() {
        let s: Spec =
            serde_yaml::from_str(include_str!("../../duit/examples/todos/widgets/root.yml"))
                .unwrap();

        s.validate().unwrap();
    }

    #[test]
    fn invalid_idents() {
        validate_ident("1foo").unwrap_err();
        validate_ident("foo-").unwrap_err();
    }
}
