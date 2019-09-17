#![allow(clippy::default_trait_access, clippy::use_self)]
#![allow(unused_imports, unused_variables)]

use amethyst_core::{
    ecs::{
        DispatcherBuilder, Entities, Join, Read, ReadExpect, ReadStorage, System, SystemData, World,
    },
    geometry::{Plane, Ray},
    math::{self, convert, Matrix4, Point2, Point3, Vector2, Vector3, Vector4},
    transform::Transform,
    Hidden,
};

use amethyst_assets::{AssetStorage, Handle};
use amethyst_rendy::{
    batch::{GroupIterator, OneLevelBatch, OrderedOneLevelBatch},
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    camera::{ActiveCamera, Camera, Projection},
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::{IntoPod, SpriteArgs},
    rendy::{
        command::{QueueId, RenderPassEncoder},
        factory::Factory,
        graph::{
            render::{PrepareResult, RenderGroup, RenderGroupDesc},
            GraphContext, NodeBuffer, NodeImage,
        },
        hal::{
            self,
            device::Device,
            pso::{self, ShaderStageFlags},
        },
        mesh::AsVertex,
        shader::{Shader, ShaderSetBuilder, SpirvShader},
    },
    sprite::{SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibility,
    submodules::{
        gather::CameraGatherer, DynamicVertexBuffer, FlatEnvironmentSub, TextureId, TextureSub,
    },
    types::{Backend, Texture},
    util,
};
use amethyst_window::ScreenDimensions;
use derivative::Derivative;
use std::marker::PhantomData;

use crate::{
    iters::Region,
    map::{Map, MapStorage, Tile, TileMap},
    CoordinateEncoder, MortonEncoder2D,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

lazy_static::lazy_static! {
    static ref TILES_VERTEX: SpirvShader = SpirvShader::new(
        include_bytes!("../compiled/tiles.vert.spv").to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref TILES_FRAGMENT: SpirvShader = SpirvShader::new(
        include_bytes!("../compiled/tiles.frag.spv").to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );

    static ref SHADERS: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&*TILES_VERTEX).unwrap()
        .with_fragment(&*TILES_FRAGMENT).unwrap();
}

/// Trait to describe how rendering tiles may be culled for the tilemap to render
pub trait DrawTiles2DBounds: 'static + std::fmt::Debug + Send + Sync {
    /// Returns the region to render the tiles
    fn bounds<T: Tile, E: CoordinateEncoder>(map: &TileMap<T, E>, world: &World) -> Region;
}

/// Draw opaque sprites without lighting.
#[derive(Clone, PartialEq, Derivative)]
#[derivative(Default(bound = ""), Debug(bound = ""))]
pub struct DrawTiles2DDesc<
    T: Tile,
    E: CoordinateEncoder,
    Z: DrawTiles2DBounds = DrawTiles2DBoundsDefault,
> {
    #[derivative(Debug = "ignore")]
    _marker: PhantomData<(T, E, Z)>,
}

impl<B: Backend, T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds> RenderGroupDesc<B, World>
    for DrawTiles2DDesc<T, E, Z>
{
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("build");

        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_tiles_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawTiles2D::<B, T, E, Z> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
            _marker: PhantomData::default(),
            change: Default::default(),
        }))
    }
}

/// `RenderGroup` providing culling, drawing and transparency functionality for 3D `TileMap` components.
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct DrawTiles2D<
    B: Backend,
    T: Tile,
    E: CoordinateEncoder,
    Z: DrawTiles2DBounds = DrawTiles2DBoundsDefault,
> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OrderedOneLevelBatch<TextureId, SpriteArgs>,
    change: util::ChangeDetection,

    #[derivative(Debug = "ignore")]
    _marker: PhantomData<(T, E, Z)>,
}

impl<B: Backend, T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds> RenderGroup<B, World>
    for DrawTiles2D<B, T, E, Z>
{
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare");

        let mut changed = false;
        let (sprite_sheet_storage, tex_storage, hiddens, tile_maps) = <(
            Read<'_, AssetStorage<SpriteSheet>>,
            Read<'_, AssetStorage<Texture>>,
            ReadStorage<'_, Hidden>,
            ReadStorage<'_, TileMap<T, E>>,
        )>::fetch(world);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.swap_clear();

        self.env.process(factory, index, world);

        if let Some((tile_map, _)) = (&tile_maps, !&hiddens).join().next() {
            Z::bounds(tile_map, world)
                .iter()
                .filter_map(|coord| {
                    let tile = tile_map.get(&coord)?;
                    let sprite_number = tile.sprite(coord, world)?;
                    let mut world_transform = Transform::default();
                    world_transform.set_translation(tile_map.to_world(&coord));

                    let (batch_data, texture) = {
                        let sprite_sheet =
                            sprite_sheet_storage.get(tile_map.sprite_sheet.as_ref().unwrap())?;
                        if !tex_storage.contains(&sprite_sheet.texture) {
                            return None;
                        }

                        let sprite = &sprite_sheet.sprites[sprite_number];

                        let transform = world_transform.matrix();
                        let dir_x = transform.column(0) * sprite.width;
                        let dir_y = transform.column(1) * -sprite.height;
                        let pos = transform
                            * Vector4::new(-sprite.offsets[0], -sprite.offsets[1], 0.0, 1.0);

                        let color = tile.tint(coord, world);
                        let (r, g, b, a) = color.into_components();

                        Some((
                            SpriteArgs {
                                dir_x: dir_x.xy().into_pod(),
                                dir_y: dir_y.xy().into_pod(),
                                pos: pos.xy().into_pod(),
                                u_offset: [sprite.tex_coords.left, sprite.tex_coords.right].into(),
                                v_offset: [sprite.tex_coords.top, sprite.tex_coords.bottom].into(),
                                depth: pos.z,
                                tint: [r, g, b, a].into(),
                            },
                            &sprite_sheet.texture,
                        ))
                    }?;

                    let (tex_id, this_changed) = textures_ref.insert(
                        factory,
                        world,
                        texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;
                    changed = changed || this_changed;

                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..))
                });
        }

        self.textures.maintain(factory, world);
        changed = changed || self.sprites.changed();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("write");
            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                Some(self.sprites.data()),
            );
        }

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("draw");

        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);

                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_tiles_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = unsafe { TILES_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { TILES_FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(SpriteArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    pso::BlendState::PREMULTIPLIED_ALPHA,
                )])
                .with_depth_test(pso::DepthTest::On {
                    fun: pso::Comparison::Less,
                    write: true,
                }),
        )
        .build(factory, None);

    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}

/// Default bounds that returns the entire tilemap
// TODO: This should at least return Z-perpendicular bounds for ortho
#[derive(Default, Debug)]
pub struct DrawTiles2DBoundsDefault;
impl DrawTiles2DBounds for DrawTiles2DBoundsDefault {
    fn bounds<T: Tile, E: CoordinateEncoder>(map: &TileMap<T, E>, world: &World) -> Region {
        Region::new(Point3::new(0, 0, 0), Point3::from(*map.dimensions()))
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::similar_names
)]

/// A `RenderPlugin` for rendering a 2D Tiles entity.
#[derive(Clone, Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct RenderTiles2D<
    T: Tile,
    E: CoordinateEncoder = MortonEncoder2D,
    Z: DrawTiles2DBounds = DrawTiles2DBoundsDefault,
> {
    target: Target,
    _marker: PhantomData<(T, E, Z)>,
}

impl<T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds> RenderTiles2D<T, E, Z> {
    /// Select render target on which Tiles should be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

type SetupData<'a, T, E> = (
    ReadStorage<'a, Handle<SpriteSheet>>,
    ReadStorage<'a, Handle<Texture>>,
    ReadStorage<'a, Hidden>,
    ReadStorage<'a, TileMap<T, E>>,
);

impl<B: Backend, T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds> RenderPlugin<B>
    for RenderTiles2D<T, E, Z>
{
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), amethyst_error::Error> {
        SetupData::<T, E>::setup(world);

        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _res: &World,
    ) -> Result<(), amethyst_error::Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(
                RenderOrder::BeforeTransparent,
                DrawTiles2DDesc::<T, E, Z>::default().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}
