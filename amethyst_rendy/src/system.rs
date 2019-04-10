//! Renderer system
use crate::{
    camera::{ActiveCamera, Camera},
    hidden::{Hidden, HiddenPropagate},
    light::Light,
    mtl::{Material, MaterialDefaults},
    resources::Tint,
    skinning::JointTransforms,
    types::{Mesh, Texture},
    visibility::Visibility,
};
use amethyst_assets::{
    AssetStorage, Handle, HotReloadStrategy, ProcessableAsset, ProcessingState, ThreadPool,
};
use amethyst_core::{
    ecs::{Read, ReadExpect, ReadStorage, Resources, RunNow, SystemData, Write, WriteExpect},
    timing::Time,
};
use palette::{LinSrgba, Srgba};
use rendy::{
    command::{Families, QueueId},
    factory::{Factory, ImageState},
    graph::{Graph, GraphBuilder},
    hal::Backend,
    texture::palette::{load_from_linear_rgba, load_from_srgba},
};
use std::{mem::ManuallyDrop, sync::Arc};

pub trait GraphCreator<B: Backend> {
    /// Check if graph needs to be rebuilt.
    /// This function is evaluated every frame before running the graph.
    fn rebuild(&mut self, res: &Resources) -> bool;

    /// Retreive configured complete graph builder.
    fn builder(&mut self, factory: &mut Factory<B>, res: &Resources) -> GraphBuilder<B, Resources>;
}

pub struct RendererSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    // ManuallyDrop is used as a workaround for inability to dispose the graph properly on exit.
    // This should be removed once we can implement disposal with access to `&Resources`.
    graph: ManuallyDrop<Option<Graph<B, Resources>>>,
    graph_creator: G,
}

impl<B, G> RendererSystem<B, G>
where
    B: Backend,
    G: GraphCreator<B>,
{
    pub fn new(graph_creator: G) -> Self {
        Self {
            graph: ManuallyDrop::new(None),
            graph_creator,
        }
    }
}

type AssetLoadingData<'a, B> = (
    Read<'a, Time>,
    ReadExpect<'a, Arc<ThreadPool>>,
    Option<Read<'a, HotReloadStrategy>>,
    WriteExpect<'a, Factory<B>>,
    ReadExpect<'a, Families<B>>,
    Write<'a, AssetStorage<Mesh<B>>>,
    Write<'a, AssetStorage<Texture<B>>>,
    Write<'a, AssetStorage<Material<B>>>,
);

type SetupData<'a, B> = (
    ReadStorage<'a, Handle<Mesh<B>>>,
    ReadStorage<'a, Handle<Texture<B>>>,
    ReadStorage<'a, Handle<Material<B>>>,
    ReadStorage<'a, Tint>,
    ReadStorage<'a, Light>,
    ReadStorage<'a, Camera>,
    ReadStorage<'a, Hidden>,
    ReadStorage<'a, HiddenPropagate>,
    Option<Read<'a, Visibility>>,
    Option<Read<'a, ActiveCamera>>,
    ReadStorage<'a, JointTransforms>,
);

impl<B, G> RendererSystem<B, G>
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
            families,
            mut mesh_storage,
            mut texture_storage,
            mut material_storage,
        ): AssetLoadingData<'_, B>,
    ) {
        use std::ops::Deref;

        let queue_id = QueueId {
            family: families.family_by_index(0).id(),
            index: 0,
        };

        let strategy = strategy.as_ref().map(Deref::deref);

        mesh_storage.process(
            |b| {
                b.build(queue_id, &mut factory)
                    .map(Mesh)
                    .map(ProcessingState::Loaded)
                    .map_err(|e| e.compat().into())
            },
            time.frame_number(),
            &**pool,
            strategy,
        );

        texture_storage.process(
            |b| {
                b.build(
                    ImageState {
                        queue: queue_id,
                        stage: rendy::hal::pso::PipelineStage::FRAGMENT_SHADER,
                        access: rendy::hal::image::Access::SHADER_READ,
                        layout: rendy::hal::image::Layout::ShaderReadOnlyOptimal,
                    },
                    &mut factory,
                )
                .map(Texture)
                .map(ProcessingState::Loaded)
                .map_err(|e| e.compat().into())
            },
            time.frame_number(),
            &**pool,
            strategy,
        );

        material_storage.process(
            ProcessableAsset::process,
            time.frame_number(),
            &**pool,
            strategy,
        );
    }

    fn rebuild_graph(&mut self, res: &Resources) {
        let mut factory = res.fetch_mut::<Factory<B>>();
        let mut families = res.fetch_mut::<Families<B>>();

        if let Some(graph) = self.graph.take() {
            graph.dispose(&mut *factory, res);
        }

        self.graph = ManuallyDrop::new(Some(
            self.graph_creator
                .builder(&mut factory, res)
                .build(&mut factory, &mut families, res)
                .unwrap(),
        ));
    }

    fn run_graph(&mut self, res: &Resources) {
        let mut factory = res.fetch_mut::<Factory<B>>();
        let mut families = res.fetch_mut::<Families<B>>();
        factory.maintain(&mut families);
        self.graph
            .as_mut()
            .unwrap()
            .run(&mut factory, &mut families, res)
    }
}

impl<'a, B, G> RunNow<'a> for RendererSystem<B, G>
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

        res.insert(factory);
        res.insert(families);
        AssetLoadingData::<B>::setup(res);
        SetupData::<B>::setup(res);

        let mat = create_default_mat::<B>(res);
        res.insert(MaterialDefaults(mat));
    }
}

fn create_default_mat<B: Backend>(res: &mut Resources) -> Material<B> {
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

    let albedo = loader.load_from_data(albedo, (), &tex_storage);
    let emission = loader.load_from_data(emission, (), &tex_storage);
    let normal = loader.load_from_data(normal, (), &tex_storage);
    let metallic_roughness = loader.load_from_data(metallic_roughness, (), &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion, (), &tex_storage);
    let cavity = loader.load_from_data(cavity, (), &tex_storage);

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
