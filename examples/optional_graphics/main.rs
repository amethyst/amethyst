use amethyst::{
    input::is_key_down, prelude::*, utils::application_root_dir, window::WindowBundle,
    winit::VirtualKeyCode,
};

struct ExampleState;

impl SimpleState for ExampleState {
    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

struct EmptyState;

impl SimpleState for EmptyState {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let _render_graphics = true;
    #[cfg(feature = "empty")]
    let _render_graphics = false;

    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("examples/optional_graphics/assets/");
    let mut game_data = GameDataBuilder::default();
    let mut game: CoreApplication<GameData>;

    if _render_graphics {
        let display_config_path = app_root.join("examples/optional_graphics/config/display.ron");
        game_data = game_data.with_bundle(WindowBundle::from_config_path(display_config_path)?)?;
        game = Application::new(assets_dir, ExampleState, game_data)?;
    } else {
        game = Application::new(assets_dir, EmptyState, game_data)?;
    }

    game.run();

    Ok(())
}
