//! Module containing the Amethyst ui configurations.

// TODO: Complete this with pertinent information.
// Until we have a proper idea of what we want for theming, this will act as a temporary fix to avoid having globals during the refactor.
// File ignored because it is missing in lib.rs. Enable it when working on cursor.rs.

#[derivative(Default)]
#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct UiConfig {
    #[derivative(Default = "0.5")]
    pub cursor_blink_rate: f32,
    pub cursor_color: [f32; 4],
}
