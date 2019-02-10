use amethyst::{
    assets::{Loader, ProgressCounter},
    core::{
        cgmath::{Array, Deg, Euler, Quaternion, Rotation3, Vector3},
        Transform,
    },
    ecs::prelude::World,
    renderer::{
        AmbientColor, Camera, DirectionalLight, Light, Material, MaterialDefaults, MeshHandle,
        ObjFormat, PngFormat, PointLight, Projection, Rgba,
    },
    ui::{FontHandle, TtfFormat},
};

#[derive(Clone)]
pub struct Assets {
    cube: MeshHandle,
    cone: MeshHandle,
    lid: MeshHandle,
    rectangle: MeshHandle,
    teapot: MeshHandle,
    red: Material,
    white: Material,
    logo: Material,
    pub font: FontHandle,
}

pub fn add_graphics_to_world(world: &mut World) {
    initialise_camera(world);

    let assets = world.read_resource::<Assets>().clone();

    // Add teapot and lid to scene
    for mesh in vec![assets.lid.clone(), assets.teapot.clone()] {
        let mut trans = Transform::default();
        trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
        trans.translation = Vector3::new(5.0, 5.0, 0.0);

        world
            .create_entity()
            .with(mesh)
            .with(assets.red.clone())
            .with(trans)
            .build();
    }

    // Add cube to scene
    let mut trans = Transform::default();
    trans.translation = Vector3::new(5.0, -5.0, 2.0);
    trans.scale = [2.0; 3].into();

    world
        .create_entity()
        .with(assets.cube.clone())
        .with(assets.logo.clone())
        .with(trans)
        .build();

    // Add cone to scene
    let mut trans = Transform::default();
    trans.translation = Vector3::new(-5.0, 5.0, 0.0);
    trans.scale = [2.0; 3].into();

    world
        .create_entity()
        .with(assets.cone.clone())
        .with(assets.white.clone())
        .with(trans)
        .build();

    // Add custom cube object to scene
    let mut trans = Transform::default();
    trans.translation = Vector3::new(-5.0, -5.0, 1.0);
    world
        .create_entity()
        .with(assets.cube.clone())
        .with(assets.red.clone())
        .with(trans)
        .build();

    // Create base rectangle as floor
    let mut trans = Transform::default();
    trans.scale = Vector3::from_value(10.);

    world
        .create_entity()
        .with(assets.rectangle.clone())
        .with(assets.white.clone())
        .with(trans)
        .build();

    let light: Light = PointLight {
        color: [1.0, 1.0, 0.0].into(),
        intensity: 50.0,
        ..PointLight::default()
    }.into();

    // Add lights to scene
    world
        .create_entity()
        .with(light)
        .with(Transform::default())
        .build();

    let light: Light = DirectionalLight {
        color: [0.2; 4].into(),
        direction: [-1.0; 3],
    }.into();

    world.create_entity().with(light).build();
    world.add_resource(AmbientColor(Rgba::from([0.01; 3])));
}

fn initialise_camera(world: &mut World) {
    let mut local = Transform::default();
    local.translation = Vector3::new(0., -20., 10.);
    local.rotation = Quaternion::from_angle_x(Deg(75.)).into();
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(local)
        .build();
}

pub fn load_assets(world: &mut World) -> ProgressCounter {
    let mut progress = ProgressCounter::default();
    let assets = {
        let mesh_storage = world.read_resource();
        let tex_storage = world.read_resource();
        let font_storage = world.read_resource();
        let mat_defaults = world.read_resource::<MaterialDefaults>();
        let loader = world.read_resource::<Loader>();

        let red = loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), &mut progress, &tex_storage);
        let red = Material {
            albedo: red,
            ..mat_defaults.0.clone()
        };

        let white = loader.load_from_data([1.0, 1.0, 1.0, 1.0].into(), &mut progress, &tex_storage);
        let white = Material {
            albedo: white,
            ..mat_defaults.0.clone()
        };

        let logo = Material {
            albedo: loader.load(
                "texture/logo.png",
                PngFormat,
                Default::default(),
                &mut progress,
                &tex_storage,
            ),
            ..mat_defaults.0.clone()
        };

        let cube = loader.load("mesh/cube.obj", ObjFormat, (), &mut progress, &mesh_storage);
        let cone = loader.load("mesh/cone.obj", ObjFormat, (), &mut progress, &mesh_storage);
        let lid = loader.load("mesh/lid.obj", ObjFormat, (), &mut progress, &mesh_storage);
        let teapot = loader.load(
            "mesh/teapot.obj",
            ObjFormat,
            (),
            &mut progress,
            &mesh_storage,
        );
        let rectangle = loader.load(
            "mesh/rectangle.obj",
            ObjFormat,
            (),
            &mut progress,
            &mesh_storage,
        );
        let font = loader.load(
            "font/square.ttf",
            TtfFormat,
            (),
            &mut progress,
            &font_storage,
        );

        Assets {
            cube,
            cone,
            lid,
            rectangle,
            teapot,
            red,
            white,
            logo,
            font,
        }
    };

    world.add_resource(assets);
    progress
}
