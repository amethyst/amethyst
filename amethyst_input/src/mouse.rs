use serde::{Deserialize, Serialize};

/// Mouse axis
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MouseAxis {
    /// The X axis represents moving the mouse left / right
    X,
    /// The Y axis represents the mouse moving up / down
    Y,
}
