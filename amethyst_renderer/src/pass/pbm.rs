//! Forward physically-based drawing pass.

use std::marker::PhantomData;
use std::mem;

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
use mesh::Mesh;
use mtl::Material;
use pipe::{DepthMode, Effect, NewEffect};
use pipe::pass::{Pass, PassApply, PassData, Supplier};
use types::Encoder;
use vertex::{Normal, Position, Query, Tangent, TexCoord};

static VERT_SRC: &[u8] = include_bytes!("shaders/vertex/basic.glsl");
static FRAG_SRC: &[u8] = include_bytes!("shaders/fragment/pbm.glsl");

/// Draw mesh with physically based lighting
/// `V` is `VertexFormat`
/// `A` is ambient light resource
/// `M` is `Mesh` component
/// `N` is `Material` component
/// `T` is transform matrix component
/// `L` is `Light` component
#[derive(Clone, Debug, PartialEq)]
pub struct DrawPbm<V, A, M, N, T, L> {
    _pd: PhantomData<(V, A, M, N, T, L)>,
}

impl<V, A, M, N, T, L> DrawPbm<V, A, M, N, T, L>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
    L: Component + AsRef<Light> + Send + Sync,
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

impl<'a, V, A, M, N, T, L> PassData<'a> for DrawPbm<V, A, M, N, T, L>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
    L: Component + AsRef<Light> + Send + Sync,
{
    type Data = (
        Option<Fetch<'a, Camera>>,
        Fetch<'a, A>,
        ReadStorage<'a, M>,
        ReadStorage<'a, N>,
        ReadStorage<'a, T>,
        ReadStorage<'a, L>,
    );
}

impl<'a, V, A, M, N, T, L> PassApply<'a> for DrawPbm<V, A, M, N, T, L>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
    L: Component + AsRef<Light> + Send + Sync,
{
    type Apply = DrawPbmApply<'a, V, A, M, N, T, L>;
}

impl<V, A, M, N, T, L> Pass for DrawPbm<V, A, M, N, T, L>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
    L: Component + AsRef<Light> + Send + Sync,
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
        (camera, ambient, mesh, material, global, light): (
            Option<Fetch<'a, Camera>>,
            Fetch<'a, A>,
            ReadStorage<'a, M>,
            ReadStorage<'a, N>,
            ReadStorage<'a, T>,
            ReadStorage<'a, L>,
        ),
    ) -> DrawPbmApply<'a, V, A, M, N, T, L> {
        DrawPbmApply {
            camera: camera,
            mesh: mesh,
            material: material,
            global: global,
            ambient: ambient,
            light: light,
            supplier: supplier,
            pd: PhantomData,
        }
    }
}

pub struct DrawPbmApply<'a, V, A: 'static, M: Component, N: Component, T: Component, L: Component> {
    camera: Option<Fetch<'a, Camera>>,
    ambient: Fetch<'a, A>,
    mesh: ReadStorage<'a, M>,
    material: ReadStorage<'a, N>,
    global: ReadStorage<'a, T>,
    light: ReadStorage<'a, L>,
    supplier: Supplier<'a>,
    pd: PhantomData<V>,
}

impl<'a, V, A, M, N, T, L> ParallelIterator for DrawPbmApply<'a, V, A, M, N, T, L>
where
    V: Query<(Position, Normal, Tangent, TexCoord)>,
    A: AsRef<Rgba> + Send + Sync + 'static,
    T: Component + AsRef<[[f32; 4]; 4]> + Send + Sync,
    M: Component + AsRef<Mesh> + Send + Sync,
    N: Component + AsRef<Material> + Send + Sync,
    L: Component + AsRef<Light> + Send + Sync,
{
    type Item = ();

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let DrawPbmApply {
            camera,
            mesh,
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

        supplier
            .supply((&mesh, &material, &global).par_join().map(
                |(mesh, material, global)| {
                    move |encoder: &mut Encoder, effect: &mut Effect| {
                        let mesh = mesh.as_ref();

                        let vbuf = match mesh.buffer(V::QUERIED_ATTRIBUTES) {
                            Some(vbuf) => vbuf.clone(),
                            None => return,
                        };

                        let material = material.as_ref();

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
                            .filter_map(|light| if let Light::Point(ref light) = *light.as_ref() {
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
                            .filter_map(|light| {
                                if let Light::Directional(ref light) = *light.as_ref() {
                                    Some(DirectionalLightPod {
                                        color: pad(light.color.into()),
                                        direction: pad(light.direction.into()),
                                    })
                                } else {
                                    None
                                }
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

                        effect.data.textures.push(material.roughness.view().clone());
                        effect
                            .data
                            .samplers
                            .push(material.roughness.sampler().clone());
                        effect.data.textures.push(material.caveat.view().clone());
                        effect.data.samplers.push(material.caveat.sampler().clone());
                        effect.data.textures.push(material.metallic.view().clone());
                        effect
                            .data
                            .samplers
                            .push(material.metallic.sampler().clone());
                        effect
                            .data
                            .textures
                            .push(material.ambient_occlusion.view().clone());
                        effect
                            .data
                            .samplers
                            .push(material.ambient_occlusion.sampler().clone());
                        effect.data.textures.push(material.emission.view().clone());
                        effect
                            .data
                            .samplers
                            .push(material.emission.sampler().clone());
                        effect.data.textures.push(material.normal.view().clone());
                        effect.data.samplers.push(material.normal.sampler().clone());
                        effect.data.textures.push(material.albedo.view().clone());
                        effect.data.samplers.push(material.albedo.sampler().clone());

                        effect.data.vertex_bufs.push(vbuf);

                        effect.draw(mesh.slice(), encoder);
                    }
                },
            ))
            .drive_unindexed(consumer)
    }
}
