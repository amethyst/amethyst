extern crate amethyst;
use amethyst::assets::Loader;
use amethyst::core::cgmath::{Deg, Quaternion, Rotation3, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::core::Time;
use amethyst::ecs::prelude::*;
use amethyst::input::InputBundle;
use amethyst::physics::picking::{primitive, MouseRaySys, PickEventSys, PickSys, Pickable, AB};
use amethyst::prelude::*;
use amethyst::renderer::{
    ActiveCamera, Camera, DirectionalLight, DrawShaded, Light, Pipeline, PosNormTex, RenderBundle,
    Rgba, Shape, Stage,
};
use amethyst::renderer::{Material, MaterialDefaults, MeshHandle};
use amethyst::shrev::{EventChannel, ReaderId};
use amethyst::ui::{UiEvent, UiEventType};

struct Scene;

impl<'a, 'b> State<GameData<'a, 'b>, ()> for Scene {
    fn on_start(&mut self, state: StateData<GameData>) {
        let a_mesh: MeshHandle = {
            let mesh_assets = &state.world.read_resource();
            let loader = state.world.read_resource::<Loader>();
            let verts = Shape::Cube.generate::<Vec<PosNormTex>>(None);
            loader.load_from_data(verts, (), mesh_assets)
        };

        let plain_mat = {
            let mat_assets = &state.world.read_resource();
            let loader = state.world.read_resource::<Loader>();
            let tex = loader.load_from_data([0.3, 0.2, 0.1, 1.].into(), (), mat_assets);
            Material {
                albedo: tex,
                ..state.world.read_resource::<MaterialDefaults>().0.clone()
            }
        };

        let hover_mat = {
            let mat_assets = &state.world.read_resource();
            let loader = state.world.read_resource::<Loader>();
            let tex = loader.load_from_data([0., 1., 0., 1.].into(), (), mat_assets);
            Material {
                albedo: tex,
                ..state.world.read_resource::<MaterialDefaults>().0.clone()
            }
        };

        // Light
        state
            .world
            .create_entity()
            .with({
                let light: Light = DirectionalLight {
                    color: Rgba::white(),
                    direction: [0.5, -1.5, -0.25],
                }
                .into();
                light
            })
            .build();

        // Camera
        let camera = state
            .world
            .create_entity()
            .with(Camera::standard_3d(1., 1.))
            .with(GlobalTransform::default())
            .with(Transform {
                rotation: Quaternion::from_angle_x(Deg(-30.)),
                ..Default::default()
            })
            .build();

        // XXX: An active camera is required by MouseRaySys
        state.world.add_resource(ActiveCamera { entity: camera });

        // Selectable objects
        for x in 0..2 {
            for y in 0..2 {
                let x = x as f32;
                let y = y as f32;
                state
                    .world
                    .create_entity()
                    .with(a_mesh.clone())
                    .with(plain_mat.clone())
                    .with(GlobalTransform::default())
                    .with(Transform {
                        scale: [0.5, 0.5, 0.5].into(),
                        ..Default::default()
                    })
                    .with(Pickable {
                        bounds: AB::A(primitive::Cube::new(2.).into()),
                    })
                    .with(HoverMat::new(hover_mat.clone()))
                    .with(Revolve {
                        center: Vector3::new(x - 0.5, -y - 0.5, -2.5) * 2.,
                        radius: 0.75,
                        per_second: 0.1 + (0.1 * x) + (0.1 * y),
                    })
                    .build();
            }
        }
    }
    fn update(&mut self, state: StateData<GameData>) -> Trans<GameData<'a, 'b>, ()> {
        state.data.update(&state.world);
        Trans::None
    }
}

/// Component which describes the point, radius, and speed of revolutions
struct Revolve {
    center: Vector3<f32>,
    radius: f32,
    per_second: f32,
}
impl Component for Revolve {
    type Storage = HashMapStorage<Self>;
}
/// System which revoles an entity around a point in space
struct RevolveSys;
impl<'s> System<'s> for RevolveSys {
    type SystemData = (
        Read<'s, Time>,
        ReadStorage<'s, Revolve>,
        WriteStorage<'s, Transform>,
    );
    fn run(&mut self, (time, revolves, mut transforms): Self::SystemData) {
        let seconds = time.absolute_time_seconds();
        for (revolve, transform) in (&revolves, &mut transforms).join() {
            let revolutions = revolve.per_second * seconds as f32;
            let angle = revolutions * 2.0 * std::f32::consts::PI;
            transform.translation = Vector3::new(
                revolve.center.x + revolve.radius * angle.cos(),
                revolve.center.y,
                revolve.center.z + revolve.radius * angle.sin(),
            );
        }
    }
}

/// Component which indicates the material a `Pickable` entity has when picked
pub struct HoverMat {
    pub on_hover: Material,
    pub restore_to: Option<Material>,
}
impl HoverMat {
    pub fn new(hover: Material) -> Self {
        HoverMat {
            on_hover: hover,
            restore_to: None,
        }
    }
}
impl Component for HoverMat {
    type Storage = HashMapStorage<Self>;
}
/// System which applies and removes a material when a `Pickable` entity is hovered
pub struct HoverMatSys {
    reader_id: Option<ReaderId<UiEvent>>,
}
impl HoverMatSys {
    pub fn new() -> Self {
        HoverMatSys { reader_id: None }
    }
}
impl<'s> System<'s> for HoverMatSys {
    type SystemData = (
        Write<'s, EventChannel<UiEvent>>,
        WriteStorage<'s, HoverMat>,
        WriteStorage<'s, Material>,
    );

    fn run(&mut self, (mut events, mut hovermat, mut material): Self::SystemData) {
        // one time setup
        if self.reader_id.is_none() {
            self.reader_id = Some(events.register_reader());
        }

        // system
        for ev in events.read(self.reader_id.as_mut().unwrap()) {
            match ev.event_type {
                UiEventType::HoverStart => {
                    hovermat.get_mut(ev.target).map(|hm| {
                        hm.restore_to = material.get(ev.target).map(|m| m.to_owned());
                        material
                            .insert(ev.target, hm.on_hover.clone())
                            .expect("set hover material");
                    });
                }
                UiEventType::HoverStop => {
                    hovermat.get_mut(ev.target).map(|hm| {
                        hm.restore_to.take().map(|m| material.insert(ev.target, m));
                    });
                }
                _ => (),
            }
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.01, 0.02, 0.03, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );
    let state = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, None))?
        .with_bundle(InputBundle::<String, String>::new())?
        // TODO: put this in a bundle
        .with(MouseRaySys, "mouse_ray_sys", &["transform_system"])
        .with(PickSys, "pick_sys", &["mouse_ray_sys"])
        .with(
            PickEventSys::<String, String>::new(),
            "pick_event_sys",
            &["pick_sys"],
        )
        .with(RevolveSys, "example_revolve_sys", &[])
        .with(
            HoverMatSys::new(),
            "example_hovermat_sys",
            &["pick_event_sys"],
        );
    Application::build("", Scene)?.build(state)?.run();
    Ok(())
}
