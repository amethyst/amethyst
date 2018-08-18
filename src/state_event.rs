use renderer::Event;
use ui::UiEvent;

/// The enum holding the different types of event that can be received in a `State` in the handle_event method.
pub enum StateEvent<E: Send + Sync + 'static> {
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
    /// Custom user events.
    /// To receive events from there, you need to write `E` instances into EventChannel<E>
    Custom(E),
}
