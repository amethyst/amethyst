use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash, Serialize, Deserialize)]
/// Indicates in what direction a mouse wheel scroll event was.
pub enum ScrollDirection {
    /// Scroll was upwards
    ScrollUp,
    /// Scroll was downwards
    ScrollDown,
    /// Scroll was to the left
    ScrollLeft,
    /// Scroll was to the right
    ScrollRight,
}
