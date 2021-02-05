use amethyst::{
    core::transform::Transform,
    ecs::SystemBuilder,
    input::{get_input_axis_simple, InputHandler},
    prelude::*,
};

use crate::pong::{Paddle, Side, ARENA_HEIGHT, PADDLE_HEIGHT};

pub struct PaddleSystem;

impl System for PaddleSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("PaddleSystem")
                .with_query(<(&mut Paddle, &mut Transform)>::query())
                .read_resource::<InputHandler>()
                .read_component::<Paddle>()
                .write_component::<Transform>()
                .build(move |_commands, world, input, query_paddles| {
                    for (paddle, transform) in query_paddles.iter_mut(world) {
                        let movement = match paddle.side {
                            Side::Left => get_input_axis_simple(&Some("left_paddle".into()), input),
                            Side::Right => {
                                get_input_axis_simple(&Some("right_paddle".into()), input)
                            }
                        };
                        let scaled_amount = 1.2 * movement;
                        let paddle_y = transform.translation().y;
                        let new_paddle_y = (paddle_y + scaled_amount)
                            .min(ARENA_HEIGHT - PADDLE_HEIGHT * 0.5)
                            .max(PADDLE_HEIGHT * 0.5);
                        transform.set_translation_y(new_paddle_y);
                        paddle.y = new_paddle_y;
                    }
                }),
        )
    }
}
