use amethyst::{core::{Time, math::Vector3, transform::Transform}, ecs::{ParallelRunnable, System,IntoQuery}, input::InputHandler};
use amethyst_tiles::{MortonEncoder, TileMap};
use legion::SystemBuilder;

use crate::ExampleTile;


pub(crate) struct MapMovementSystem {
    rotate: bool,
    translate: bool,
    vector: Vector3<f32>,
}
impl Default for MapMovementSystem {
    fn default() -> Self {
        Self {
            rotate: false,
            translate: false,
            vector: Vector3::new(100.0, 0.0, 0.0),
        }
    }
}
impl System<'static> for MapMovementSystem {
    fn build(&'static mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MapMovementSystem")
                .read_resource::<Time>()
                .read_resource::<InputHandler>()
                .with_query(<(&TileMap<ExampleTile, MortonEncoder>, &mut Transform)>::query())
                .build( move |_commands, world, (time, input), query| {
                    if input.action_is_down("toggle_rotation").unwrap() {
                        self.rotate ^= true;
                    }
                    if input.action_is_down("toggle_translation").unwrap() {
                        self.translate ^= true;
                    }
                    if self.rotate {
                        for (_, transform) in query.iter_mut(world) {
                            transform.rotate_2d(time.delta_seconds());
                        }
                    }
                    if self.translate {
                        for (_, transform) in query.iter_mut(world) {
                            transform.prepend_translation(self.vector * time.delta_seconds());
                            if transform.translation().x > 500.0 {
                                self.vector = Vector3::new(-100.0, 0.0, 0.0);
                            } else if transform.translation().x < -500.0 {
                                self.vector = Vector3::new(100.0, 0.0, 0.0);
                            }
                        }
                    }
                })
        )
    }
}