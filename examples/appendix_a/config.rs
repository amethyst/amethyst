use amethyst::core::nalgebra::Vector2;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ArenaConfig {
    /// Height of the arena. Used to define Y axis coordinate scheme
    pub height: f32,
    /// Width of the arena. Used to define X axis coordinate scheme
    pub width: f32,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        ArenaConfig {
            height: 100.0,
            width: 100.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BallConfig {
    /// X axis and Y axis velocity of the ball
    pub velocity: Vector2<f32>,
    /// Ball radius in pixels
    pub radius: f32,
    /// RGBA Color of the ball
    pub colour: (f32, f32, f32, f32),
}

impl Default for BallConfig {
    fn default() -> Self {
        BallConfig {
            velocity: Vector2::new(75.0, 50.0),
            radius: 2.5,
            colour: (1.0, 0.0, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PaddlesConfig {
    pub left: PaddleConfig,
    pub right: PaddleConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaddleConfig {
    /// Height, in pixels, of the paddle
    pub height: f32,
    /// Width, in pixels, of the paddle
    pub width: f32,
    /// Velocity of the paddle's vertical movement
    pub velocity: f32,
    /// RGBA colour tuple
    pub colour: (f32, f32, f32, f32),
}

impl Default for PaddleConfig {
    fn default() -> Self {
        PaddleConfig {
            height: 15.0,
            width: 2.5,
            velocity: 75.0,
            colour: (0.0, 0.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PongConfig {
    /// Configuration of the paddles
    pub paddles: PaddlesConfig,
    /// Configuration of the ball
    pub ball: BallConfig,
    /// Configuration of the arena
    pub arena: ArenaConfig,
}
