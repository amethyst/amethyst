//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Texture`s, `Mesh`es, and `Fragment`s.

extern crate amethyst_renderer;
extern crate gfx_device_gl;
extern crate gfx;
extern crate genmesh;
extern crate cgmath;
extern crate amethyst_ecs;

pub use self::gfx::tex::Kind;
use self::gfx::traits::FactoryExt;
use self::gfx::Factory;
use self::gfx::format::{Formatted, SurfaceTyped};
use self::amethyst_renderer::VertexPosNormal;
use self::amethyst_renderer::target::ColorFormat;
use self::amethyst_ecs::{World, Component, Storage, VecStorage, Allocator, Entity, MaskedStorage};

use self::genmesh::generators::{SphereUV, Cube};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, InnerSpace};

use std::collections::HashMap;
use renderer::{Fragment, FragmentImpl};

use std::ops::{Deref, DerefMut};
use std::any::{Any, TypeId};
use std::sync::RwLockReadGuard;

type AssetTypeId = TypeId;
type SourceTypeId = TypeId;
type LoaderTypeId = TypeId;
type AssetId = Entity;

pub struct Asset<T>(pub T);

impl<T: Any + Send + Sync> Component for Asset<T> {
    type Storage = VecStorage<Asset<T>>;
}

pub trait AssetLoaderRaw<S>: Any {
    fn from_raw(&self, data: &[u8]) -> Option<S>;
}

pub trait AssetLoader<A, S>: Any {
    fn from_data(&mut self, data: S) -> Option<A>;
}

pub trait AssetStore {}

pub trait AssetReadStorage<T> {
    fn read(&self, id: AssetId) -> Option<&T>;
}

impl<'a, T: Any + Send + Sync> AssetReadStorage<T> for Storage<Asset<T>, RwLockReadGuard<'a, Allocator>, RwLockReadGuard<'a, MaskedStorage<Asset<T>>>> {
    fn read(&self, id: AssetId) -> Option<&T> {
        self.get(id).map(|asset| &asset.0)
    }
}

pub struct Assets {
    loaders: HashMap<TypeId, Box<Any>>,
    asset_ids: HashMap<String, AssetId>,
    assets: World,
}

impl Assets {
    fn new() -> Assets {
        Assets {
            loaders: HashMap::new(),
            asset_ids: HashMap::new(),
            assets: World::new(),
        }
    }

    /// Add loader resource to the manager
    pub fn add_loader<T: Any>(&mut self, loader: T) {
        let loader = Box::new(loader);
        self.loaders.insert(TypeId::of::<T>(), loader);
    }

    fn get_loader<T: Any>(&self) -> Option<&T> {
        self.loaders.get(&TypeId::of::<T>())
                    .expect("Unregistered loader")
                    .downcast_ref()
    }

    fn get_loader_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.loaders.get_mut(&TypeId::of::<T>())
                    .expect("Unregistered loader")
                    .downcast_mut()
    }

    /// Register a new asset type
    pub fn register_asset<A: Any + Send + Sync>(&mut self) {
        self.assets.register::<Asset<A>>();
    }

    /// Retrieve the `AssetId` from the asset name
    pub fn id_from_name(&self, name: &str) -> Option<AssetId> {
        self.asset_ids.get(name).map(|id| *id)
    }

    /// Read the storage of all assets for a certain type
    pub fn read_assets<A: Any + Send + Sync>(&self) -> Storage<Asset<A>, RwLockReadGuard<Allocator>, RwLockReadGuard<MaskedStorage<Asset<A>>>> {
        self.assets.read()
    }

    /// Load an asset from data
    pub fn load_asset_from_data<A: Any + Sync + Send, S>(&mut self, name: &str, data: S) -> Option<AssetId> {
        let asset = (self as &mut AssetLoader<A, S>).from_data(data);
        if let Some(asset) = asset {
            Some(self.add_asset(name, asset))
        } else {
            None
        }
    }

    fn add_asset<A: Any + Send + Sync>(&mut self, name: &str, asset: A) -> AssetId {
        let asset_id = self.assets.create_now().with(Asset::<A>(asset)).build();
        self.asset_ids.insert(name.into(), asset_id);
        asset_id
    }
}

// Default implementation for asset loading
// Overwrite if you want to support a new asset type
impl<S> AssetLoaderRaw<S> for Assets {
    default fn from_raw(&self, _: &[u8]) -> Option<S> {
        unimplemented!()
    }
}

impl<A, S> AssetLoader<A, S> for Assets {
    default fn from_data(&mut self, _: S) -> Option<A> {
        unimplemented!()
    }
}

pub struct AssetManager {
    assets: Assets,
    asset_type_ids: HashMap<String, (AssetTypeId, SourceTypeId)>,
    closures: HashMap<(AssetTypeId, SourceTypeId), Box<FnMut(&mut Assets, &str, &[u8]) -> Option<AssetId>>>,
}

impl AssetManager {
    /// Create a new asset manager
    pub fn new() -> AssetManager {
        AssetManager {
            asset_type_ids: HashMap::new(),
            assets: Assets::new(),
            closures: HashMap::new(),
        }
    }

    /// Register a new loading method for a specific asset data type
    pub fn register_loader<A: Any + Send + Sync, S: Any>(&mut self, asset: &str) {
        let asset_id = TypeId::of::<A>();
        let source_id = TypeId::of::<S>();

        self.closures.insert((asset_id, source_id), Box::new(|loader: &mut Assets, name: &str, raw: &[u8]| {
            let data = (loader as &AssetLoaderRaw<S>).from_raw(raw);
            let asset = (loader as &mut AssetLoader<A, S>).from_data(data.unwrap());
            if let Some(asset) = asset {
                Some(loader.add_asset(name, asset))
            } else {
                None
            }
        }));

        self.asset_type_ids.insert(asset.into(), (asset_id, source_id));
    }

    /// Load an asset from raw data
    pub fn load_asset<A: Any + Send + Sync>(&mut self, name: &str, asset_type: &str, raw: &[u8]) -> Option<AssetId> {
        let &(asset_type_id, source_id) = self.asset_type_ids.get(asset_type).expect("Unregistered asset type id");
        assert!(asset_type_id == TypeId::of::<A>());
        let ref mut loader = self.closures.get_mut(&(asset_type_id, source_id)).unwrap();
        loader(&mut self.assets, name, raw)
    }
}

impl Deref for AssetManager {
    type Target = Assets;

    fn deref(&self) -> &Assets {
        &self.assets
    }
}

impl DerefMut for AssetManager {
    fn deref_mut(&mut self) -> &mut Assets {
        &mut self.assets
    }
}

/// An enum with variants representing concrete
/// `Factory` types compatible with different backends.
pub enum FactoryImpl {
    OpenGL { factory: gfx_device_gl::Factory },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

impl AssetLoader<Mesh, Vec<VertexPosNormal>> for Assets {
    fn from_data(&mut self, data: Vec<VertexPosNormal>) -> Option<Mesh> {
        let factory_impl = self.get_loader_mut::<FactoryImpl>().expect("Unable to retrieve factory");
        let mesh_impl = match *factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let (buffer, slice) = factory.create_vertex_buffer_with_slice(&data, ());
                MeshImpl::OpenGL {
                    buffer: buffer,
                    slice: slice,
                }
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => unimplemented!(),
            FactoryImpl::Null => MeshImpl::Null,
        };
        Some(Mesh { mesh_impl: mesh_impl })
    }
}

pub struct TextureLoadData<'a> {
    pub kind: Kind,
    pub raw: &'a [&'a [<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]],
}

impl<'a> AssetLoader<Texture, TextureLoadData<'a>> for Assets {
    fn from_data(&mut self, load_data: TextureLoadData) -> Option<Texture> {
        let factory_impl = self.get_loader_mut::<FactoryImpl>().expect("Unable to retrieve factory");
        let texture_impl = match *factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let shader_resource_view = match factory.create_texture_const::<ColorFormat>(load_data.kind, load_data.raw) {
                    Ok((_, shader_resource_view)) => shader_resource_view,
                    Err(_) => return None,
                };
                let texture = amethyst_renderer::Texture::Texture(shader_resource_view);
                TextureImpl::OpenGL { texture: texture }
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => unimplemented!(),
            FactoryImpl::Null => TextureImpl::Null,
        };
        Some(Texture { texture_impl: texture_impl })
    }
}

impl AssetLoader<Texture, [f32; 4]> for Assets {
    fn from_data(&mut self, color: [f32; 4]) -> Option<Texture> {
        let texture = amethyst_renderer::Texture::Constant(color);
        let texture_impl = TextureImpl::OpenGL { texture: texture };
        Some(Texture { texture_impl: texture_impl })
    }
}

impl AssetManager {
    /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
    /// and the number of vertices from pole to pole (v).
    pub fn gen_sphere(&mut self, name: &str, u: usize, v: usize) {
        let data: Vec<VertexPosNormal> = SphereUV::new(u, v)
            .vertex(|(x, y, z)| {
                VertexPosNormal {
                    pos: [x, y, z],
                    normal: Vector3::new(x, y, z).normalize().into(),
                    tex_coord: [0., 0.],
                }
            })
            .triangulate()
            .vertices()
            .collect();
        self.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(name, data);
    }
    /// Generate and load a cube mesh.
    pub fn gen_cube(&mut self, name: &str) {
        let data: Vec<VertexPosNormal> = Cube::new()
            .vertex(|(x, y, z)| {
                VertexPosNormal {
                    pos: [x, y, z],
                    normal: Vector3::new(x, y, z).normalize().into(),
                    tex_coord: [0., 0.],
                }
            })
            .triangulate()
            .vertices()
            .collect();
        self.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(name, data);
    }
    /// Generate and load a rectangle mesh in XY plane with given `width` and `height`.
    pub fn gen_rectangle(&mut self, name: &str, width: f32, height: f32) {
        let data = vec![
            VertexPosNormal {
                pos: [-width/2., height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 1.],
            },
            VertexPosNormal {
                pos: [-width/2., -height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 0.],
            },
            VertexPosNormal {
                pos: [width/2., -height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [1., 0.],
            },
            VertexPosNormal {
                pos: [width/2., -height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 1.],
            },
            VertexPosNormal {
                pos: [width/2., height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 0.],
            },
            VertexPosNormal {
                pos: [-width/2., height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [1., 0.],
            },
        ];
        self.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(name, data);
    }

    /// Create a constant solid color `Texture` from a specified color.
    pub fn create_constant_texture(&mut self, name: &str, color: [f32; 4]) {
        self.load_asset_from_data::<Texture, [f32; 4]>(name, color);
    }

    /// Construct and return a `Fragment` from previously loaded mesh, ka and kd textures and a transform matrix.
    pub fn get_fragment(&mut self, mesh: &str, ka: &str, kd: &str, transform: [[f32; 4]; 4]) -> Option<Fragment> {
        let mesh_assets = self.read_assets::<Mesh>();
        let texture_assets = self.read_assets::<Texture>();

        let mesh_id = if let Some(id) = self.id_from_name(mesh) {
            id
        } else {
            return None;
        };
        let mesh = match mesh_assets.read(mesh_id) {
            Some(mesh) => mesh,
            None => return None,
        };
        let ka_id = if let Some(id) = self.id_from_name(ka) {
            id
        } else {
            return None;
        };
        let ka = match texture_assets.read(ka_id) {
            Some(ka) => ka,
            None => return None,
        };
        let kd_id = if let Some(id) = self.id_from_name(kd) {
            id
        } else {
            return None;
        };
        let kd = match texture_assets.read(kd_id) {
            Some(kd) => kd,
            None => return None,
        };
        let factory_impl = self.get_loader::<FactoryImpl>().expect("Unable to retrieve factory");
        match *factory_impl {
            FactoryImpl::OpenGL { .. } => {
                let ka = match ka.texture_impl {
                    TextureImpl::OpenGL { ref texture } => texture,
                    #[cfg(windows)]
                    TextureImpl::Direct3D {} => return None,
                    TextureImpl::Null => return None,
                };

                let kd = match kd.texture_impl {
                    TextureImpl::OpenGL { ref texture } => texture,
                    #[cfg(windows)]
                    TextureImpl::Direct3D {} => return None,
                    TextureImpl::Null => return None,
                };

                let (buffer, slice) = match mesh.mesh_impl {
                    MeshImpl::OpenGL { ref buffer, ref slice } => (buffer, slice),
                    #[cfg(windows)]
                    MeshImpl::Direct3D {} => return None,
                    MeshImpl::Null => return None,
                };

                let fragment = amethyst_renderer::Fragment {
                    transform: transform,
                    buffer: buffer.clone(),
                    slice: slice.clone(),
                    ka: ka.clone(),
                    kd: kd.clone(),
                };
                let fragment_impl = FragmentImpl::OpenGL { fragment: fragment };
                Some(Fragment { fragment_impl: fragment_impl })
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {
                unimplemented!();
            }
            FactoryImpl::Null => None,
        }
    }
}

/// An enum with variants representing concrete
/// `Mesh` types compatible with different backends.
#[derive(Clone)]
pub enum MeshImpl {
    OpenGL {
        buffer: gfx::handle::Buffer<gfx_device_gl::Resources, VertexPosNormal>,
        slice: gfx::Slice<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// A wraper around `Buffer` and `Slice` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Mesh {
    mesh_impl: MeshImpl,
}

/// An enum with variants representing concrete
/// `Texture` types compatible with different backends.
#[derive(Clone)]
pub enum TextureImpl {
    OpenGL { texture: amethyst_renderer::Texture<gfx_device_gl::Resources>, },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// A wraper around `Texture` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Texture {
    texture_impl: TextureImpl,
}

#[cfg(test)]
mod tests {
    use super::{Assets, AssetManager, AssetLoader, AssetLoaderRaw};

    struct Foo;
    struct FooLoader;

    impl AssetLoader<Foo, u32> for Assets {
        fn from_data(&mut self, x: u32) -> Option<Foo> {
            println!("{:?}", x);
            if x == 10 {
                Some(Foo)
            } else {
                None
            }
        }
    }

    impl AssetLoaderRaw<u32> for Assets {
        fn from_raw(&self, _: &[u8]) -> Option<u32> {
            let _ = self.get_loader::<FooLoader>();
            Some(10)
        }
    }


    #[test]
    fn loader_resource() {
        let mut assets = AssetManager::new();
        assets.add_loader(0.0f32);
        assert_eq!(Some(&0.0f32), assets.get_loader::<f32>());
    }

    #[test]
    fn load_custom_asset() {
        let mut assets = AssetManager::new();
        assets.register_asset::<Foo>();
        assets.register_loader::<Foo, u32>("foo");
        assets.add_loader::<FooLoader>(FooLoader);

        assert!(assets.load_asset::<Foo>("asset01", "foo", &[0; 2]).is_some());
        assert_eq!(None, assets.load_asset_from_data::<Foo, u32>("asset02", 2));
    }

    #[test]
    #[should_panic]
    fn missing_loader_implementation() {
        let mut assets = AssetManager::new();
        assets.load_asset_from_data::<f32, f32>("test", 1.0);
    }
}