use amethyst::ecs::{Component, DenseVecStorage};

use crate::{BALL_VELOCITY_X, BALL_VELOCITY_Y, BALL_RADIUS};

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

impl Ball {
    pub fn new() -> Ball {
        Ball {
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
            radius: BALL_RADIUS,
        }
    }

    pub fn reverse_x(&mut self) {
        self.velocity[0] = -self.velocity[0];
    }

    pub fn reverse_y(&mut self) {
        self.velocity[1] = -self.velocity[1];
    }

    pub fn heads_up(&self) -> bool {
        self.velocity[1] > 0.0
    }

    pub fn heads_down(&self) -> bool {
        self.velocity[1] < 0.0
    }

    pub fn heads_right(&self) -> bool {
        self.velocity[0] > 0.0
    }

    pub fn heads_left(&self) -> bool {
        self.velocity[0] < 0.0
    }
}