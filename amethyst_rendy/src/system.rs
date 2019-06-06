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
use amethyst_assets::{
    AssetStorage, Handle, HotReloadStrategy, ProcessableAsset, ProcessingState, ThreadPool,
};
use amethyst_core::{
    components::Transform,
    ecs::{Read, ReadExpect, ReadStorage, Resources, RunNow, SystemData, Write, WriteExpect},
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
use std::sync::Arc;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

pub trait GraphCreator<B: Backend> {
    /// Check if graph needs to be rebuilt.
    /// This function is evaluated every frame before running the graph.
    fn rebuild(&mut self, res: &Resources) -> bool;

    /// Retreive configured complete graph builder.
    fn builder(&mut self, factory: &mut Factory<B>, res: &Resources) -> GraphBuilder<B, Resources>;
}

pub struct RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    graph: Option<Graph<B, Resources>>,
    families: Option<Families<B>>,
    graph_creator: G,
}

impl<B, G> RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    pub fn new(graph_creator: G) -> Self {
        Self {
            graph: None,
            families: None,
            graph_creator,
        }
    }
}

type AssetLoadingData<'a, B> = (
    Read<'a, Time>,
    ReadExpect<'a, Arc<ThreadPool>>,
    Option<Read<'a, HotReloadStrategy>>,
    WriteExpect<'a, Factory<B>>,
    Write<'a, AssetStorage<Mesh>>,
    Write<'a, AssetStorage<Texture>>,
    Write<'a, AssetStorage<Material>>,
    ReadExpect<'a, QueueId>,
);

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
    Option<Read<'a, ActiveCamera>>,
    ReadStorage<'a, JointTransforms>,
);

// struct MeshProcessor<B: Backend>(PhantomData<B>);

impl<B, G> RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    fn asset_loading(
        &mut self,
        (
            time,
            pool,
            strategy,
            mut factory,
            mut mesh_storage,
            mut texture_storage,
            mut material_storage,
            queue_id,
        ): AssetLoadingData<'_, B>,
    ) {
        use std::ops::Deref;
        let strategy = strategy.as_ref().map(Deref::deref);

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
            strategy,
        );

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
            strategy,
        );

        material_storage.process(
            |b| {
                #[cfg(feature = "profiler")]
                profile_scope!("process_material");

                ProcessableAsset::process(b)
            },
            time.frame_number(),
            &**pool,
            strategy,
        );
    }

    fn rebuild_graph(&mut self, res: &Resources) {
        #[cfg(feature = "profiler")]
        profile_scope!("rebuild_graph");

        let mut factory = res.fetch_mut::<Factory<B>>();

        if let Some(graph) = self.graph.take() {
            #[cfg(feature = "profiler")]
            profile_scope!("dispose_graph");
            graph.dispose(&mut *factory, res);
        }

        let builder = {
            #[cfg(feature = "profiler")]
            profile_scope!("run_graph_creator");
            self.graph_creator.builder(&mut factory, res)
        };

        let graph = {
            #[cfg(feature = "profiler")]
            profile_scope!("build_graph");
            builder
                .build(&mut factory, self.families.as_mut().unwrap(), res)
                .unwrap()
        };

        self.graph = Some(graph);
    }

    fn run_graph(&mut self, res: &Resources) {
        let mut factory = res.fetch_mut::<Factory<B>>();
        factory.maintain(self.families.as_mut().unwrap());
        self.graph
            .as_mut()
            .unwrap()
            .run(&mut factory, self.families.as_mut().unwrap(), res)
    }
}

impl<'a, B, G> RunNow<'a> for RenderingSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    fn run_now(&mut self, res: &'a Resources) {
        self.asset_loading(SystemData::fetch(res));

        let rebuild = self.graph_creator.rebuild(res);
        if self.graph.is_none() || rebuild {
            self.rebuild_graph(res);
        }
        self.run_graph(res);
    }

    fn setup(&mut self, res: &mut Resources) {
        let config: rendy::factory::Config = Default::default();
        let (factory, families): (Factory<B>, _) = rendy::factory::init(config).unwrap();

        let queue_id = QueueId {
            family: families.family_by_index(0).id(),
            index: 0,
        };

        self.families = Some(families);
        res.insert(factory);
        res.insert(queue_id);
        AssetLoadingData::<B>::setup(res);
        SetupData::setup(res);

        let mat = create_default_mat::<B>(res);
        res.insert(MaterialDefaults(mat));
    }

    fn dispose(mut self: Box<Self>, res: &mut Resources) {
        if let Some(graph) = self.graph.take() {
            let mut factory = res.fetch_mut::<Factory<B>>();
            log::debug!("Dispose graph");
            graph.dispose(&mut *factory, res);
        }

        log::debug!("Unload resources");
        if let Some(mut storage) = res.try_fetch_mut::<AssetStorage<Mesh>>() {
            storage.unload_all();
        }
        if let Some(mut storage) = res.try_fetch_mut::<AssetStorage<Texture>>() {
            storage.unload_all();
        }

        log::debug!("Drop families");
        drop(self.families);
    }
}

fn create_default_mat<B: Backend>(res: &mut Resources) -> Material {
    use crate::mtl::TextureOffset;

    use amethyst_assets::Loader;

    let loader = res.fetch::<Loader>();

    let albedo = load_from_srgba(Srgba::new(0.5, 0.5, 0.5, 1.0));
    let emission = load_from_srgba(Srgba::new(0.0, 0.0, 0.0, 0.0));
    let normal = load_from_linear_rgba(LinSrgba::new(0.5, 0.5, 1.0, 1.0));
    let metallic_roughness = load_from_linear_rgba(LinSrgba::new(0.0, 0.5, 0.0, 0.0));
    let ambient_occlusion = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));
    let cavity = load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 1.0));

    let tex_storage = res.fetch();

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
