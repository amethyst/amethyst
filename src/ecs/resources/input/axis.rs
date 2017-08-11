use super::Button;

/// Represents an axis made up of digital inputs, like W and S or A and D.
/// Two of these could be analogous to a DPAD.
#[derive(Serialize, Deserialize)]
pub struct Axis {
    /// Positive button, when pressed down axis value will return 1 if `neg` is not pressed down.
    pub pos: Button,
    /// Negative button, when pressed down axis value will return -1 if `pos` is not pressed down.
    pub neg: Button,
}
