use serde::{Deserialize, Serialize};

/// Struct which holds information about whether the window is focused.
/// Written to by MouseFocusUpdateSystem
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct WindowFocus {
    /// If true then the window is actively focused.
    pub is_focused: bool,
}

impl WindowFocus {
    /// Builds a new WindowFocus resource.
    pub fn new() -> WindowFocus {
        WindowFocus { is_focused: true }
    }
}

/// Resource indicating if the mouse should be grabbed and hidden by the CursorHideSystem
/// when the window is focused.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HideCursor {
    /// If true this system will take control of the cursor.
    pub hide: bool,
}
