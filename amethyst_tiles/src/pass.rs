#![allow(clippy::default_trait_access, clippy::use_self)]
#![allow(unused_imports, unused_variables)]

use amethyst_core::{
    dispatcher::*,
    ecs::prelude::*,
    geometry::{Plane, Ray},
    math::{self, clamp, convert, Matrix4, Point2, Point3, Vector2, Vector3, Vector4},
    transform::Transform,
    Hidden,
};

use amethyst_assets::{AssetStorage, Handle};
use amethyst_rendy::{
    batch::{GroupIterator, OneLevelBatch, OrderedTwoLevelBatch},
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    camera::{ActiveCamera, Camera},
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    pod::IntoPod,
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
    resources::Tint as TintComponent,
    sprite::{SpriteRender, SpriteSheet},
    sprite_visibility::SpriteVisibility,
    submodules::{
        gather::CameraGatherer, DynamicUniform, DynamicVertexBuffer, FlatEnvironmentSub, TextureId,
        TextureSub,
    },
    types::{Backend, Texture},
    util,
};
use amethyst_window::ScreenDimensions;
use derivative::Derivative;
use glsl_layout::AsStd140;
use std::marker::PhantomData;

use crate::{
    iters::Region,
    map::{Map, MapStorage, Tile, TileMap},
    pod::{TileArgs, TileMapArgs},
    CoordinateEncoder, MortonEncoder2D,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/tiles.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/tiles.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();

    static ref SHADERS: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();

}

/// Trait to describe how rendering tiles may be culled for the tilemap to render
pub trait DrawTiles2DBounds: 'static + std::fmt::Debug + Send + Sync {
    /// Returns the region to render the tiles
    fn bounds<T: Tile, E: CoordinateEncoder>(map: &TileMap<T, E>, world: &World) -> Region;
}

/// Default bounds that returns the entire tilemap
#[derive(Default, Debug)]
pub struct DrawTiles2DBoundsDefault;
impl DrawTiles2DBounds for DrawTiles2DBoundsDefault {
    fn bounds<T: Tile, E: CoordinateEncoder>(map: &TileMap<T, E>, world: &World) -> Region {
        Region::new(
            Point3::new(0, 0, 0),
            Point3::from(*map.dimensions() - Vector3::new(1, 1, 1)),
        )
    }
}

/// Draw opaque tilemap without lighting.
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

        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;

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
            textures,
            vertex,
            env: vec![env],
            sprites: Default::default(),
            _marker: PhantomData::default(),
            change: Default::default(),
        }))
    }
}

/// `RenderGroup` providing culling, drawing and transparency functionality for 3D `TileMap` components.
///
/// Notes on use:
/// - Due to the use of transparency and Z-order, the `TileMap` entity must be viewed from a Z-up perspective
/// for  transparency to occur correctly. If viewed from "underneath", transparency ordering issues will occur.
///
/// In shorter terms, this means that the camera must "Look Down" at the tiles.
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
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, TileArgs>,
    sprites: OrderedTwoLevelBatch<TextureId, usize, TileArgs>,
    change: util::ChangeDetection,

    env: Vec<DynamicUniform<B, TileMapArgs>>,

    #[derivative(Debug = "ignore")]
    _marker: PhantomData<(T, E, Z)>,
}

impl<B: Backend, T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds> RenderGroup<B, World>
    for DrawTiles2D<B, T, E, Z>
{
    #[allow(clippy::cast_precision_loss)]
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
        let (sprite_sheet_storage, tex_storage, hiddens, tile_maps, transforms) =
            <(
                Read<'_, AssetStorage<SpriteSheet>>,
                Read<'_, AssetStorage<Texture>>,
                ReadStorage<'_, Hidden>,
                ReadStorage<'_, TileMap<T, E>>,
                ReadStorage<'_, Transform>,
            )>::fetch(world);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.swap_clear();

        let CameraGatherer { projview, .. } = CameraGatherer::gather(world);

        let mut tilemap_args = vec![];

        for (tile_map, _, transform) in (&tile_maps, !&hiddens, transforms.maybe()).join() {
            let maybe_sheet = tile_map
                .sprite_sheet
                .as_ref()
                .and_then(|handle| sprite_sheet_storage.get(handle))
                .filter(|sheet| tex_storage.contains(&sheet.texture));

            let sprite_sheet = match maybe_sheet {
                Some(sheet) => sheet,
                None => continue,
            };

            let tilemap_args_index = tilemap_args.len();
            let map_coordinate_transform: [[f32; 4]; 4] = (*tile_map.transform()).into();
            let map_transform: [[f32; 4]; 4] = if let Some(transform) = transform {
                (*transform.global_matrix()).into()
            } else {
                Matrix4::identity().into()
            };
            tilemap_args.push(TileMapArgs {
                proj: projview.proj,
                view: projview.view,
                map_coordinate_transform: map_coordinate_transform.into(),
                map_transform: map_transform.into(),
                sprite_dimensions: [
                    tile_map.tile_dimensions().x as f32,
                    tile_map.tile_dimensions().y as f32,
                ]
                .into(),
            });

            compute_region::<T, E, Z>(&tile_map, &world)
                .iter()
                .filter_map(|coord| {
                    let tile = tile_map.get(&coord).unwrap();
                    if let Some(sprite_number) = tile.sprite(coord, world) {
                        let (batch_data, texture) = TileArgs::from_data(
                            &tex_storage,
                            &sprite_sheet,
                            sprite_number,
                            Some(&TintComponent(tile.tint(coord, world))),
                            &coord,
                        )?;

                        let (tex_id, this_changed) = textures_ref.insert(
                            factory,
                            world,
                            texture,
                            hal::image::Layout::ShaderReadOnlyOptimal,
                        )?;
                        changed = changed || this_changed;

                        return Some((tex_id, batch_data));
                    }
                    None
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, tilemap_args_index, batch_data.drain(..))
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

            // grow tilemap_args cache if necessary, or shrink it
            if self.env.len() < tilemap_args.len() || self.env.len() <= tilemap_args.len() / 2 {
                self.env.resize_with(tilemap_args.len(), || {
                    DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX).unwrap()
                });
            }

            for (env, tilemap_args) in self.env.iter_mut().zip(&tilemap_args) {
                env.write(factory, index, tilemap_args.std140());
            }
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

        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, ranges) in self.sprites.iter() {
            for (tilemap_args_index, range) in ranges {
                let env = self.env.get(*tilemap_args_index).unwrap();
                env.bind(index, layout, 0, &mut encoder);

                if self.textures.loaded(tex) {
                    self.textures.bind(layout, 1, tex, &mut encoder);

                    unsafe {
                        encoder.draw(0..4, range.to_owned());
                    }
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

fn compute_region<T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds>(
    tile_map: &TileMap<T, E>,
    world: &World,
) -> Region {
    let mut region = Z::bounds(tile_map, world);
    let max_value = tile_map.dimensions() - Vector3::new(1, 1, 1);

    region.min = Point3::new(
        region.min.x.max(0).min(max_value.x),
        region.min.y.max(0).min(max_value.y),
        region.min.z.max(0).min(max_value.z),
    );
    region.max = Point3::new(
        region.max.x.max(0).min(max_value.x),
        region.max.y.max(0).min(max_value.y),
        region.max.z.max(0).min(max_value.z),
    );

    region
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

    let mut shaders = SHADERS.build(factory, Default::default())?;

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(TileArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
                .with_shaders(shaders.raw()?)
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: Some(pso::BlendState::PREMULTIPLIED_ALPHA),
                }])
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::Greater,
                    write: false,
                }),
        )
        .build(factory, None);

    shaders.dispose(factory);

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
    #[must_use]
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B: Backend, T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds> RenderPlugin<B>
    for RenderTiles2D<T, E, Z>
{
    //fn on_build<'a, 'b>(
    //    &mut self,
    //    world: &mut World,
    //    builder: &mut DispatcherBuilder<'a, 'b>,
    //) -> Result<(), amethyst_error::Error> {
    //    Ok(())
    //}

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
