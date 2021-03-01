//! Renderer system

use amethyst_assets::{AssetStorage, DefaultLoader, Loader, ProcessingQueue, ProcessingState};
use amethyst_core::ecs::*;
use derivative::Derivative;
use palette::{LinSrgba, Srgba};
use rendy::{
    command::{Families, QueueId},
    factory::{Factory, ImageState},
    graph::{Graph, GraphBuilder},
    texture::palette::{load_from_linear_rgba, load_from_srgba},
};
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    mtl::Material,
    types::{Backend, Mesh, MeshData, Texture, TextureData},
};

/// Auxiliary data for render graph.
#[allow(missing_debug_implementations)]
pub struct InternalGraphAuxData<'a> {
    /// World
    pub world: &'a World,
    /// Resources
    pub resources: &'a Resources,
}

// FIXME: It is currently impossible to pass types with lifetimes (except for a single reference)
// to auxiliary data structures. It worked before when passing just `World`, but with legion we
// also need to pass `Resources`. To do this we have to transmute `InternalGraphAuxData<'a>` into
// `InternalGraphAuxData<'static>` and ensure that none of the graph nodes store the references.
// Simplified issue: https://github.com/rust-lang/rust/issues/51567
#[allow(missing_docs)]
pub fn make_graph_aux_data(world: &World, resources: &Resources) -> GraphAuxData {
    unsafe { std::mem::transmute(InternalGraphAuxData { world, resources }) }
}

/// Auxiliary data for render graph. Even though it is `'static` any reference inside it must not
/// be saved in any render node. See comments on `make_graph_aux_data`.
pub type GraphAuxData = InternalGraphAuxData<'static>;

/// Graph trait implementation required by consumers. Builds a graph and manages signaling when
/// the graph needs to be rebuilt.
pub trait GraphCreator<B: Backend> {
    /// Check if graph needs to be rebuilt.
    /// This function is evaluated every frame before running the graph.
    fn rebuild(&mut self, world: &World, resources: &Resources) -> bool;

    /// Retrieve configured complete graph builder.
    fn builder(
        &mut self,
        factory: &mut Factory<B>,
        world: &World,
        resources: &Resources,
    ) -> GraphBuilder<B, GraphAuxData>;
}

/// Holds internal state of the rendering system
#[allow(missing_debug_implementations)]
pub struct RenderState<B: Backend, G> {
    /// Renderer graph
    pub graph: Option<Graph<B, GraphAuxData>>,
    /// Device queue families
    pub families: Families<B>,
    /// Graph creator
    pub graph_creator: G,
}

fn rebuild_graph<B, G>(state: &mut RenderState<B, G>, world: &World, resources: &Resources)
where
    B: Backend,
    G: GraphCreator<B>,
{
    #[cfg(feature = "profiler")]
    profile_scope!("rebuild_graph");

    let mut factory = resources.get_mut::<Factory<B>>().unwrap();

    if let Some(graph) = state.graph.take() {
        #[cfg(feature = "profiler")]
        profile_scope!("dispose_graph");
        let aux = make_graph_aux_data(world, resources);
        graph.dispose(&mut *factory, &aux);
    }

    let builder = {
        #[cfg(feature = "profiler")]
        profile_scope!("run_graph_creator");
        state.graph_creator.builder(&mut factory, world, resources)
    };

    let graph = {
        #[cfg(feature = "profiler")]
        profile_scope!("build_graph");
        let aux = make_graph_aux_data(world, resources);
        builder
            .build(&mut factory, &mut state.families, &aux)
            .unwrap()
    };

    state.graph = Some(graph);
}

fn run_graph<B, G>(state: &mut RenderState<B, G>, world: &World, resources: &Resources)
where
    B: Backend,
    G: GraphCreator<B>,
{
    let mut factory = resources.get_mut::<Factory<B>>().unwrap();
    factory.maintain(&mut state.families);
    let aux = make_graph_aux_data(world, resources);
    state
        .graph
        .as_mut()
        .unwrap()
        .run(&mut factory, &mut state.families, &aux)
}

/// Main render function to be executed as thread local system.
/// This should not be used directly and [RenderingBundle] should be used instead.
pub fn render<B, G>(world: &mut World, resources: &mut Resources)
where
    B: Backend,
    G: 'static + GraphCreator<B>,
{
    let mut state = resources.get_mut::<RenderState<B, G>>().unwrap();
    let rebuild = state.graph_creator.rebuild(world, resources);
    if state.graph.is_none() || rebuild {
        rebuild_graph(&mut state, world, resources);
    }
    run_graph(&mut state, world, resources);
}

/// Asset processing system for `Mesh` asset type.
#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct MeshProcessorSystem<B: Backend> {
    pub(crate) _marker: std::marker::PhantomData<B>,
}

impl<B: Backend> System for MeshProcessorSystem<B> {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MeshProcessorSystem")
                .write_resource::<ProcessingQueue<MeshData>>()
                .write_resource::<AssetStorage<Mesh>>()
                .read_resource::<QueueId>()
                .read_resource::<Factory<B>>()
                .build(
                    move |commands,
                          world,
                          (
                        processing_queue,
                        mesh_storage,
                        queue_id,
                        /* time, pool, */ factory,
                    ),
                          _| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("mesh_processor");
                        processing_queue.process(mesh_storage, |b, _, _| {
                            log::trace!("Processing Mesh: {:?}", b);

                            #[cfg(feature = "profiler")]
                            profile_scope!("process_mesh");

                            b.0.build(**queue_id, &factory)
                                .map(B::wrap_mesh)
                                .map(ProcessingState::Loaded)
                                .map_err(|e| e.into())
                        });
                        mesh_storage.process_custom_drop(|_| {});
                    },
                ),
        )
    }
}

/// Asset processing system for `Texture` asset type.
#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct TextureProcessorSystem<B> {
    pub(crate) _marker: std::marker::PhantomData<B>,
}

impl<B: Backend> System for TextureProcessorSystem<B> {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TextureProcessorSystem")
                .write_resource::<ProcessingQueue<TextureData>>()
                .write_resource::<AssetStorage<Texture>>()
                .read_resource::<QueueId>()
                .write_resource::<Factory<B>>()
                .build(
                    move |commands,
                          world,
                          (
                        processing_queue,
                        texture_storage,
                        queue_id,
                        /* time, pool, */ factory,
                    ),
                          _| {
                        #[cfg(feature = "profiler")]
                        profile_scope!("texture_processor");

                        processing_queue.process(texture_storage, |b, _, _| {
                            log::trace!("Processing Texture: {:?}", b);

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
                            .map_err(|e| e.into())
                        });
                        texture_storage.process_custom_drop(|_| {});
                    },
                ),
        )
    }
}

pub(crate) fn create_default_mat<B: Backend>(resources: &Resources) -> Material {
    use crate::mtl::TextureOffset;

    let loader = resources.get::<DefaultLoader>().unwrap();
    let albedo = load_from_srgba(Srgba::new(0.5, 0.5, 0.5, 1.0));
    let emission = load_from_srgba(Srgba::new(0.0, 0.0, 0.0, 0.0));
    let normal = load_from_linear_rgba(LinSrgba::new(0.5, 0.5, 1.0, 1.0));
    let metallic_roughness = load_from_linear_rgba(LinSrgba::new(0.0, 0.5, 0.0, 0.0));
    let ambient_occlusion = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));
    let cavity = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));

    let tex_storage = resources.get::<ProcessingQueue<TextureData>>().unwrap();

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
