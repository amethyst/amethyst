//! Mesh resource handling.

use futures::{Async, Future, Poll};
use rayon::ThreadPool;


use assets::{Asset, AssetFuture, AssetPtr, AssetSpec, Cache, Context};
use ecs::{Component, VecStorage};
use ecs::resources::{Factory, FactoryFuture};
use renderer::{Mesh, MeshBuilder, Error as RendererError};
use renderer::vertex::PosNormTex;

/// Wraps `Mesh` into component
#[derive(Clone, Debug)]
pub struct MeshComponent(pub AssetPtr<Mesh, MeshComponent>);

impl MeshComponent {
    fn new(mesh: Mesh) -> Self {
        MeshComponent(AssetPtr::new(mesh))
    }
}

impl AsRef<Mesh> for MeshComponent {
    fn as_ref(&self) -> &Mesh {
        self.0.inner()
    }
}

impl AsMut<Mesh> for MeshComponent {
    fn as_mut(&mut self) -> &mut Mesh {
        self.0.inner_mut()
    }
}

impl Component for MeshComponent {
    type Storage = VecStorage<Self>;
}

impl Asset for MeshComponent {
    type Context = MeshContext;
}

/// Error that can occur during mesh creation
pub type MeshError = RendererError;

/// Will be `MeshComponent` result type of `MeshContext::create_asset`
pub struct MeshFuture(FactoryFuture<Mesh, RendererError>);


impl Future for MeshFuture {
    type Item = MeshComponent;
    type Error = MeshError;

    fn poll(&mut self) -> Poll<MeshComponent, RendererError> {
        match self.0.poll() {
            Ok(Async::Ready(mesh)) => Ok(Async::Ready(MeshComponent::new(mesh))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Err(err),
        }
    }
}

/// Context to create meshes from vertices
pub struct MeshContext {
    cache: Cache<AssetFuture<MeshComponent>>,
    factory: Factory,
}

impl MeshContext {
    pub(crate) fn new(factory: Factory) -> Self {
        MeshContext {
            cache: Cache::new(),
            factory: factory,
        }
    }
}

impl Context for MeshContext {
    type Asset = MeshComponent;
    type Data = Vec<PosNormTex>;
    type Error = MeshError;
    type Result = MeshFuture;

    fn category(&self) -> &'static str {
        "mesh"
    }

    fn create_asset(&self, vertices: Vec<PosNormTex>, _: &ThreadPool) -> MeshFuture {
        let mb = MeshBuilder::new(vertices);
        MeshFuture(self.factory.create_mesh(mb))
    }

    fn cache(&self, spec: AssetSpec, asset: AssetFuture<MeshComponent>) {
        self.cache.insert(spec, asset);
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<AssetFuture<MeshComponent>> {
        self.cache.get(spec)
    }

    fn update(&self, spec: &AssetSpec, asset: AssetFuture<MeshComponent>) {
        if let Some(asset) = self.cache
               .access(spec, |a| match a.peek() {
            Some(Ok(a)) => {
                a.0.push_update(asset);
                None
            }
            _ => Some(asset),
        }).and_then(|a| a) {
            self.cache.insert(spec.clone(), asset);
        }
    }

    fn clear(&self) {
        self.cache.retain(|_, a| match a.peek() {
            Some(Ok(a)) => a.0.is_shared(),
            _ => true,
        });
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }
}






