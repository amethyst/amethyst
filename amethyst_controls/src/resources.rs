#[derive(Default)]
pub struct WindowFocus {
    pub is_focused: bool,
}

impl WindowFocus {
    pub fn new() -> WindowFocus {
        WindowFocus {
            is_focused: true,
        }
    }
}