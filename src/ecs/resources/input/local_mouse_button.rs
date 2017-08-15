use event::MouseButton;

#[derive(Serialize, Deserialize)]
#[serde(remote = "MouseButton")]
pub enum LocalMouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

impl From<LocalMouseButton> for MouseButton {
    fn from(remote: LocalMouseButton) -> MouseButton {
        match remote {
            LocalMouseButton::Left => MouseButton::Left,
            LocalMouseButton::Right => MouseButton::Right,
            LocalMouseButton::Middle => MouseButton::Middle,
            LocalMouseButton::Other(other) => MouseButton::Other(other),
        }
    }
}
