//! The style system.

use std::{any::Any, collections::HashMap, rc::Rc, str::FromStr};

use ahash::AHashMap;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize};
use serde_yaml::Value;

const VARIABLE_PREFIX: char = '$';

#[derive(Debug, thiserror::Error)]
pub enum StyleError {
    #[error("missing variable '{0}'")]
    MissingVariable(String),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error("style missing")]
    Missing,
}

/// Manages styles.
#[derive(Debug, Default)]
pub(crate) struct StyleEngine {
    variables: Variables,
    styles: Styles,
    cache: Cache,
}

impl StyleEngine {
    /// Appends a stylesheet from its bytes.
    ///
    /// Existing variables are overriden; existing styles
    /// are augmented.
    pub fn append_sheet(&mut self, sheet_bytes: &[u8]) -> Result<(), StyleError> {
        let sheet: StyleSheet = serde_yaml::from_slice(sheet_bytes)?;
        self.variables.append(&sheet);
        self.styles.append(&sheet, &self.variables)?;
        Ok(())
    }

    /// Gets the style for the given set of style classes.
    pub fn get_style<S: DeserializeOwned + 'static>(
        &mut self,
        classes: &[String],
    ) -> Result<Rc<S>, StyleError> {
        if self.cache.cached_styles.contains_key(classes) {
            return Ok(Rc::downcast(self.cache.get(classes).unwrap()).unwrap());
        }

        self.cache
            .insert(classes.to_vec(), Rc::new(self.create_style::<S>(classes)?));
        Ok(Rc::downcast(self.cache.get(classes).unwrap()).unwrap())
    }

    fn create_style<S: DeserializeOwned>(&self, classes: &[String]) -> Result<S, StyleError> {
        let mut value = Value::Null;
        for style in self.styles.matching_styles(classes) {
            merge_values(&mut value, style.value.clone());
        }
        let x: S = serde_yaml::from_value(value)?;
        Ok(x)
    }
}

/// Stores variables accessible to all stylesheets.
#[derive(Debug, Default)]
struct Variables {
    variables: HashMap<String, Value>,
}

impl Variables {
    pub fn get(&self, var: &str) -> Result<&Value, StyleError> {
        self.variables
            .get(var)
            .ok_or_else(|| StyleError::MissingVariable(var.to_owned()))
    }

    pub fn append(&mut self, sheet: &StyleSheet) {
        for (name, value) in &sheet.variables {
            self.variables.insert(name.clone(), value.clone());
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum ClassQueryParseError {
    #[error("no query specified (empty / blank string)")]
    Empty,
}

/// A class query specifies to which elements a style applies.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClassQuery {
    predicate: QueryPredicate,
}

impl FromStr for ClassQuery {
    type Err = ClassQueryParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut predicate = None;
        while !s.trim().is_empty() {
            s = s.trim();
            if let Some(and) = s.strip_prefix('&') {
                let and = and.trim();
                let end = and
                    .char_indices()
                    .find_map(|(p, c)| (!is_valid_class_char(c)).then(|| p))
                    .unwrap_or_else(|| and.len());
                let p = QueryPredicate::from_str(&and[..end])?;
                if let Some(pred) = predicate {
                    predicate = Some(QueryPredicate::And(Box::new(p), Box::new(pred)));
                } else {
                    predicate = Some(p);
                }
                s = &and[end..];
            } else if let Some(or) = s.strip_prefix('|') {
                let or = or.trim();
                let end = or
                    .char_indices()
                    .find_map(|(p, c)| (!is_valid_class_char(c)).then(|| p))
                    .unwrap_or_else(|| or.len());
                let p = QueryPredicate::from_str(&or[..end])?;
                if let Some(pred) = predicate {
                    predicate = Some(QueryPredicate::Or(Box::new(p), Box::new(pred)));
                } else {
                    predicate = Some(p);
                }
                s = &or[end..];
            } else {
                let end = s
                    .char_indices()
                    .find_map(|(p, c)| (!is_valid_class_char(c)).then(|| p))
                    .unwrap_or_else(|| s.len());
                let p = QueryPredicate::from_str(&s[..end])?;
                if let Some(pred) = predicate {
                    predicate = Some(QueryPredicate::And(Box::new(p), Box::new(pred)));
                } else {
                    predicate = Some(p);
                }
                s = &s[end..];
            }
        }

        Ok(Self {
            predicate: predicate.ok_or(ClassQueryParseError::Empty)?,
        })
    }
}

impl<'de> Deserialize<'de> for ClassQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

fn is_valid_class_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '(' || c == ')'
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum QueryPredicate {
    And(Box<QueryPredicate>, Box<QueryPredicate>),
    Or(Box<QueryPredicate>, Box<QueryPredicate>),

    /// The element must have a certain class.
    Class(String),
}

impl FromStr for QueryPredicate {
    type Err = ClassQueryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(QueryPredicate::Class(s.trim().to_owned()))
    }
}

impl QueryPredicate {
    fn matches(&self, classes: &[String]) -> bool {
        match self {
            QueryPredicate::And(a, b) => a.matches(classes) && b.matches(classes),
            QueryPredicate::Or(a, b) => a.matches(classes) || b.matches(classes),
            QueryPredicate::Class(c) => classes.iter().any(|class| class == c),
        }
    }
}

/// A single style sheet.
#[derive(Debug, Deserialize)]
struct StyleSheet {
    #[serde(default)]
    variables: HashMap<String, Value>,
    #[serde(default)]
    styles: IndexMap<ClassQuery, Value>,
}

/// Stores styles based on class queries.
#[derive(Debug, Default)]
struct Styles {
    styles: Vec<(ClassQuery, Style)>,
}

impl Styles {
    /// Gets all styles matching the given classes.
    pub fn matching_styles<'a>(
        &'a self,
        classes: &'a [String],
    ) -> impl Iterator<Item = &'a Style> + 'a {
        // TODO: avoid linear search
        self.styles.iter().filter_map(move |(query, style)| {
            if query.predicate.matches(classes) {
                Some(style)
            } else {
                None
            }
        })
    }

    /// Appends a style sheet.
    ///
    /// Variables need to be appended first.
    pub fn append(&mut self, sheet: &StyleSheet, variables: &Variables) -> Result<(), StyleError> {
        for (query, style_value) in &sheet.styles {
            let style = Style::new(style_value.clone(), variables)?;
            self.styles.push((query.clone(), style));
        }
        Ok(())
    }
}

/// A style, stored as a YAML value.
///
/// Variables have been applied.
#[derive(Debug)]
struct Style {
    value: Value,
}

impl Style {
    pub fn new(mut value: Value, variables: &Variables) -> Result<Self, StyleError> {
        apply_variables(&mut value, variables)?;
        Ok(Self { value })
    }
}

fn apply_variables(value: &mut Value, variables: &Variables) -> Result<(), StyleError> {
    // Recursively traverse the tree, looking for string values starting
    // with the variable prefix character '$.'
    match value {
        Value::String(s) => {
            if let Some(var) = s.strip_prefix(VARIABLE_PREFIX) {
                *value = variables.get(var)?.clone();
            }
        }
        Value::Mapping(mapping) => {
            for (_, child_value) in mapping.iter_mut() {
                apply_variables(child_value, variables)?;
            }
        }
        Value::Sequence(s) => {
            for child_value in s {
                apply_variables(child_value, variables)?;
            }
        }
        _ => (),
    }
    Ok(())
}

/// A cache of deserialized style values for sets of style classes.
#[derive(Default)]
struct Cache {
    cached_styles: AHashMap<Vec<String>, Rc<dyn Any>>,
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Cache");
        s.field("size", &self.cached_styles.len());
        s.finish()
    }
}

impl Cache {
    /// Gets the cached style for a set of classes, if any.
    pub fn get(&self, classes: &[String]) -> Option<Rc<dyn Any>> {
        self.cached_styles.get(classes).cloned()
    }

    /// Inserts a cached style for a class set.
    pub fn insert(&mut self, classes: Vec<String>, style: Rc<dyn Any>) {
        self.cached_styles.insert(classes, style);
    }
}

fn merge_values(bottom: &mut Value, top: Value) {
    match (bottom, top) {
        (Value::Mapping(bottom_map), Value::Mapping(top_map)) => {
            for (key, top_value) in top_map {
                match bottom_map.get_mut(&key) {
                    Some(existing_value) => merge_values(existing_value, top_value),
                    None => {
                        bottom_map.insert(key, top_value);
                    }
                }
            }
        }
        (bottom, top) => *bottom = top,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_class_query() {
        let query = ClassQuery::from_str("class").unwrap();
        assert_eq!(query.predicate, QueryPredicate::Class("class".to_owned()));
    }

    #[test]
    fn and_class_query() {
        let query = ClassQuery::from_str("class1 & class2").unwrap();
        assert_eq!(
            query.predicate,
            QueryPredicate::And(
                Box::new(QueryPredicate::Class("class2".to_owned())),
                Box::new(QueryPredicate::Class("class1".to_owned())),
            )
        );
    }

    #[test]
    fn or_class_query() {
        let query = ClassQuery::from_str("class1 | class2").unwrap();
        assert_eq!(
            query.predicate,
            QueryPredicate::Or(
                Box::new(QueryPredicate::Class("class2".to_owned())),
                Box::new(QueryPredicate::Class("class1".to_owned())),
            )
        );
    }
}
