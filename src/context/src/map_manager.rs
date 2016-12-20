extern crate amethyst_config;
extern crate amethyst_ecs;
extern crate amethyst_renderer;
extern crate yaml_rust;

use std::fs::File;
use std::collections::HashMap;
use std::io::{Error, Read};

use super::Context;
use self::amethyst_ecs::{Component, World};
use self::amethyst_renderer::Light as RendererLight;

pub struct MapError(pub String);

impl From<Error> for MapError {
    fn from(e: Error) -> MapError {
        MapError(format!("MapError: {}", e))
    }
}

impl From<yaml_rust::ScanError> for MapError {
    fn from(e: yaml_rust::ScanError) -> MapError {
        MapError(format!("MapError: {}", e))
    }
}

macro_rules! get_f32 {
    ( $x:expr, $y:expr ) => {
        {
            let error_msg = format!("Expected '{:?}' attribute to be an f32, got `{:?}` instead.", $y, $x[$y]);
            $x[$y].as_f64().ok_or(MapError(error_msg.into()))? as f32
        }
    };
}

macro_rules! get_str {
    ( $x:expr, $y:expr ) => {
        {
            let error_msg = format!("Expected '{:?}' attribute to be a string, got `{:?}` instead.", $y, $x[$y]);
            $x[$y].as_str().ok_or(MapError(error_msg.into()))?
        }
    };
}

macro_rules! get_vec {
    ( $x:expr, $y:expr ) => {
        {
            let error_msg = format!("Expected '{:?}' attribute to be a list, got `{:?}` instead.", $y, $x[$y]);
            $x[$y].as_vec().ok_or(MapError(error_msg.into()))?
        }
    };
}

macro_rules! get_hash {
    ( $x:expr, $y:expr ) => {
        {
            let error_msg = format!("Expected '{:?}' attribute to be a hash, got `{:?}` instead.", $y, $x[$y]);
            $x[$y].as_hash().ok_or(MapError(error_msg.into()))?
        }
    };
}

pub trait MapRenderable {
    fn new(mesh: &str, ka: &str, kd: &str) -> Self;
}

pub trait MapLight {
    fn new(light: RendererLight) -> Self;
}

pub trait MapTransform {
    fn default_transform() -> Self;
}

pub fn load_map<R, L, LT, T>(ctx: &mut Context, world: &mut World, filename: &str) -> Result<(), MapError>
    where R: Component + MapRenderable + Clone,
          L: Component + MapLight + Clone,
          LT: Component + MapTransform,
          T: Component + MapTransform {

    let mut f = File::open(filename)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;

    // Multiple YAML sections are ignored
    let map = yaml_rust::YamlLoader::load_from_str(&s)?[0].clone();

    let textures = get_vec!(map, "textures");
    for tex in textures {
        ctx.asset_manager.create_constant_texture(
            get_str!(tex, "name"),
            [0.20, 0.53, 0.55, 1.0],
        );
    }

    let meshes = get_hash!(map, "meshes");
    let key_name = yaml_rust::Yaml::String("render".into());

    for mesh in get_vec!(meshes, &key_name) {
        let mesh_type = get_str!(mesh, "type");
        match mesh_type {
            "sphere" => {
                let name = get_str!(mesh, "name");
                let uv: Vec<usize> = mesh["uv"]
                    .as_vec()
                    .unwrap()
                    .iter()
                    .map(|i| i.as_i64().unwrap() as usize)
                    .collect();

                ctx.asset_manager.gen_sphere(name, uv[0], uv[1]);
            }
            t @ _ => return Err(MapError(format!("Unknown type {}", t).into())),
        }
    }

    let mut renderables: HashMap<&str, R> = HashMap::new();
    for renderable in get_vec!(map, "renderables") {
        let mesh_name = get_str!(renderable, "mesh");
        renderables.insert(
            mesh_name,
            R::new(
                mesh_name,
                get_str!(renderable, "ambient"),
                get_str!(renderable, "diffuse"),
            ));
    }

    for obj in get_vec!(map, "objects") {
        let render_name = get_str!(obj, "renderable");
        let renderable = renderables
            .get(render_name)
            .ok_or(MapError(format!("Renderable {} could not be found!", render_name).into()))?;

        world.create_now()
            .with(renderable.clone())
            .with(LT::default_transform())
            .with(T::default_transform())
            .build();
    }

    for light in get_vec!(map, "lights") {
        let l = L::new(RendererLight {
            color: [0.96, 0.89, 0.34, 1.0],
            radius: get_f32!(light, "radius"),
            center: [5.0, -10.0, 0.0],
            propagation_constant: get_f32!(light, "propagation_constant"),
            propagation_linear: get_f32!(light, "propagation_linear"),
            propagation_r_square: get_f32!(light, "propagation_r_square"),
        });

        world.create_now().with(l).build();
    }

    Ok(())
}
