/// Struct which holds information about whether the window is focused.
/// Written to by MouseFocusUpdateSystem
#[derive(Default)]
pub struct WindowFocus {
    pub is_focused: bool,
}

impl WindowFocus {
    pub fn new() -> WindowFocus {
        WindowFocus { is_focused: true }
    }
}

/// Resource indicating if the mouse should be grabbed and hidden by the CursorHideSystem
/// when the window is focused.
pub struct HideCursor {
	pub hide: bool,
}

impl Default for HideCursor {
	fn default() -> Self {
		HideCursor {
			hide: true,
		}
	}
}
