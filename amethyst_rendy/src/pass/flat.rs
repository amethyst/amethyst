use {
    super::util::*,
    crate::{
        camera::{ActiveCamera, Camera},
        hidden::Hidden,
        mtl::{Material, MaterialDefaults},
        skinning::JointTransforms,
        types::{Backend, Mesh, Texture},
        visibility::Visibility,
    },
    amethyst_assets::{AssetStorage, Handle},
    amethyst_core::{
        specs::{Entity, Join, Read, ReadExpect, ReadStorage, Resources, SystemData},
        transform::GlobalTransform,
    },
    rendy::{
        command::{QueueId, RenderPassEncoder},
        factory::Factory,
        graph::{
            render::{PrepareResult, SimpleGraphicsPipeline, SimpleGraphicsPipelineDesc},
            NodeBuffer, NodeImage,
        },
        hal::pso::{
            BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc, EntryPoint, GraphicsShaderSet,
            Specialization,
        },
        shader::Shader,
    },
    std::collections::HashMap,
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

impl SimpleGraphicsPipelineDesc<Backend, Resources> for DrawFlatDesc {
    type Pipeline = DrawFlat;

    fn load_shader_set<'a>(
        &self,
        storage: &'a mut Vec<Backend::ShaderModule>,
        factory: &mut Factory<Backend>,
        _aux: &mut Resources,
    ) -> GraphicsShaderSet<'a, Backend> {
        storage.clear();

        if self.skinning {
            log::trace!("Loading shader module '{:#?}'", *super::SKINNED_VERTEX);
            storage.push(super::SKINNED_VERTEX.module(factory).unwrap());
        } else {
            log::trace!("Loading shader module '{:#?}'", *super::BASIC_VERTEX);
            storage.push(super::BASIC_VERTEX.module(factory).unwrap());
        };

        log::trace!("Loading shader module '{:#?}'", *super::FLAT_FRAGMEN);
        storage.push(super::FLAT_FRAGMEN.module(factory).unwrap());

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
        factory: &mut Factory<Backend>,
        _queue: QueueId,
        _resources: &mut Resources,
        _buffers: Vec<NodeBuffer<'a, Backend>>,
        _images: Vec<NodeImage<'a, Backend>>,
        _set_layouts: &[Backend::DescriptorSetLayout],
    ) -> Result<DrawFlat, failure::Error> {
        let buffer = factory.create_buffer(1, 1024, rendy::resource::buffer::UniformBuffer)?;

        Ok(DrawFlat {
            skinning: self.skinning,
            buffer,
        })
    }
}

#[derive(Debug)]
pub struct DrawFlat {
    skinning: bool,
    buffer: rendy::resource::Buffer<Backend>,
}

impl SimpleGraphicsPipeline<Backend, Resources> for DrawFlat {
    type Desc = DrawFlatDesc;

    fn prepare(
        &mut self,
        _factory: &Factory<Backend>,
        _queue: QueueId,
        _set_layouts: &[<Backend as rendy::hal::Backend>::DescriptorSetLayout],
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
            Read<'_, AssetStorage<Mesh>>,
            Read<'_, AssetStorage<Texture>>,
            ReadExpect<'_, MaterialDefaults>,
            Option<Read<'_, Visibility>>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, Handle<Mesh>>,
            ReadStorage<'_, Material>,
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
                unimplemented!()
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
        layout: &<Backend as rendy::hal::Backend>::PipelineLayout,
        encoder: RenderPassEncoder<'_, Backend>,
        index: usize,
        aux: &Resources,
    ) {
        unimplemented!()
    }

    fn dispose(self, factory: &mut Factory<Backend>, aux: &mut Resources) {
        unimplemented!()
    }
}
