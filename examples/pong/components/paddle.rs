use amethyst::ecs::{Component, DenseVecStorage};

use crate::{PADDLE_HEIGHT, PADDLE_VELOCITY, PADDLE_WIDTH};

#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub width: f32,
    pub height: f32,
    pub velocity: f32,
    pub side: Side,
}

impl Paddle {
    pub fn new(side: Side) -> Paddle {
        Paddle {
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            velocity: PADDLE_VELOCITY,
            side,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}
