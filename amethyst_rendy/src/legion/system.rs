//! Renderer system
use crate::{
    camera::{ActiveCamera, Camera},
    debug_drawing::DebugLinesComponent,
    light::Light,
    mtl::{Material, MaterialDefaults},
    resources::Tint,
    skinning::JointTransforms,
    sprite::SpriteRender,
    transparent::Transparent,
    types::{Backend, Mesh, Texture},
    visibility::Visibility,
};
use amethyst_assets::{AssetStorage, Handle, HotReloadStrategy, ProcessingState, ThreadPool};
use amethyst_core::{
    components::Transform,
    ecs::{Read, ReadExpect, ReadStorage, RunNow, System, SystemData, Write, WriteExpect},
    legion::{
        self, command::CommandBuffer, storage::ComponentTypeId, system::Schedulable, LegionState,
        Resources, SystemBuilder, SystemDesc, ThreadLocal, World,
    },
    timing::Time,
    Hidden, HiddenPropagate,
};
use amethyst_error::Error;
use palette::{LinSrgba, Srgba};
use rendy::{
    command::{Families, QueueId},
    factory::{Factory, ImageState},
    graph::{Graph, GraphBuilder},
    texture::palette::{load_from_linear_rgba, load_from_srgba},
};
use std::{any::TypeId, marker::PhantomData, sync::Arc};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Graph trait implementation required by consumers. Builds a graph and manages signaling when
/// the graph needs to be rebuilt.
pub trait GraphCreator<B: Backend>: Send {
    /// Check if graph needs to be rebuilt.
    /// This function is evaluated every frame before running the graph.
    fn rebuild(&mut self, world: &World) -> bool;

    /// Retrieve configured complete graph builder.
    fn builder(&mut self, factory: &mut Factory<B>, world: &World) -> GraphBuilder<B, World>;
}

/// Amethyst rendering system
#[allow(missing_debug_implementations)]
pub struct RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    graph: Option<Graph<B, World>>,
    families: Families<B>,
    graph_creator: G,
}

impl<B, G> RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    /// Create a new `RenderingSystem` with the supplied graph via `GraphCreator`
    pub fn new(graph_creator: G, families: Families<B>) -> Self {
        Self {
            graph: None,
            families,
            graph_creator,
        }
    }
}

impl<B, G> RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    fn rebuild_graph(&mut self, world: &World) {
        #[cfg(feature = "profiler")]
        profile_scope!("rebuild_graph");

        let mut factory = world.resources.get_mut::<Factory<B>>().unwrap();

        if let Some(graph) = self.graph.take() {
            #[cfg(feature = "profiler")]
            profile_scope!("dispose_graph");
            graph.dispose(&mut *factory, world);
        }

        let builder = {
            #[cfg(feature = "profiler")]
            profile_scope!("run_graph_creator");
            self.graph_creator.builder(&mut factory, world)
        };

        let graph = {
            #[cfg(feature = "profiler")]
            profile_scope!("build_graph");
            builder
                .build(&mut factory, &mut self.families, world)
                .unwrap()
        };

        self.graph = Some(graph);
    }

    fn run_graph(&mut self, world: &World) {
        let mut factory = world.resources.get_mut::<Factory<B>>().unwrap();
        factory.maintain(&mut self.families);
        self.graph
            .as_mut()
            .unwrap()
            .run(&mut factory, &mut self.families, world)
    }
}

impl<B, G> ThreadLocal for RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    fn run(&mut self, world: &mut World) {
        let rebuild = self.graph_creator.rebuild(world);
        if self.graph.is_none() || rebuild {
            self.rebuild_graph(world);
        }
        self.run_graph(world);
    }
    fn dispose(self, world: &mut World) {
        let mut graph = self.graph;
        if let Some(graph) = graph.take() {
            let mut factory = world.resources.get_mut::<Factory<B>>().unwrap();
            log::debug!("Dispose graph");

            graph.dispose(&mut factory, world);
        }

        log::debug!("Unload resources");
        if let Some(mut storage) = world.resources.get_mut::<AssetStorage<Mesh>>() {
            storage.unload_all();
        }
        if let Some(mut storage) = world.resources.get_mut::<AssetStorage<Texture>>() {
            storage.unload_all();
        }

        log::debug!("Drop families");
        drop(self.families);
    }
}

/// Asset processing system for `Mesh` asset type.
#[derive(Debug, derivative::Derivative)]
#[derivative(Default(bound = ""))]
pub struct MeshProcessorSystemDesc<B: Backend>(PhantomData<B>);
impl<B: Backend> SystemDesc for MeshProcessorSystemDesc<B> {
    fn build(mut self, world: &mut legion::world::World) -> Box<dyn legion::system::Schedulable> {
        SystemBuilder::<()>::new("MeshProcessorSystem")
            .write_resource::<AssetStorage<Mesh>>()
            .read_resource::<QueueId>()
            .read_resource::<Time>()
            .read_resource::<amethyst_core::ArcThreadPool>()
            // .read_resource::<HotReloadStrategy>() // TODO: Optional resources should be OPTIONS instead.
            .read_resource::<Factory<B>>()
            .build(
                move |commands, world, (mesh_storage, queue_id, time, pool, factory), _| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("mesh_processor");

                    mesh_storage.process(
                        |b| {
                            #[cfg(feature = "profiler")]
                            profile_scope!("process_mesh");

                            b.0.build(**queue_id, &factory)
                                .map(B::wrap_mesh)
                                .map(ProcessingState::Loaded)
                                .map_err(|e| e.compat().into())
                        },
                        time.frame_number(),
                        &**pool,
                        None, // TODO: Fix strategy optional
                    )
                },
            )
    }
}

/// Asset processing system for `Mesh` asset type.
#[derive(Debug, derivative::Derivative)]
#[derivative(Default(bound = ""))]
pub struct TextureProcessorSystemDesc<B: Backend>(PhantomData<B>);
impl<B: Backend> SystemDesc for TextureProcessorSystemDesc<B> {
    fn build(mut self, world: &mut legion::world::World) -> Box<dyn legion::system::Schedulable> {
        SystemBuilder::<()>::new("TextureProcessorSystem")
            .write_resource::<AssetStorage<Texture>>()
            .read_resource::<QueueId>()
            .read_resource::<Time>()
            .read_resource::<amethyst_core::ArcThreadPool>()
            // .read_resource::<HotReloadStrategy>() // TODO: Optional resources should be OPTIONS instead.
            .write_resource::<Factory<B>>()
            .build(
                move |commands, world, (texture_storage, queue_id, time, pool, factory), _| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("texture_processor");

                    use std::ops::Deref;
                    texture_storage.process(
                        |b| {
                            #[cfg(feature = "profiler")]
                            profile_scope!("process_texture");

                            b.0.build(
                                ImageState {
                                    queue: **queue_id,
                                    stage: rendy::hal::pso::PipelineStage::VERTEX_SHADER
                                        | rendy::hal::pso::PipelineStage::FRAGMENT_SHADER,
                                    access: rendy::hal::image::Access::SHADER_READ,
                                    layout: rendy::hal::image::Layout::ShaderReadOnlyOptimal,
                                },
                                &mut *factory,
                            )
                            .map(B::wrap_texture)
                            .map(ProcessingState::Loaded)
                            .map_err(|e| e.compat().into())
                        },
                        time.frame_number(),
                        &**pool,
                        None, // TODO: Fix strategy optional
                    );
                },
            )
    }
}

pub(crate) fn create_default_mat<B: Backend>(res: &Resources) -> Material {
    use crate::mtl::TextureOffset;

    use amethyst_assets::Loader;

    let loader = res.get::<Loader>().unwrap();

    let albedo = load_from_srgba(Srgba::new(0.5, 0.5, 0.5, 1.0));
    let emission = load_from_srgba(Srgba::new(0.0, 0.0, 0.0, 0.0));
    let normal = load_from_linear_rgba(LinSrgba::new(0.5, 0.5, 1.0, 1.0));
    let metallic_roughness = load_from_linear_rgba(LinSrgba::new(0.0, 0.5, 0.0, 0.0));
    let ambient_occlusion = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));
    let cavity = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));

    let tex_storage = res.get::<AssetStorage<Texture>>().unwrap();

    let albedo = loader.load_from_data(albedo.into(), (), &tex_storage);
    let emission = loader.load_from_data(emission.into(), (), &tex_storage);
    let normal = loader.load_from_data(normal.into(), (), &tex_storage);
    let metallic_roughness = loader.load_from_data(metallic_roughness.into(), (), &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion.into(), (), &tex_storage);
    let cavity = loader.load_from_data(cavity.into(), (), &tex_storage);

    Material {
        alpha_cutoff: 0.01,
        albedo,
        emission,
        normal,
        metallic_roughness,
        ambient_occlusion,
        cavity,
        uv_offset: TextureOffset::default(),
    }
}
