use amethyst::input::{is_close_requested, is_key_down};
use amethyst::prelude::*;
use amethyst::renderer::{Event, VirtualKeyCode};

use camera::initialise_camera;
use map::initialise_map;
use tiles::load_sprite_sheet;
use cars::{initialise_cars, load_cars_sprite_sheet};

pub struct IsometryState;

impl<'a, 'b> State<GameData<'a, 'b>> for IsometryState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        let sprite_sheet = load_sprite_sheet(world);
        let cars_sprite_sheet = load_cars_sprite_sheet(world);

        initialise_camera(world);
        initialise_map(world, sprite_sheet);
        initialise_cars(world, cars_sprite_sheet);
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}
