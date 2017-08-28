//! Mesh resource handling.



use assets::{Asset, AssetFuture, AssetPtr, AssetSpec, Cache, Context};
use ecs::{Component, VecStorage};
use ecs::rendering::resources::{Factory, FactoryFuture};
use futures::{Async, Future, Poll};
use rayon::ThreadPool;
use renderer::{Mesh, MeshBuilder, Error as RendererError};
use renderer::vertex::*;

/// Wraps `Mesh` into component
#[derive(Clone, Debug)]
pub struct MeshComponent(pub AssetPtr<Mesh, MeshComponent>);

impl MeshComponent {
    /// Create new `MeshComponent` from `Mesh`
    pub fn new(mesh: Mesh) -> Self {
        MeshComponent(AssetPtr::new(mesh))
    }
}

impl AsRef<Mesh> for MeshComponent {
    fn as_ref(&self) -> &Mesh {
        self.0.inner_ref()
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


/// One of known vertices type
pub enum Vertices {
    /// Position and color
    PosColor(Vec<PosColor>),

    /// Position and texture coordinates
    PosTex(Vec<PosTex>),

    /// Position, normal and texture coordinates
    PosNormTex(Vec<PosNormTex>),

    /// Position, normal, tangent and texture coordinates
    PosNormTangTex(Vec<PosNormTangTex>),
}

impl From<Vec<PosColor>> for Vertices {
    fn from(vertcies: Vec<PosColor>) -> Self {
        Vertices::PosColor(vertcies)
    }
}

impl From<Vec<PosTex>> for Vertices {
    fn from(vertcies: Vec<PosTex>) -> Self {
        Vertices::PosTex(vertcies)
    }
}

impl From<Vec<PosNormTex>> for Vertices {
    fn from(vertcies: Vec<PosNormTex>) -> Self {
        Vertices::PosNormTex(vertcies)
    }
}

impl From<Vec<PosNormTangTex>> for Vertices {
    fn from(vertcies: Vec<PosNormTangTex>) -> Self {
        Vertices::PosNormTangTex(vertcies)
    }
}

impl Context for MeshContext {
    type Asset = MeshComponent;
    type Data = Vertices;
    type Error = MeshError;
    type Result = MeshFuture;

    fn category(&self) -> &'static str {
        "mesh"
    }

    fn create_asset(&self, vertices: Vertices, _: &ThreadPool) -> MeshFuture {
        match vertices {
            Vertices::PosColor(vertices) => {
                let mb = MeshBuilder::new(vertices);
                MeshFuture(self.factory.create_mesh(mb))
            }
            Vertices::PosTex(vertices) => {
                let mb = MeshBuilder::new(vertices);
                MeshFuture(self.factory.create_mesh(mb))
            }
            Vertices::PosNormTex(vertices) => {
                let mb = MeshBuilder::new(vertices);
                MeshFuture(self.factory.create_mesh(mb))
            }
            Vertices::PosNormTangTex(vertices) => {
                let mb = MeshBuilder::new(vertices);
                MeshFuture(self.factory.create_mesh(mb))
            }
        }
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
            })
            .and_then(|a| a)
        {
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
