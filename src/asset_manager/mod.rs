//! Asset manager used to load assets (like `Mesh`es and `Texture`s).

mod asset;
mod io;

pub use self::asset::{Asset, AssetFormat, AssetStore, AssetStoreError};
pub use self::io::{Import, Export};

use cgmath::{InnerSpace, Vector3};
use dds::DDS;
use gfx::texture::{AaMode, Kind};
use imagefmt::{ColFmt, Image, read_from};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::{env, fs};
use std::io::{Cursor, Read};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::fmt::{Display, Error as FormatError, Formatter};
use std::str;
use std::sync::RwLockReadGuard;
use wavefront_obj::obj::{ObjSet, parse, Primitive};

use ecs::{Allocator, Component, Entity, MaskedStorage, Storage, VecStorage, World};
use ecs::components::{Mesh, Renderable, Texture, TextureLoadData};
use renderer::VertexPosNormal;

use futures::Future;
use futures_cpupool::{CpuPool, CpuFuture};

pub struct AssetManager {
    cpupool: CpuPool,
}

pub struct AssetFuture<T> {
    inner: CpuFuture<T, AssetError>,
}

#[derive(Debug)]
pub enum AssetError {
    StoreError(AssetStoreError),
    ImportError(String),
    ExportError(String),
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager { cpupool: CpuPool::new_num_cpus() }
    }

    pub fn load<T, F>(&self, store: &AssetStore, name: &str) -> AssetFuture<T>
        where T: Asset,
              F: AssetFormat + Import<T::Data>
    {
        AssetFuture {
            inner: self.cpupool.spawn_fn(|x| {
                let bytes = store.read_asset(name)?;
                T::Import::import(bytes).map_err(|x| AssetError::ImportError(x)).map(|x| x.into())
            }),
        }
    }
}

impl<T> Future for AssetFuture<T> {
    type Item = T;
    type Error = AssetError;
}

impl From<AssetStoreError> for AssetError {
    fn from(e: AssetStoreError) -> Self {
        AssetError::StoreError(e)
    }
}

impl Display for AssetError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &AssetError::StoreError(ref x) => write!(f, "IO Error: {}", x),
            &AssetError::ImportError(ref x) => write!(f, "Import Error: {}", x),
            &AssetError::ExportError(ref x) => write!(f, "Export Error: {}", x),
        }
    }
}
