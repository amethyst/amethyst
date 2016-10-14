//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Texture`s, `Mesh`es, and `Fragment`s.
extern crate amethyst_renderer;
extern crate gfx;

use prefab::{PrefabGenerator, PrefabIndex, PrefabManager};
use resource::{MeshID, TextureID};
use device::{DeviceError, DeviceManager, Mesh, Texture};
use renderer::Fragment;

use std::collections::HashMap;

use asset_manager::gfx::tex::Kind;
use asset_manager::gfx::format::{Formatted, SurfaceTyped};
use self::amethyst_renderer::VertexPosNormal;
use self::amethyst_renderer::target::ColorFormat;

// pub use self::gfx::tex::Kind;
// use self::gfx::traits::FactoryExt;
// use self::gfx::Factory;
// use self::gfx::format::{Formatted, SurfaceTyped};
// use self::amethyst_renderer::VertexPosNormal;
// use self::amethyst_renderer::target::ColorFormat;
// use renderer::{Fragment, FragmentImpl};
//

/// Storage for different `Assets`.
pub trait AssetManager {
    fn add_cube(&mut self, name: &str);
    fn add_sphere(&mut self, name: &str, u: usize, v: usize);
    fn add_rectangle(&mut self, name: &str, width: f32, height: f32);

    fn get_cube(&self, name: &str) -> &Option<Mesh>;
    fn get_sphere(&self, name: &str) -> &Option<Mesh>;
    fn get_rectangle(&self, name: &str) -> &Option<Mesh>;

    fn new(m: &mut PrefabManager) -> HashmapAssetManager {
        HashmapAssetManager::new(m)
    }
}

pub struct HashmapAssetManager<'a> {
    manager: &'a mut PrefabManager,
    cubes: HashMap<String, MeshID>,
    spheres: HashMap<String, MeshID>,
    rectangles: HashMap<String, MeshID>,
}

impl<'a> HashmapAssetManager<'a> {
    fn new(m: &'a mut PrefabManager) -> HashmapAssetManager<'a> {
        HashmapAssetManager {
            manager: m,
            cubes: HashMap::new(),
            spheres: HashMap::new(),
            rectangles: HashMap::new(),
        }
    }
}

impl<'a> AssetManager for HashmapAssetManager<'a> {
    fn add_cube(&mut self, name: &str) {
        let cube = self.manager.gen_cube();
        self.cubes.insert(name.into(), cube);
    }
    fn add_sphere(&mut self, name: &str, u: usize, v: usize) {
        let sphere = self.manager.gen_sphere(u, v);
        self.spheres.insert(name.into(), sphere);
    }
    fn add_rectangle(&mut self, name: &str, width: f32, height: f32) {
        let rectangle = self.manager.gen_rectangle(width, height);
        self.rectangles.insert(name.into(), rectangle);
    }

    fn get_cube(&self, name: &str) -> &Option<Mesh> {
        let ref id = self.cubes[name];
        self.manager.load_cube(id)
    }
    fn get_sphere(&self, name: &str) -> &Option<Mesh> {
        let ref id = self.spheres[name];
        self.manager.load_sphere(id)
    }
    fn get_rectangle(&self, name: &str) -> &Option<Mesh> {
        let ref id = self.rectangles[name];
        self.manager.load_rectangle(id)
    }
}

impl<'a> PrefabGenerator for HashmapAssetManager<'a> {
    fn gen_sphere(&mut self, u: usize, v: usize) -> MeshID {
        self.manager.gen_sphere(u, v)
    }
    fn gen_cube(&mut self) -> MeshID {
        self.manager.gen_cube()
    }
    fn gen_rectangle(&mut self, width: f32, height: f32) -> MeshID {
        self.manager.gen_rectangle(width, height)
    }
}

impl<'a> DeviceManager for HashmapAssetManager<'a> {
    fn load_mesh(&mut self, data: &Vec<VertexPosNormal>) -> MeshID {
        unimplemented!()
    }
    fn get_mesh(&self, id: &MeshID) -> &Option<Mesh> {
        unimplemented!()
    }
    fn load_texture(&mut self, kind: Kind, data: &[&[<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]]) -> Result<TextureID, DeviceError> {
        unimplemented!()
    }
    fn create_constant_texture(&mut self, color: [f32; 4]) -> TextureID {
        unimplemented!()
    }
    fn get_texture(&mut self, id: &TextureID) -> &Option<Texture> {
        unimplemented!()
    }
    fn make_fragment(&mut self, mesh: Mesh, ka: Texture, kd: Texture, transform: [[f32; 4]; 4]) -> Fragment {
        unimplemented!()
    }
}

impl<'a> PrefabIndex for HashmapAssetManager<'a> {
    fn load_sphere(&self, id: &MeshID) -> &Option<Mesh> {
        unimplemented!()
    }
    fn load_cube(&self, id: &MeshID) -> &Option<Mesh> {
        unimplemented!()
    }
    fn load_rectangle(&self, id: &MeshID) -> &Option<Mesh> {
        unimplemented!()
    }

    fn get_fragment(&mut self, m: &MeshID, ka: &TextureID, kd: &TextureID, transform: [[f32; 4]; 4]) -> Option<Fragment> {
        unimplemented!()
    }
}
