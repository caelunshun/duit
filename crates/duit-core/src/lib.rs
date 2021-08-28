//! Code shared between `duit-codegen` and `duit`.

pub mod spec;

use serde::Deserialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
pub enum Align {
    /// Left or top
    Start,
    /// Center or middle
    Center,
    /// Right or bottom
    End,
}

impl Default for Align {
    fn default() -> Self {
        Align::Start
    }
}

/// Indicates an axis used for layout.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Axis {
    Horizontal = 0,
    Vertical = 1,
}

impl Default for Axis {
    fn default() -> Self {
        Axis::Vertical
    }
}
