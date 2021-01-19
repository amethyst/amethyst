#![allow(clippy::default_trait_access, clippy::use_self)]
#![allow(unused_imports, unused_variables)]

use std::marker::PhantomData;

use amethyst_assets::{AssetHandle, AssetStorage, Handle};
use amethyst_core::{
    dispatcher::{System, ThreadLocalSystem},
    ecs::{component, world::World, EntityStore, IntoQuery, Resources, TryRead},
    geometry::{Plane, Ray},
    math::{self, clamp, convert, Matrix4, Point2, Point3, Vector2, Vector3, Vector4},
    transform::Transform,
    Hidden,
};
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
    sprite::{SpriteRender, SpriteSheet, Sprites},
    sprite_visibility::SpriteVisibility,
    submodules::{
        gather::CameraGatherer, DynamicUniform, DynamicVertexBuffer, FlatEnvironmentSub, TextureId,
        TextureSub,
    },
    system::GraphAuxData,
    types::{Backend, Texture},
    util,
};
use amethyst_window::ScreenDimensions;
use derivative::Derivative;
use glsl_layout::Uniform;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    iters::Region,
    map::{Map, MapStorage, Tile, TileMap},
    pod::{TileArgs, TileMapArgs},
    CoordinateEncoder, MortonEncoder2D,
};

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
    fn bounds<T: Tile, E: CoordinateEncoder>(
        map: &TileMap<T, E>,
        map_transform: Option<&Transform>,
        aux: &GraphAuxData,
    ) -> Region;
}

/// Default bounds that returns the entire tilemap
#[derive(Default, Debug)]
pub struct DrawTiles2DBoundsDefault;
impl DrawTiles2DBounds for DrawTiles2DBoundsDefault {
    fn bounds<T: Tile, E: CoordinateEncoder>(
        map: &TileMap<T, E>,
        map_transform: Option<&Transform>,
        aux: &GraphAuxData,
    ) -> Region {
        Region::new(Point3::new(0, 0, 0), Point3::from(*map.dimensions()))
    }
}

/// Bounds that use the active camera to cull visible tiles only. If there is no active camera, an empty region is returned.
#[derive(Default, Debug)]
pub struct DrawTiles2DBoundsCameraCulling;

fn camera_ray_to_tile_coords<T: Tile, E: CoordinateEncoder>(
    ray: Ray<f32>,
    tile_plane: &Plane<f32>,
    map: &TileMap<T, E>,
    map_transform: Option<&Transform>,
) -> Point3<i64> {
    // Intersect rays with the tilemap, get intersecting tile coordinates
    let distance = ray.intersect_plane(&tile_plane).unwrap_or(0.0);
    map.to_tile(&ray.at_distance(distance).coords, map_transform)
        .map_or_else(
            |e| {
                // If the point is out of bounds, clamp it to the first/last tile of each dimension
                Point3::new(
                    i64::from(e.point_dimensions.x),
                    i64::from(e.point_dimensions.y),
                    i64::from(e.point_dimensions.z),
                )
            },
            |p| Point3::new(i64::from(p.x), i64::from(p.y), i64::from(p.z)),
        )
}

impl DrawTiles2DBounds for DrawTiles2DBoundsCameraCulling {
    fn bounds<T: Tile, E: CoordinateEncoder>(
        map: &TileMap<T, E>,
        map_transform: Option<&Transform>,
        aux: &GraphAuxData,
    ) -> Region {
        let active_camera = aux.resources.get::<ActiveCamera>();
        if active_camera.is_none() {
            // No active camera
            return Region::empty();
        }
        let active_camera = active_camera.unwrap();
        if active_camera.entity.is_none() {
            // No entity in active camera
            return Region::empty();
        }
        if let Ok(entry) = aux.world.entry_ref(active_camera.entity.unwrap()) {
            let tile_plane = Plane::from_point_normal(
                &map_transform.map_or(Point3::new(0.0, 0.0, 0.0), |t| {
                    Point3::from(*t.translation())
                }),
                &map_transform.map_or(Vector3::new(0.0, 0.0, -1.0), |t| {
                    t.matrix().transform_vector(&Vector3::new(0.0, 0.0, -1.0))
                }),
            );
            let camera_transform = entry.get_component::<Transform>().unwrap();
            let camera = entry.get_component::<Camera>().unwrap();
            let dimensions = aux.resources.get::<ScreenDimensions>().unwrap();
            let w = dimensions.width();
            let h = dimensions.height();
            let diagonal = Vector2::new(w, h);
            // Cast 4 rays from the four corners of the camera, and get at which tile they intersect
            let points = [
                camera_ray_to_tile_coords(
                    camera.screen_ray(Point2::new(0.0, 0.0), diagonal, camera_transform),
                    &tile_plane,
                    map,
                    map_transform,
                ),
                camera_ray_to_tile_coords(
                    camera.screen_ray(Point2::new(0.0, h), diagonal, camera_transform),
                    &tile_plane,
                    map,
                    map_transform,
                ),
                camera_ray_to_tile_coords(
                    camera.screen_ray(Point2::new(w, 0.0), diagonal, camera_transform),
                    &tile_plane,
                    map,
                    map_transform,
                ),
                camera_ray_to_tile_coords(
                    camera.screen_ray(Point2::new(w, h), diagonal, camera_transform),
                    &tile_plane,
                    map,
                    map_transform,
                ),
            ];
            let x = i64::from(map.dimensions().x);
            let y = i64::from(map.dimensions().y);
            // Cull the tilemap using the min and max coordinates along each axis of the tilemap
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            return Region::new(
                Point3::new(
                    points.iter().map(|p| p.x).min().unwrap().max(0).min(x) as u32,
                    points.iter().map(|p| p.y).min().unwrap().max(0).min(y) as u32,
                    0,
                ),
                Point3::new(
                    points.iter().map(|p| p.x + 1).max().unwrap().max(0).min(x) as u32,
                    points.iter().map(|p| p.y + 1).max().unwrap().max(0).min(y) as u32,
                    map.dimensions().z,
                ),
            );
        }
        // Active camera exists but its entity is not found
        Region::empty()
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

impl<B: Backend, T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds>
    RenderGroupDesc<B, GraphAuxData> for DrawTiles2DDesc<T, E, Z>
{
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &GraphAuxData,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, GraphAuxData>>, pso::CreationError> {
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

/// `RenderGroup` providing culling, drawing and transparency functionality for 2D `TileMap` components.
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

impl<B: Backend, T: Tile, E: CoordinateEncoder, Z: DrawTiles2DBounds> RenderGroup<B, GraphAuxData>
    for DrawTiles2D<B, T, E, Z>
{
    #[allow(clippy::cast_precision_loss)]
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData,
    ) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare");

        let mut changed = false;
        let sprite_sheet_storage = aux
            .resources
            .get::<AssetStorage<SpriteSheet>>()
            .expect("getting SpriteSheet AssetStorage");
        let tex_storage = aux
            .resources
            .get::<AssetStorage<Texture>>()
            .expect("getting Texture asset storage");
        let sprites_storage = aux
            .resources
            .get::<AssetStorage<Sprites>>()
            .expect("Could not get Sprites storage.");

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.swap_clear();

        let CameraGatherer { projview, .. } = CameraGatherer::gather(aux.world, aux.resources);

        let mut tilemap_args = vec![];

        let mut query =
            <(&TileMap<T, E>, TryRead<Transform>)>::query().filter(!component::<Hidden>());

        for (tile_map, transform) in query.iter(aux.world) {
            if let Some(sheet) = tile_map
                .sprite_sheet
                .as_ref()
                .and_then(|handle| sprite_sheet_storage.get(handle))
            {
                if let Some(sprites) = sprites_storage.get(&sheet.sprites) {
                    let sprites = sprites.build_sprites();

                    let tilemap_args_index = tilemap_args.len();
                    let map_coordinate_transform: [[f32; 4]; 4] = (*tile_map.transform()).into();
                    let map_transform: [[f32; 4]; 4] = transform.map_or_else(
                        || Matrix4::identity().into(),
                        |transform| (*transform.global_matrix()).into(),
                    );

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

                    compute_region::<T, E, Z>(&tile_map, transform, aux)
                        .iter()
                        .filter_map(|coord| {
                            let tile = tile_map.get(&coord).unwrap();
                            if let Some(sprite_number) = tile.sprite(coord, aux.world) {
                                let batch_data = TileArgs::from_data(
                                    &sprites,
                                    sprite_number,
                                    Some(&TintComponent(tile.tint(coord, aux.world))),
                                    &coord,
                                );

                                let (tex_id, this_changed) = textures_ref.insert(
                                    factory,
                                    aux.resources,
                                    &sheet.texture,
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
            }
        }

        self.textures.maintain(factory, aux.resources);
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
        _aux: &GraphAuxData,
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

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &GraphAuxData) {
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
    map_transform: Option<&Transform>,
    aux: &GraphAuxData,
) -> Region {
    let mut region = Z::bounds(tile_map, map_transform, aux);
    let max_value = tile_map.dimensions();

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
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), pso::CreationError> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let mut shaders = SHADERS.build(factory, Default::default()).map_err(|e| {
        match e {
            hal::device::ShaderError::OutOfMemory(oom) => oom.into(),
            _ => pso::CreationError::Other,
        }
    })?;

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(TileArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(pso::Primitive::TriangleStrip))
                .with_shaders(shaders.raw().map_err(|_| pso::CreationError::Other)?)
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
        _world: &World,
        _resources: &Resources,
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

#[cfg(test)]
mod tests {
    // use amethyst_core::math::Point3;
    // use rayon::prelude::*;
    use amethyst_core::ecs::world::WorldOptions;
    use amethyst_rendy::system::make_graph_aux_data;

    use super::*;
    use crate::FlatEncoder;
    #[derive(Default, Clone, Debug)]
    struct TestTile;
    impl Tile for TestTile {
        fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
            None
        }
    }

    fn bounds_camera_culling_region(
        dim_2d: Vector2<u32>,
        tile_size_2d: Vector2<u32>,
        cam_dim: Vector2<u32>,
        cam_transform: Transform,
        mut map_transform: Transform,
    ) -> Region {
        let map = TileMap::<TestTile, FlatEncoder>::new(
            Vector3::new(dim_2d.x, dim_2d.y, 1),
            Vector3::new(tile_size_2d.x, tile_size_2d.y, 1),
            None,
        );
        let mut world = World::new(WorldOptions::default());
        #[allow(clippy::cast_precision_loss)]
        let camera = world.push((
            cam_transform,
            Camera::standard_2d(cam_dim.x as f32, cam_dim.y as f32),
        ));
        map_transform.copy_local_to_global();
        let map_entity = world.push((map, map_transform));
        let mut resources = Resources::default();
        resources.insert(ActiveCamera {
            entity: Some(camera),
        });
        resources.insert(ScreenDimensions::new(cam_dim.x, cam_dim.y));
        let aux = make_graph_aux_data(&world, &resources);
        let map_entity = world.entry_ref(map_entity).unwrap();
        let map = map_entity
            .get_component::<TileMap<TestTile, FlatEncoder>>()
            .unwrap();
        DrawTiles2DBoundsCameraCulling::bounds(&map, Some(&map_transform), &aux)
    }

    #[test]
    pub fn bounds_camera_culling() {
        // Tilemap occupies exactly 100% of the camera viewport
        let region = bounds_camera_culling_region(
            Vector2::new(10, 10),                         // Number of tiles
            Vector2::new(50, 40),                         // Tile size
            Vector2::new(500, 400),                       // Camera size
            Transform::from(Vector3::new(0.0, 0.0, 2.0)), // Camera transform
            Transform::from(Vector3::new(0.0, 0.0, 0.0)), // Tilemap transform
        );
        assert_eq!(region.min, Point3::new(0, 0, 0));
        assert_eq!(region.max, Point3::new(10, 10, 1));
        assert_eq!(region.volume(), 100);
        // Tilemap is offscreen
        let region = bounds_camera_culling_region(
            Vector2::new(10, 10),                               // Number of tiles
            Vector2::new(50, 40),                               // Tile size
            Vector2::new(500, 400),                             // Camera size
            Transform::from(Vector3::new(0.0, 0.0, 2.0)),       // Camera transform
            Transform::from(Vector3::new(1000.0, -200.0, 0.0)), // Tilemap transform
        );
        assert_eq!(region.min, Point3::new(0, 0, 0));
        assert_eq!(region.max, Point3::new(0, 6, 1));
        assert_eq!(region.volume(), 0);
        // Tilemap is offscreen (opposite side)
        let region = bounds_camera_culling_region(
            Vector2::new(10, 10),                               // Number of tiles
            Vector2::new(50, 40),                               // Tile size
            Vector2::new(500, 400),                             // Camera size
            Transform::from(Vector3::new(0.0, 0.0, 2.0)),       // Camera transform
            Transform::from(Vector3::new(-1000.0, 200.0, 0.0)), // Tilemap transform
        );
        assert_eq!(region.min, Point3::new(10, 4, 0));
        assert_eq!(region.max, Point3::new(10, 10, 1));
        assert_eq!(region.volume(), 0);
        // Tilemap is larger than the camera viewport
        let region = bounds_camera_culling_region(
            Vector2::new(20, 15),                            // Number of tiles
            Vector2::new(50, 40),                            // Tile size
            Vector2::new(505, 405),                          // Camera size
            Transform::from(Vector3::new(50.0, -20.0, 2.0)), // Camera transform
            Transform::from(Vector3::new(50.0, -20.0, 0.0)), // Tilemap transform
        );
        assert_eq!(region.min, Point3::new(3, 1, 0));
        assert_eq!(region.max, Point3::new(15, 13, 1));
        assert_eq!(region.volume(), 144);
        // Tilemap is scaled
        let mut map_transform = Transform::from(Vector3::new(50.0, -20.0, 0.0));
        map_transform.set_scale(Vector3::new(2.0, 0.5, 1.0));
        let region = bounds_camera_culling_region(
            Vector2::new(10, 40),                            // Number of tiles
            Vector2::new(50, 40),                            // Tile size
            Vector2::new(505, 405),                          // Camera size
            Transform::from(Vector3::new(50.0, -20.0, 2.0)), // Camera transform
            map_transform,                                   // Tilemap transform
        );
        assert_eq!(region.min, Point3::new(1, 8, 0));
        assert_eq!(region.max, Point3::new(8, 30, 1));
        assert_eq!(region.volume(), 154);
        // Tilemap is rotated
        let mut map_transform = Transform::from(Vector3::new(50.0, -20.0, 0.0));
        map_transform.append_rotation_z_axis(std::f32::consts::PI / 4.0);
        let region = bounds_camera_culling_region(
            Vector2::new(10, 40),                            // Number of tiles
            Vector2::new(100, 25),                           // Tile size
            Vector2::new(500, 250),                          // Camera size
            Transform::from(Vector3::new(50.0, -20.0, 2.0)), // Camera transform
            map_transform,                                   // Tilemap transform
        );
        assert_eq!(region.min, Point3::new(1, 10, 0));
        assert_eq!(region.max, Point3::new(8, 32, 1));
        assert_eq!(region.volume(), 154);
    }
}
