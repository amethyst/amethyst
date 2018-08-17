use renderer::Event;
use ui::UiEvent;

/// The enum holding the different types of event that can be received in a `State` in the handle_event method.
pub enum StateEvent<T: Send + Sync + 'static> {
    Window(Event),
    Ui(UiEvent),
    Custom(T),
}
