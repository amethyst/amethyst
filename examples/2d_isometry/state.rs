use amethyst::prelude::*;

use camera::initialise_camera;
use map::initialise_map;
use tiles::load_sprite_sheet;
use cars::{initialise_cars, load_cars_sprite_sheet};

pub struct IsometryState;

impl<'a, 'b> SimpleState<'a, 'b> for IsometryState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        let sprite_sheet = load_sprite_sheet(world);
        let cars_sprite_sheet = load_cars_sprite_sheet(world);

        initialise_camera(world);
        initialise_map(world, sprite_sheet);
        initialise_cars(world, cars_sprite_sheet);
    }
}
