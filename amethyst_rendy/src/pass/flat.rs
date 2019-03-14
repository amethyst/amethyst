use super::util::*;
use crate::{
    camera::{ActiveCamera, Camera},
    hidden::Hidden,
    mtl::{Material, MaterialDefaults},
    skinning::JointTransforms,
    types::{Mesh, Texture},
    visibility::Visibility,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc},
        NodeBuffer, NodeImage,
    },
    hal::{
        pso::{
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, EntryPoint, GraphicsShaderSet,
            Specialization,
        },
        Backend,
    },
    shader::Shader,
};

/// Draw mesh without lighting
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawFlatDesc {
    skinning: bool,
    transparency: Option<(ColorBlendDesc, Option<DepthStencilDesc>)>,
}

impl DrawFlatDesc {
    /// Create instance of `DrawFlat` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable vertex skinning
    pub fn with_vertex_skinning(mut self) -> Self {
        self.skinning = true;
        self
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        color: ColorBlendDesc,
        depth: Option<DepthStencilDesc>,
    ) -> Self {
        self.transparency = Some((color, depth));
        self
    }
}

impl<B: Backend> SimpleGraphicsPipelineDesc<B, Resources> for DrawFlatDesc {
    type Pipeline = DrawFlat<B>;

    fn load_shader_set<'a>(
        &self,
        storage: &'a mut Vec<B::ShaderModule>,
        factory: &mut Factory<B>,
        _aux: &mut Resources,
    ) -> GraphicsShaderSet<'a, B> {
        storage.clear();

        if self.skinning {
            log::trace!("Loading shader module '{:#?}'", *super::SKINNED_VERTEX);
            storage.push(super::SKINNED_VERTEX.module(factory).unwrap());
        } else {
            log::trace!("Loading shader module '{:#?}'", *super::BASIC_VERTEX);
            storage.push(super::BASIC_VERTEX.module(factory).unwrap());
        };

        log::trace!("Loading shader module '{:#?}'", *super::FLAT_FRAGMENT);
        storage.push(super::FLAT_FRAGMENT.module(factory).unwrap());

        GraphicsShaderSet {
            vertex: EntryPoint {
                entry: "main",
                module: &storage[0],
                specialization: Specialization::default(),
            },
            fragment: Some(EntryPoint {
                entry: "main",
                module: &storage[1],
                specialization: Specialization::default(),
            }),
            hull: None,
            domain: None,
            geometry: None,
        }
    }

    fn colors(&self) -> Vec<ColorBlendDesc> {
        if let Some((color, _)) = self.transparency {
            vec![color]
        } else {
            vec![ColorBlendDesc(ColorMask::ALL, BlendState::ALPHA)]
        }
    }

    fn build<'a>(
        self,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _resources: &mut Resources,
        _buffers: Vec<NodeBuffer<'a, B>>,
        _images: Vec<NodeImage<'a, B>>,
        _set_layouts: &[B::DescriptorSetLayout],
    ) -> Result<DrawFlat<B>, failure::Error> {
        let buffer = factory.create_buffer(1, 1024, rendy::resource::buffer::UniformBuffer)?;

        Ok(DrawFlat {
            skinning: self.skinning,
            buffer,
        })
    }
}

#[derive(Debug)]
pub struct DrawFlat<B: Backend> {
    skinning: bool,
    buffer: rendy::resource::Buffer<B>,
}

impl<B: Backend> SimpleGraphicsPipeline<B, Resources> for DrawFlat<B> {
    type Desc = DrawFlatDesc;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[B::DescriptorSetLayout],
        _index: usize,
        resources: &Resources,
    ) -> PrepareResult {
        let (
            active_camera,
            cameras,
            mesh_storage,
            texture_storage,
            material_defaults,
            visibility,
            hiddens,
            meshes,
            materials,
            globals,
            joints,
        ) = <(
            Option<Read<'_, ActiveCamera>>,
            ReadStorage<'_, Camera>,
            Read<'_, AssetStorage<Mesh<B>>>,
            Read<'_, AssetStorage<Texture<B>>>,
            ReadExpect<'_, MaterialDefaults<B>>,
            Option<Read<'_, Visibility>>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, Handle<Mesh<B>>>,
            ReadStorage<'_, Handle<Material<B>>>,
            ReadStorage<'_, GlobalTransform>,
            ReadStorage<'_, JointTransforms>,
        ) as SystemData>::fetch(resources);

        let defcam = Camera::standard_2d();
        let identity = GlobalTransform::default();
        let camera = active_camera
            .and_then(|ac| {
                cameras
                    .get(ac.entity)
                    .map(|camera| (camera, globals.get(ac.entity).unwrap_or(&identity)))
            })
            .unwrap_or_else(|| {
                (&cameras, &globals)
                    .join()
                    .next()
                    .unwrap_or((&defcam, &identity))
            });

        let mut vertex_args = Vec::new();

        match visibility {
            None => {
                for (joint, mesh, material, global, _) in
                    (joints.maybe(), &meshes, &materials, &globals, !&hiddens).join()
                {
                    let offset = vertex_args.len() * std::mem::size_of::<VertexArgs>();
                    vertex_args.push(VertexArgs::from_camera_and_object(camera, global));
                }
                PrepareResult::DrawRecord
            }
            Some(ref visibility) => {
                for (joint, mesh, material, global, _) in (
                    joints.maybe(),
                    &meshes,
                    &materials,
                    &globals,
                    &visibility.visible_unordered,
                )
                    .join()
                {}

                for &entity in &visibility.visible_ordered {
                    let joint = joints.get(entity).unwrap();
                    let mesh = meshes.get(entity).unwrap();
                    let material = materials.get(entity).unwrap();
                    let global = globals.get(entity).unwrap();
                }
                unimplemented!()
            }
        }
    }

    fn draw(
        &mut self,
        _layout: &B::PipelineLayout,
        _encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &Resources,
    ) {
        unimplemented!()
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &mut Resources) {
        unimplemented!()
    }
}
