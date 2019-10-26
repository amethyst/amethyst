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
    legion::{
        self,
        command::CommandBuffer,
        dispatcher::{ThreadLocal, ThreadLocalObject},
        Resources, SystemBuilder, World,
    },
    timing::Time,
};

use palette::{LinSrgba, Srgba};
use rendy::{
    command::{Families, QueueId},
    factory::{Factory, ImageState},
    graph::{Graph, GraphBuilder},
    texture::palette::{load_from_linear_rgba, load_from_srgba},
};

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

struct RenderState<B: Backend, G> {
    graph: Option<Graph<B, World>>,
    families: Families<B>,
    graph_creator: G,
}

fn rebuild_graph<B, G>(state: &mut RenderState<B, G>, world: &World)
where
    B: Backend,
    G: GraphCreator<B>,
{
    #[cfg(feature = "profiler")]
    profile_scope!("rebuild_graph");

    let mut factory = world.resources.get_mut::<Factory<B>>().unwrap();

    if let Some(graph) = state.graph.take() {
        #[cfg(feature = "profiler")]
        profile_scope!("dispose_graph");
        graph.dispose(&mut *factory, world);
    }

    let builder = {
        #[cfg(feature = "profiler")]
        profile_scope!("run_graph_creator");
        state.graph_creator.builder(&mut factory, world)
    };

    let graph = {
        #[cfg(feature = "profiler")]
        profile_scope!("build_graph");
        builder
            .build(&mut factory, &mut state.families, world)
            .unwrap()
    };

    state.graph = Some(graph);
}

fn run_graph<B, G>(state: &mut RenderState<B, G>, world: &World)
where
    B: Backend,
    G: GraphCreator<B>,
{
    let mut factory = world.resources.get_mut::<Factory<B>>().unwrap();
    factory.maintain(&mut state.families);
    state
        .graph
        .as_mut()
        .unwrap()
        .run(&mut factory, &mut state.families, world)
}

pub fn build_rendering_system<B, G>(
    world: &mut World,
    graph_creator: G,
    families: Families<B>,
) -> Box<dyn ThreadLocal>
where
    B: Backend,
    G: 'static + GraphCreator<B>,
{
    ThreadLocalObject::build(
        RenderState {
            graph: None,
            families,
            graph_creator,
        },
        |state, world| {
            let rebuild = state.graph_creator.rebuild(world);
            if state.graph.is_none() || rebuild {
                rebuild_graph(state, world);
            }
            run_graph(state, world);
        },
        move |state, world| {
            let mut graph = state.graph;
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
            drop(state.families);
        },
    )
}

/// Asset processing system for `Mesh` asset type.
pub fn build_mesh_processor<B: Backend>(
    world: &mut legion::world::World,
) -> Box<dyn legion::schedule::Schedulable> {
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
                        log::trace!("Processing Mesh: {:?}", b);

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

/// Asset processing system for `Mesh` asset type.
pub fn build_texture_processor<B: Backend>(
    world: &mut legion::world::World,
) -> Box<dyn legion::schedule::Schedulable> {
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

                texture_storage.process(
                    |b| {
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
                        .map_err(|e| e.compat().into())
                    },
                    time.frame_number(),
                    &**pool,
                    None, // TODO: Fix strategy optional
                );
            },
        )
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
