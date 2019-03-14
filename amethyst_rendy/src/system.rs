//! Renderer system
use crate::{
    camera::{ActiveCamera, Camera},
    hidden::Hidden,
    light::Light,
    mtl::{Material, MaterialDefaults},
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
use palette::Srgba;
use rendy::{
    command::{Families, QueueId},
    factory::{Factory, ImageState},
    graph::{Graph, GraphBuilder},
    hal::{queue::QueueFamilyId, Backend},
    texture::palette::load_from_srgba,
};
use std::sync::Arc;

pub struct RendererSystem<B, F>
where
    B: Backend,
    F: FnOnce(&mut Factory<B>) -> GraphBuilder<B, Resources>,
{
    graph: Option<Graph<B, Resources>>,
    graph_creator: Option<F>,
}

impl<B: Backend, F> RendererSystem<B, F>
where
    B: Backend,
    F: FnOnce(&mut Factory<B>) -> GraphBuilder<B, Resources>,
{
    pub fn new(graph_creator: F) -> Self {
        Self {
            graph: None,
            graph_creator: Some(graph_creator),
        }
    }
}

type AssetLoadingData<'a, B> = (
    Read<'a, Time>,
    ReadExpect<'a, Arc<ThreadPool>>,
    Option<Read<'a, HotReloadStrategy>>,
    WriteExpect<'a, Factory<B>>,
    Write<'a, AssetStorage<Mesh<B>>>,
    Write<'a, AssetStorage<Texture<B>>>,
    Write<'a, AssetStorage<Material<B>>>,
);

type SetupData<'a, B> = (
    ReadStorage<'a, Handle<Mesh<B>>>,
    ReadStorage<'a, Handle<Texture<B>>>,
    ReadStorage<'a, Handle<Material<B>>>,
    ReadStorage<'a, Light>,
    ReadStorage<'a, Camera>,
    ReadStorage<'a, Hidden>,
    Option<Read<'a, Visibility>>,
    Option<Read<'a, ActiveCamera>>,
    ReadStorage<'a, JointTransforms>,
);

// Option<Read<'_, ActiveCamera>>,
// ReadStorage<'_, Camera>,
// Read<'_, AssetStorage<Mesh>>,
// Read<'_, AssetStorage<Texture>>,
// ReadExpect<'_, MaterialDefaults>,
// Option<Read<'_, Visibility>>,
// ReadStorage<'_, Handle<Mesh>>,
// ReadStorage<'_, Material>,

impl<B, F> RendererSystem<B, F>
where
    B: Backend,
    F: FnOnce(&mut Factory<B>) -> GraphBuilder<B, Resources>,
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
        ): AssetLoadingData<'_, B>,
    ) {
        use std::ops::Deref;

        let queue_id = QueueId(QueueFamilyId(0), 0);

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
}

impl<'a, B, F> RunNow<'a> for RendererSystem<B, F>
where
    B: Backend,
    F: FnOnce(&mut Factory<B>) -> GraphBuilder<B, Resources>,
{
    fn run_now(&mut self, res: &'a Resources) {
        self.asset_loading(AssetLoadingData::<B>::fetch(res));

        let mut factory = res.fetch_mut::<Factory<B>>();
        let mut families = res.fetch_mut::<Families<B>>();
        factory.maintain(&mut families);
        self.graph
            .as_mut()
            .unwrap()
            .run(&mut factory, &mut families, res);
    }

    fn setup(&mut self, res: &mut Resources) {
        let config: rendy::factory::Config = Default::default();
        let (mut factory, mut families): (Factory<B>, _) = rendy::factory::init(config).unwrap();

        let graph_creator = self.graph_creator.take().unwrap();

        self.graph = Some(
            graph_creator(&mut factory)
                .build(&mut factory, &mut families, res)
                .unwrap(),
        );

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
    let normal = load_from_srgba(Srgba::new(0.5, 0.5, 1.0, 1.0));
    let metallic = load_from_srgba(Srgba::new(0.0, 0.0, 0.0, 0.0));
    let roughness = load_from_srgba(Srgba::new(0.5, 0.5, 0.5, 0.5));
    let ambient_occlusion = load_from_srgba(Srgba::new(1.0, 1.0, 1.0, 1.0));
    let caveat = load_from_srgba(Srgba::new(1.0, 1.0, 1.0, 1.0));

    let tex_storage = res.fetch();

    let albedo = loader.load_from_data(albedo, (), &tex_storage);
    let emission = loader.load_from_data(emission, (), &tex_storage);
    let normal = loader.load_from_data(normal, (), &tex_storage);
    let metallic = loader.load_from_data(metallic, (), &tex_storage);
    let roughness = loader.load_from_data(roughness, (), &tex_storage);
    let ambient_occlusion = loader.load_from_data(ambient_occlusion, (), &tex_storage);
    let caveat = loader.load_from_data(caveat, (), &tex_storage);

    Material {
        alpha_cutoff: 0.01,
        albedo,
        albedo_offset: TextureOffset::default(),
        emission,
        emission_offset: TextureOffset::default(),
        normal,
        normal_offset: TextureOffset::default(),
        metallic,
        metallic_offset: TextureOffset::default(),
        roughness,
        roughness_offset: TextureOffset::default(),
        ambient_occlusion,
        ambient_occlusion_offset: TextureOffset::default(),
        caveat,
        caveat_offset: TextureOffset::default(),
    }
}
