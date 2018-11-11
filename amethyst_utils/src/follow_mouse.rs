//! Module containing the utilities related to make entities follow the mouse cursor.

use std::marker::PhantomData;
use std::hash::Hash;
use cgmath::{Vector4, SquareMatrix};
use amethyst_core::{
    Transform, GlobalTransform,
    specs::{System, ReadStorage, WriteStorage, ReadExpect, Join, Component, NullStorage},
};
use amethyst_renderer::{Camera, ScreenDimensions};
use amethyst_input::InputHandler;

/// Adding this `Component` when having the `FollowMouseSystem2D` system
/// active will make the entity to which it is attach follow the mouse cursor.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FollowMouse2D;
impl Component for FollowMouse2D {
    type Storage = NullStorage<Self>;
}

/// Makes any entity having the `FollowMouse2D` component follow the mouse cursor on screen.
/// Only works when using a 2d orthographic camera.
#[derive(Default)]
pub struct FollowMouseSystem2D<A, B> {
    phantom: PhantomData<(A, B)>,
}

impl<'a, A, B> System<'a> for FollowMouseSystem2D<A, B>
where
    A: Send + Sync + Hash + Eq + 'static + Clone,
    B: Send + Sync + Hash + Eq + 'static + Clone,
{
    type SystemData = (
        ReadStorage<'a, FollowMouse2D>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, GlobalTransform>,
        ReadExpect<'a, ScreenDimensions>,
        ReadExpect<'a, InputHandler<A, B>>,
        ReadStorage<'a, Camera>,
    );
fn run(&mut self, (follow_mouses,mut transforms, global_transforms, dimension,input,cameras): Self::SystemData){
        fn fancy_normalize(v: f32, a: f32) -> f32 {
            // [0, a]
            // [-1,1]

            v / (0.5 * a) - 1.0
        }

        let width = dimension.width();
        let height = dimension.height();

        if let Some((x, y)) = input.mouse_position() {
            for (gt, cam) in (&global_transforms, &cameras).join() {
                // TODO: Breaks with multiple cameras :ok_hand:
                let proj = cam.proj;
                let view = gt.0;
                let pv = proj * view;
                let inv = pv.invert().expect("Failed to inverse matrix");
                let tmp: Vector4<f32> = [
                    fancy_normalize(x as f32, width),
                    -fancy_normalize(y as f32, height),
                    0.0,
                    1.0,
                ]
                    .into();
                let res = inv * tmp;

                //println!("Hopefully mouse pos in world: {:?}",res);

                for (mut transform, _) in (&mut transforms, &follow_mouses).join() {
                    transform.translation = [res.x, res.y, transform.translation.z].into();
                    //println!("set pos to {:?}",transform.translation);
                }
            }
        }
    }
}