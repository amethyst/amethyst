
extern crate cgmath;

use self::cgmath::{Quaternion, Vector3, Matrix3, Matrix4};

use ecs::{Join, Component, VecStorage, Entity, RunArg, Processor};
use context::Context;
use std::sync::{Mutex, Arc};

/// Local position and rotation from parent.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub pos: [f32; 3], // translation vector
    pub rot: [f32; 4], // rotation (radians) vector
    pub scale: [f32; 3], // scale vector
    pub parent: Option<Entity>,
}

impl Transform {
    #[inline]
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let quat: Matrix3<f32> = Quaternion::from(self.rot).into();
        let mut matrix: Matrix4<f32> = (&quat * Matrix3::new(self.scale[0], 0.0, 0.0,
                                                             0.0, self.scale[1], 0.0,
                                                             0.0, 0.0, self.scale[2])).into();
        matrix.w = Vector3::from(self.pos).extend(1.0f32);
        matrix.into()
    }
}

impl Component for Transform {
    type Storage = VecStorage<Transform>;
}

/// Absolute position from origin (0, 0, 0) as well as orientation.
#[derive(Debug, Copy, Clone)]
pub struct GlobalTransform(pub [[f32; 4]; 4]);

impl GlobalTransform {
    pub fn identity() -> Self {
        GlobalTransform(
            [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        )
    }
}

impl Component for GlobalTransform {
    type Storage = VecStorage<GlobalTransform>;
}

pub struct TransformProcessor;
impl Processor<Arc<Mutex<Context>>> for TransformProcessor {
    fn run(&mut self, arg: RunArg, _: Arc<Mutex<Context>>) {
        let (entities, transforms, mut globals) = arg.fetch(|w| {
            (w.entities(), w.read::<Transform>(), w.write::<GlobalTransform>())
        });

        for (entity, transform) in (&entities, &transforms).iter() {
            let combined_transform = match transform.parent {
                Some(parent) => {
                    if let Some(parent_global) = globals.get(parent) {
                        Matrix4::from(parent_global.0) * Matrix4::from(transform.matrix())
                    }
                    else {
                        Matrix4::from(transform.matrix())
                    }
                },
                None => {
                    Matrix4::from(transform.matrix())
                },
            };

            if let Some(global) = globals.get_mut(entity) {
                global.0 = combined_transform.into();
            }
        }
    }
}