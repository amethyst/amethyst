use crate::{
    batch::{GroupIterator, TwoLevelBatch},
    hidden::{Hidden, HiddenPropagate},
    mtl::Material,
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::{SkinnedVertexArgs, VertexArgs},
    resources::Tint,
    skinning::{JointTransforms, PosNormTangTexJoint},
    submodules::{DynamicVertex, EnvironmentSub, MaterialId, MaterialSub, SkinningSub},
    types::Mesh,
    util,
    visibility::Visibility,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{Join, Read, ReadStorage, Resources, SystemData},
    transform::GlobalTransform,
};
use rendy::{
    command::{QueueId, RenderPassEncoder},
    factory::Factory,
    graph::{
        render::{PrepareResult, RenderGroup, RenderGroupDesc},
        BufferAccess, GraphContext, ImageAccess, NodeBuffer, NodeImage,
    },
    hal::{self, device::Device, pso, Backend},
    mesh::{AsVertex, PosNormTangTex},
    shader::Shader,
};
use shred_derive::SystemData;
use smallvec::SmallVec;

/// Draw mesh without lighting
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawPbrDesc {
    skinning: bool,
    transparency: Option<(pso::ColorBlendDesc, Option<pso::DepthStencilDesc>)>,
}

impl DrawPbrDesc {
    /// Create instance of `DrawPbr` pass
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
        color: pso::ColorBlendDesc,
        depth: Option<pso::DepthStencilDesc>,
    ) -> Self {
        self.transparency = Some((color, depth));
        self
    }
}

impl<B: Backend> RenderGroupDesc<B, Resources> for DrawPbrDesc {
    fn buffers(&self) -> Vec<BufferAccess> {
        vec![]
    }
    fn images(&self) -> Vec<ImageAccess> {
        vec![]
    }
    fn depth(&self) -> bool {
        true
    }
    fn colors(&self) -> usize {
        1
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &Resources,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, Resources>>, failure::Error> {
        let shader_vertex_basic = unsafe { super::BASIC_VERTEX.module(factory).unwrap() };
        let shader_fragment = unsafe { super::PBR_FRAGMENT.module(factory).unwrap() };
        let shader_vertex_skinned = if self.skinning {
            Some(unsafe { super::SKINNED_VERTEX.module(factory).unwrap() })
        } else {
            None
        };

        let env = EnvironmentSub::new(factory)?;
        let materials = MaterialSub::new(factory)?;
        let skinning = SkinningSub::new(factory)?;

        let pipeline_layout = unsafe {
            factory.device().create_pipeline_layout(
                vec![
                    env.raw_layout(),
                    materials.raw_layout(),
                    skinning.raw_layout(),
                ],
                None as Option<(pso::ShaderStageFlags, std::ops::Range<u32>)>,
            )
        }?;

        let rect = pso::Rect {
            x: 0,
            y: 0,
            w: framebuffer_width as i16,
            h: framebuffer_height as i16,
        };

        let (vbos, attrs) =
            util::vertex_desc(&[(PosNormTangTex::VERTEX, 0), (VertexArgs::VERTEX, 1)]);

        let pipe_desc = PipelineDescBuilder::new()
            .with_shaders(util::simple_shader_set(
                &shader_vertex_basic,
                Some(&shader_fragment),
            ))
            .with_vertex_buffers(vbos)
            .with_attributes(attrs)
            .with_layout(&pipeline_layout)
            .with_subpass(subpass)
            .with_baked_states(pso::BakedStates {
                viewport: Some(pso::Viewport {
                    rect,
                    depth: 0.0..1.0,
                }),
                scissor: Some(rect),
                blend_color: None,
                depth_bounds: None,
            })
            .with_blender(pso::BlendDesc {
                logic_op: None,
                targets: vec![pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    pso::BlendState::ALPHA,
                )],
            })
            .with_depth_stencil(pso::DepthStencilDesc {
                depth: pso::DepthTest::On {
                    fun: pso::Comparison::Less,
                    write: true,
                },
                depth_bounds: false,
                stencil: pso::StencilTest::Off,
            })
            .with_rasterizer(pso::Rasterizer {
                polygon_mode: pso::PolygonMode::Fill,
                cull_face: pso::Face::BACK,
                front_face: pso::FrontFace::CounterClockwise,
                depth_clamping: false,
                depth_bias: None,
                conservative: false,
            });

        let builder = if self.skinning {
            let (vbos, attrs) = util::vertex_desc(&[
                (PosNormTangTexJoint::VERTEX, 0),
                (SkinnedVertexArgs::VERTEX, 1),
            ]);

            PipelinesBuilder::new()
                .with_pipeline(pipe_desc.clone())
                .with_child_pipeline(
                    0,
                    pipe_desc
                        .with_shaders(util::simple_shader_set(
                            shader_vertex_skinned.as_ref().unwrap(),
                            Some(&shader_fragment),
                        ))
                        .with_vertex_buffers(vbos)
                        .with_attributes(attrs),
                )
        } else {
            PipelinesBuilder::new().with_pipeline(pipe_desc)
        };

        let pipelines = builder.build(factory, None);

        unsafe {
            factory.destroy_shader_module(shader_vertex_basic);
            factory.destroy_shader_module(shader_fragment);
            shader_vertex_skinned.map(|m| factory.destroy_shader_module(m));
        }

        let mut pipelines = pipelines?;
        Ok(Box::new(DrawPbr::<B> {
            pipeline_basic: pipelines.remove(0),
            pipeline_skinned: pipelines.pop(),
            pipeline_layout,
            static_batches: Default::default(),
            skinned_batches: Default::default(),
            env,
            materials,
            skinning,
            models: DynamicVertex::new(),
            skinned_models: DynamicVertex::new(),
        }))
    }
}

#[derive(Debug)]
pub struct DrawPbr<B: Backend> {
    pipeline_basic: B::GraphicsPipeline,
    pipeline_skinned: Option<B::GraphicsPipeline>,
    pipeline_layout: B::PipelineLayout,
    static_batches: TwoLevelBatch<MaterialId, u32, SmallVec<[VertexArgs; 4]>>,
    skinned_batches: TwoLevelBatch<MaterialId, u32, SmallVec<[SkinnedVertexArgs; 4]>>,
    env: EnvironmentSub<B>,
    materials: MaterialSub<B>,
    skinning: SkinningSub<B>,
    models: DynamicVertex<B, VertexArgs>,
    skinned_models: DynamicVertex<B, SkinnedVertexArgs>,
}

#[derive(SystemData)]
struct PbrPassData<'a, B: Backend> {
    mesh_storage: Read<'a, AssetStorage<Mesh<B>>>,
    visibility: Option<Read<'a, Visibility>>,
    hiddens: ReadStorage<'a, Hidden>,
    hiddens_prop: ReadStorage<'a, HiddenPropagate>,
    meshes: ReadStorage<'a, Handle<Mesh<B>>>,
    materials: ReadStorage<'a, Handle<Material<B>>>,
    global_transforms: ReadStorage<'a, GlobalTransform>,
    joints: ReadStorage<'a, JointTransforms>,
    tints: ReadStorage<'a, Tint>,
}

impl<B: Backend> RenderGroup<B, Resources> for DrawPbr<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &Resources,
    ) -> PrepareResult {
        let PbrPassData {
            mesh_storage,
            visibility,
            hiddens,
            hiddens_prop,
            meshes,
            materials,
            global_transforms,
            joints,
            tints,
        } = PbrPassData::<B>::fetch(resources);

        // Prepare environment
        self.env.process(factory, index, resources);
        self.materials.maintain();

        self.static_batches.clear_inner();
        self.skinned_batches.clear_inner();

        let materials_ref = &mut self.materials;
        let skinning_ref = &mut self.skinning;
        let statics_ref = &mut self.static_batches;
        let skinned_ref = &mut self.skinned_batches;

        let static_input = || {
            (
                (&materials, &meshes, &global_transforms, tints.maybe()),
                !&joints,
            )
        };

        let skinned_input = || {
            (
                &materials,
                &meshes,
                &global_transforms,
                tints.maybe(),
                &joints,
            )
        };

        match &visibility {
            None => {
                (static_input(), (!&hiddens, !&hiddens_prop))
                    .join()
                    .map(|(((mat, mesh, tform, tint), _), _)| {
                        ((mat, mesh.id()), VertexArgs::from_object_data(tform, tint))
                    })
                    .for_each_group(|(mat, mesh_id), data| {
                        if mesh_storage.contains_id(mesh_id) {
                            if let Some((mat, _)) = materials_ref.insert(factory, resources, mat) {
                                statics_ref.insert(mat, mesh_id, data.drain(..));
                            }
                        }
                    });

                if self.pipeline_skinned.is_some() {
                    (skinned_input(), (!&hiddens, !&hiddens_prop))
                        .join()
                        .map(|((mat, mesh, tform, tint, joints), _)| {
                            (
                                (mat, mesh.id()),
                                SkinnedVertexArgs::from_object_data(
                                    tform,
                                    tint,
                                    skinning_ref.insert(joints),
                                ),
                            )
                        })
                        .for_each_group(|(mat, mesh_id), data| {
                            if mesh_storage.contains_id(mesh_id) {
                                if let Some((mat, _)) =
                                    materials_ref.insert(factory, resources, mat)
                                {
                                    skinned_ref.insert(mat, mesh_id, data.drain(..));
                                }
                            }
                        });
                }
            }
            Some(visibility) => {
                (static_input(), &visibility.visible_unordered)
                    .join()
                    .map(|(((mat, mesh, tform, tint), _), _)| {
                        ((mat, mesh.id()), VertexArgs::from_object_data(tform, tint))
                    })
                    .for_each_group(|(mat, mesh_id), data| {
                        if mesh_storage.contains_id(mesh_id) {
                            if let Some((mat, _)) = materials_ref.insert(factory, resources, mat) {
                                statics_ref.insert(mat, mesh_id, data.drain(..));
                            }
                        }
                    });

                if self.pipeline_skinned.is_some() {
                    (skinned_input(), &visibility.visible_unordered)
                        .join()
                        .map(|((mat, mesh, tform, tint, joints), _)| {
                            (
                                (mat, mesh.id()),
                                SkinnedVertexArgs::from_object_data(
                                    tform,
                                    tint,
                                    skinning_ref.insert(joints),
                                ),
                            )
                        })
                        .for_each_group(|(mat, mesh_id), data| {
                            if mesh_storage.contains_id(mesh_id) {
                                if let Some((mat, _)) =
                                    materials_ref.insert(factory, resources, mat)
                                {
                                    skinned_ref.insert(mat, mesh_id, data.drain(..));
                                }
                            }
                        });
                }
            }
        };

        self.static_batches.prune();
        self.skinned_batches.prune();

        self.models.write(
            factory,
            index,
            self.static_batches.count() as u64,
            self.static_batches.data(),
        );

        self.skinned_models.write(
            factory,
            index,
            self.skinned_batches.count() as u64,
            self.skinned_batches.data(),
        );
        self.skinning.commit(factory, index);

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &Resources,
    ) {
        let mesh_storage = <Read<'_, AssetStorage<Mesh<B>>>>::fetch(resources);

        encoder.bind_graphics_pipeline(&self.pipeline_basic);
        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);

        if self.models.bind(index, 1, &mut encoder) {
            let mut instances_drawn = 0;
            for (&mat_id, batch_iter) in self.static_batches.iter() {
                if self
                    .materials
                    .bind(&self.pipeline_layout, 1, mat_id, &mut encoder)
                {
                    for (&mesh_id, batch_data) in batch_iter {
                        // This invariant should always be verified before inserting batches in prepare
                        debug_assert!(mesh_storage.contains_id(mesh_id));
                        let Mesh(mesh) = unsafe { mesh_storage.get_by_id_unchecked(mesh_id) };
                        mesh.bind(&[PosNormTangTex::VERTEX], &mut encoder).unwrap();

                        encoder.draw(
                            0..mesh.len(),
                            instances_drawn..instances_drawn + batch_data.len() as u32,
                        );
                        instances_drawn += batch_data.len() as u32;
                    }
                }
            }
        }

        if let Some(pipeline_skinned) = self.pipeline_skinned.as_ref() {
            encoder.bind_graphics_pipeline(pipeline_skinned);

            if self.skinned_models.bind(index, 1, &mut encoder) {
                self.skinning
                    .bind(index, &self.pipeline_layout, 2, &mut encoder);

                let mut instances_drawn = 0;
                for (&mat_id, batch_iter) in self.skinned_batches.iter() {
                    if self
                        .materials
                        .bind(&self.pipeline_layout, 1, mat_id, &mut encoder)
                    {
                        for (&mesh_id, batch_data) in batch_iter {
                            // This invariant should always be verified before inserting batches in prepare
                            debug_assert!(mesh_storage.contains_id(mesh_id));
                            let Mesh(mesh) = unsafe { mesh_storage.get_by_id_unchecked(mesh_id) };
                            mesh.bind(&[PosNormTangTexJoint::VERTEX], &mut encoder)
                                .unwrap();
                            encoder.draw(
                                0..mesh.len(),
                                instances_drawn..instances_drawn + batch_data.len() as u32,
                            );
                            instances_drawn += batch_data.len() as u32;
                        }
                    }
                }
            }
        }
    }

    fn dispose(mut self: Box<Self>, factory: &mut Factory<B>, _aux: &Resources) {
        unsafe {
            factory
                .device()
                .destroy_graphics_pipeline(self.pipeline_basic);
            self.pipeline_skinned.take().map(|pipeline| {
                factory.device().destroy_graphics_pipeline(pipeline);
            });
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}
