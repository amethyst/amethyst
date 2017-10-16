//! Forward physically-based drawing pass.

use std::marker::PhantomData;
use std::mem;

use amethyst_assets::AssetStorage;
use cgmath::{Matrix4, One};
use gfx::pso::buffer::ElemStride;
use gfx::traits::Pod;
use rayon::iter::ParallelIterator;
use rayon::iter::internal::UnindexedConsumer;
use specs::{Component, Fetch, Join, ParJoin, ReadStorage};

use cam::Camera;
use color::Rgba;
use error::Result;
use light::{DirectionalLight, Light, PointLight};
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassApply, PassData, Supplier};
use tex::Texture;
use types::Encoder;
use vertex::{Normal, Position, Query, Tangent, TexCoord};

static VERT_SRC: &[u8] = include_bytes!("shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("shaders/fragment/pbm.glsl");

/// Draw mesh with physically based lighting
/// `V` is `VertexFormat`
/// `A` is ambient light resource
/// `T` is transform matrix component
#[derive(Clone, Debug, PartialEq)]
pub struct DrawPbm<V, A, T> {
    _pd: PhantomData<(V, A, T)>,
}

impl<V, A, T> DrawPbm<V, A, T>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    /// Create instance of `DrawPbm` pass
    pub fn new() -> Self {
        DrawPbm { _pd: PhantomData }
    }
}

fn pad(x: [f32; 3]) -> [f32; 4] {
    [x[0], x[1], x[2], 1.0]
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct VertexArgs {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct FragmentArgs {
    point_light_count: i32,
    directional_light_count: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PointLightPod {
    position: [f32; 4],
    color: [f32; 4],
    intensity: f32,
    _pad: [f32; 3],
}

unsafe impl Pod for PointLightPod {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct DirectionalLightPod {
    color: [f32; 4],
    direction: [f32; 4],
}

unsafe impl Pod for DirectionalLightPod {}

impl<'a, V, A, T> PassData<'a> for DrawPbm<V, A, T>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    type Data = (
        Option<Fetch<'a, Camera>>,
        Fetch<'a, A>,
        Fetch<'a, AssetStorage<Mesh>>,
        Fetch<'a, AssetStorage<Texture>>,
        Fetch<'a, MaterialDefaults>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, T>,
        ReadStorage<'a, Light>,
    );
}

impl<'a, V, A, T> PassApply<'a> for DrawPbm<V, A, T>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    type Apply = DrawPbmApply<'a, V, A, T>;
}

impl<V, A, T> Pass for DrawPbm<V, A, T>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    fn compile(&self, effect: NewEffect) -> Result<Effect> {
        effect
            .simple(VERT_SRC, FRAG_SRC)
            .with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0)
            .with_raw_constant_buffer("VertexArgs", mem::size_of::<VertexArgs>(), 1)
            .with_raw_constant_buffer("FragmentArgs", mem::size_of::<FragmentArgs>(), 1)
            .with_raw_constant_buffer("PointLights", mem::size_of::<PointLight>(), 512)
            .with_raw_constant_buffer("DirectionalLight", mem::size_of::<DirectionalLight>(), 16)
            .with_raw_global("ambient_color")
            .with_raw_global("camera_position")
            .with_texture("roughness")
            .with_texture("caveat")
            .with_texture("metallic")
            .with_texture("ambient_occlusion")
            .with_texture("emission")
            .with_texture("normal")
            .with_texture("albedo")
            .with_output("out_color", Some(DepthMode::LessEqualWrite))
            .build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        supplier: Supplier<'a>,
        (camera, ambient, mesh_storage, tex_storage, material_defaults,
            mesh, material, global, light): (
            Option<Fetch<'a, Camera>>,
            Fetch<'a, A>,
            Fetch<'a, AssetStorage<Mesh>>,
            Fetch<'a, AssetStorage<Texture>>,
            Fetch<'a, MaterialDefaults>,
            ReadStorage<'a, MeshHandle>,
            ReadStorage<'a, Material>,
            ReadStorage<'a, T>,
            ReadStorage<'a, Light>,
        ),
) -> DrawPbmApply<'a, V, A, T>{
        DrawPbmApply {
            camera,
            mesh_storage,
            tex_storage,
            material_defaults,
            mesh,
            material,
            global,
            ambient,
            light,
            supplier,
            pd: PhantomData,
        }
    }
}

pub struct DrawPbmApply<'a, V, A: 'static, T: Component> {
    camera: Option<Fetch<'a, Camera>>,
    ambient: Fetch<'a, A>,
    mesh_storage: Fetch<'a, AssetStorage<Mesh>>,
    tex_storage: Fetch<'a, AssetStorage<Texture>>,
    material_defaults: Fetch<'a, MaterialDefaults>,
    mesh: ReadStorage<'a, MeshHandle>,
    material: ReadStorage<'a, Material>,
    global: ReadStorage<'a, T>,
    light: ReadStorage<'a, Light>,
    supplier: Supplier<'a>,
    pd: PhantomData<V>,
}

impl<'a, V, A, T> ParallelIterator for DrawPbmApply<'a, V, A, T>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
{
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawPbmApply {
            camera,
            mesh,
            mesh_storage,
            tex_storage,
            material_defaults,
            material,
            global,
            ambient,
            light,
            supplier,
            ..
        } = self;

        let camera = &camera;
        let ambient = &ambient;
        let light = &light;
        let mesh_storage = &mesh_storage;
        let tex_storage = &tex_storage;
        let material_defaults = &material_defaults;

        supplier
            .supply((&mesh, &material, &global).par_join().map(
                |(mesh, material, global)| {
                    move |encoder: &mut Encoder, effect: &mut Effect| if let Some(mesh) =
                        mesh_storage.get(mesh)
                    {
                        let vbuf = match mesh.buffer(V::QUERIED_ATTRIBUTES) {
                            Some(vbuf) => vbuf.clone(),
                            None => return,
                        };

                        let vertex_args = camera
                            .as_ref()
                            .map(|cam| {
                                VertexArgs {
                                    proj: cam.proj.into(),
                                    view: cam.to_view_matrix().into(),
                                    model: *global.as_ref(),
                                }
                            })
                            .unwrap_or_else(|| {
                                VertexArgs {
                                    proj: Matrix4::one().into(),
                                    view: Matrix4::one().into(),
                                    model: *global.as_ref(),
                                }
                            });

                        effect.update_constant_buffer("VertexArgs", &vertex_args, encoder);

                        let point_lights: Vec<PointLightPod> = light
                            .join()
                            .filter_map(|light| if let Light::Point(ref light) = *light {
                                Some(PointLightPod {
                                    position: pad(light.center.into()),
                                    color: pad(light.color.into()),
                                    intensity: light.intensity,
                                    _pad: [0.0; 3],
                                })
                            } else {
                                None
                            })
                            .collect();

                        let directional_lights: Vec<DirectionalLightPod> = light
                            .join()
                            .filter_map(|light| if let Light::Directional(ref light) = *light {
                                Some(DirectionalLightPod {
                                    color: pad(light.color.into()),
                                    direction: pad(light.direction.into()),
                                })
                            } else {
                                None
                            })
                            .collect();

                        let fragment_args = FragmentArgs {
                            point_light_count: point_lights.len() as i32,
                            directional_light_count: directional_lights.len() as i32,
                        };

                        effect.update_constant_buffer("FragmentArgs", &fragment_args, encoder);
                        effect.update_buffer("PointLights", &point_lights[..], encoder);
                        effect.update_buffer("DirectionalLights", &directional_lights[..], encoder);

                        effect.update_global(
                            "ambient_color",
                            Into::<[f32; 3]>::into(*ambient.as_ref()),
                        );
                        effect.update_global(
                            "camera_position",
                            camera
                                .as_ref()
                                .map(|cam| cam.eye.into())
                                .unwrap_or([0.0; 3]),
                        );

                        let albedo = tex_storage
                            .get(&material.albedo)
                            .or_else(|| tex_storage.get(&material_defaults.0.albedo))
                            .unwrap();

                        let roughness = tex_storage
                            .get(&material.roughness)
                            .or_else(|| tex_storage.get(&material_defaults.0.roughness))
                            .unwrap();

                        let emission = tex_storage
                            .get(&material.emission)
                            .or_else(|| tex_storage.get(&material_defaults.0.emission))
                            .unwrap();

                        let caveat = tex_storage
                            .get(&material.caveat)
                            .or_else(|| tex_storage.get(&material_defaults.0.caveat))
                            .unwrap();

                        let metallic = tex_storage
                            .get(&material.metallic)
                            .or_else(|| tex_storage.get(&material_defaults.0.metallic))
                            .unwrap();

                        let ambient_occlusion = tex_storage
                            .get(&material.ambient_occlusion)
                            .or_else(|| tex_storage.get(&material_defaults.0.ambient_occlusion))
                            .unwrap();

                        let normal = tex_storage
                            .get(&material.normal)
                            .or_else(|| tex_storage.get(&material_defaults.0.normal))
                            .unwrap();

                        effect.data.textures.push(roughness.view().clone());
                        effect.data.samplers.push(roughness.sampler().clone());
                        effect.data.textures.push(caveat.view().clone());
                        effect.data.samplers.push(caveat.sampler().clone());
                        effect.data.textures.push(metallic.view().clone());
                        effect.data.samplers.push(metallic.sampler().clone());
                        effect.data.textures.push(ambient_occlusion.view().clone());
                        effect
                            .data
                            .samplers
                            .push(ambient_occlusion.sampler().clone());
                        effect.data.textures.push(emission.view().clone());
                        effect.data.samplers.push(emission.sampler().clone());
                        effect.data.textures.push(normal.view().clone());
                        effect.data.samplers.push(normal.sampler().clone());
                        effect.data.textures.push(albedo.view().clone());
                        effect.data.samplers.push(albedo.sampler().clone());

                        effect.data.vertex_bufs.push(vbuf);

                        effect.draw(mesh.slice(), encoder);
                    }
                },
            ))
            .drive_unindexed(consumer)
    }
}
