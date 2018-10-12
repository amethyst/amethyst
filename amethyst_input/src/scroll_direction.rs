#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}
