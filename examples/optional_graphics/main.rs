use amethyst::{
    input::is_key_down,
    prelude::*,
    renderer::{
        plugins::RenderToWindow, rendy::hal::command::ClearColor, types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::{DisplayConfig, EventLoop},
    winit::event::VirtualKeyCode,
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
        let display_config = DisplayConfig::load(display_config_path)?;
        let event_loop = EventLoop::new();
        game_data = game_data.with_bundle(
            RenderingBundle::<DefaultBackend>::new(display_config, &event_loop).with_plugin(
                RenderToWindow::new().with_clear(ClearColor {
                    float32: [0.0, 0.0, 0.0, 1.0],
                }),
            ),
        )?;
        game = Application::new(assets_dir, ExampleState, game_data)?;
    } else {
        game = Application::new(assets_dir, EmptyState, game_data)?;
    }

    game.run();

    Ok(())
}
