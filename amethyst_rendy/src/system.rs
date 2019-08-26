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
    ecs::{Read, ReadExpect, ReadStorage, RunNow, System, SystemData, World, Write, WriteExpect},
    timing::Time,
    Hidden, HiddenPropagate,
};
use palette::{LinSrgba, Srgba};
use rendy::{
    command::{Families, QueueId},
    factory::{Factory, ImageState},
    graph::{Graph, GraphBuilder},
    texture::palette::{load_from_linear_rgba, load_from_srgba},
};
use std::{marker::PhantomData, sync::Arc};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Graph trait implementation required by consumers. Builds a graph and manages signaling when
/// the graph needs to be rebuilt.
pub trait GraphCreator<B: Backend> {
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
    families: Option<Families<B>>,
    graph_creator: G,
}

impl<B, G> RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    /// Create a new `RenderingSystem` with the supplied graph via `GraphCreator`
    pub fn new(graph_creator: G) -> Self {
        Self {
            graph: None,
            families: None,
            graph_creator,
        }
    }
}

type SetupData<'a> = (
    ReadStorage<'a, Handle<Mesh>>,
    ReadStorage<'a, Handle<Texture>>,
    ReadStorage<'a, Handle<Material>>,
    ReadStorage<'a, Tint>,
    ReadStorage<'a, Light>,
    ReadStorage<'a, Camera>,
    ReadStorage<'a, Hidden>,
    ReadStorage<'a, HiddenPropagate>,
    ReadStorage<'a, DebugLinesComponent>,
    ReadStorage<'a, Transparent>,
    ReadStorage<'a, Transform>,
    ReadStorage<'a, SpriteRender>,
    Option<Read<'a, Visibility>>,
    Read<'a, ActiveCamera>,
    ReadStorage<'a, JointTransforms>,
);

impl<B, G> RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    fn rebuild_graph(&mut self, world: &World) {
        #[cfg(feature = "profiler")]
        profile_scope!("rebuild_graph");

        let mut factory = world.fetch_mut::<Factory<B>>();

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
                .build(&mut factory, self.families.as_mut().unwrap(), world)
                .unwrap()
        };

        self.graph = Some(graph);
    }

    fn run_graph(&mut self, world: &World) {
        let mut factory = world.fetch_mut::<Factory<B>>();
        factory.maintain(self.families.as_mut().unwrap());
        self.graph
            .as_mut()
            .unwrap()
            .run(&mut factory, self.families.as_mut().unwrap(), world)
    }
}

impl<'a, B, G> RunNow<'a> for RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    fn run_now(&mut self, world: &'a World) {
        let rebuild = self.graph_creator.rebuild(world);
        if self.graph.is_none() || rebuild {
            self.rebuild_graph(world);
        }
        self.run_graph(world);
    }

    fn setup(&mut self, world: &mut World) {
        let config: rendy::factory::Config = Default::default();
        let (factory, families): (Factory<B>, _) = rendy::factory::init(config).unwrap();

        let queue_id = QueueId {
            family: families.family_by_index(0).id(),
            index: 0,
        };

        self.families = Some(families);
        world.insert(factory);
        world.insert(queue_id);

        SetupData::setup(world);

        let mat = create_default_mat::<B>(world);
        world.insert(MaterialDefaults(mat));
    }

    fn dispose(mut self: Box<Self>, world: &mut World) {
        if let Some(graph) = self.graph.take() {
            let mut factory = world.fetch_mut::<Factory<B>>();
            log::debug!("Dispose graph");
            graph.dispose(&mut *factory, world);
        }

        log::debug!("Unload resources");
        if let Some(mut storage) = world.try_fetch_mut::<AssetStorage<Mesh>>() {
            storage.unload_all();
        }
        if let Some(mut storage) = world.try_fetch_mut::<AssetStorage<Texture>>() {
            storage.unload_all();
        }

        log::debug!("Drop families");
        drop(self.families);
    }
}

/// Asset processing system for `Mesh` asset type.
#[derive(Debug, derivative::Derivative)]
#[derivative(Default(bound = ""))]
pub struct MeshProcessorSystem<B: Backend>(PhantomData<B>);
impl<'a, B: Backend> System<'a> for MeshProcessorSystem<B> {
    type SystemData = (
        Write<'a, AssetStorage<Mesh>>,
        ReadExpect<'a, QueueId>,
        Read<'a, Time>,
        ReadExpect<'a, Arc<ThreadPool>>,
        Option<Read<'a, HotReloadStrategy>>,
        ReadExpect<'a, Factory<B>>,
    );

    fn run(
        &mut self,
        (mut mesh_storage, queue_id, time, pool, strategy, factory): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("mesh_processor");

        use std::ops::Deref;
        mesh_storage.process(
            |b| {
                #[cfg(feature = "profiler")]
                profile_scope!("process_mesh");

                b.0.build(*queue_id, &factory)
                    .map(B::wrap_mesh)
                    .map(ProcessingState::Loaded)
                    .map_err(|e| e.compat().into())
            },
            time.frame_number(),
            &**pool,
            strategy.as_ref().map(Deref::deref),
        );
    }
}

/// Asset processing system for `Texture` asset type.
#[derive(Debug, derivative::Derivative)]
#[derivative(Default(bound = ""))]
pub struct TextureProcessorSystem<B: Backend>(PhantomData<B>);
impl<'a, B: Backend> System<'a> for TextureProcessorSystem<B> {
    type SystemData = (
        Write<'a, AssetStorage<Texture>>,
        ReadExpect<'a, QueueId>,
        Read<'a, Time>,
        ReadExpect<'a, Arc<ThreadPool>>,
        Option<Read<'a, HotReloadStrategy>>,
        WriteExpect<'a, Factory<B>>,
    );

    fn run(
        &mut self,
        (mut texture_storage, queue_id, time, pool, strategy, mut factory): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("texture_processor");

        use std::ops::Deref;
        texture_storage.process(
            |b| {
                #[cfg(feature = "profiler")]
                profile_scope!("process_texture");

                b.0.build(
                    ImageState {
                        queue: *queue_id,
                        stage: rendy::hal::pso::PipelineStage::VERTEX_SHADER
                            | rendy::hal::pso::PipelineStage::FRAGMENT_SHADER,
                        access: rendy::hal::image::Access::SHADER_READ,
                        layout: rendy::hal::image::Layout::ShaderReadOnlyOptimal,
                    },
                    &mut factory,
                )
                .map(B::wrap_texture)
                .map(ProcessingState::Loaded)
                .map_err(|e| e.compat().into())
            },
            time.frame_number(),
            &**pool,
            strategy.as_ref().map(Deref::deref),
        );
    }
}

fn create_default_mat<B: Backend>(world: &mut World) -> Material {
    use crate::mtl::TextureOffset;

    use amethyst_assets::Loader;

    let loader = world.fetch::<Loader>();

    let albedo = load_from_srgba(Srgba::new(0.5, 0.5, 0.5, 1.0));
    let emission = load_from_srgba(Srgba::new(0.0, 0.0, 0.0, 0.0));
    let normal = load_from_linear_rgba(LinSrgba::new(0.5, 0.5, 1.0, 1.0));
    let metallic_roughness = load_from_linear_rgba(LinSrgba::new(0.0, 0.5, 0.0, 0.0));
    let ambient_occlusion = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));
    let cavity = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));

    let tex_storage = world.fetch();

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
