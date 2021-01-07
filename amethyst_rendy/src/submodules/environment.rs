//! Environment submodule for shared environmental descriptor set data.
//! Fetches and sets projection and lighting descriptor set information.
use amethyst_core::{
    ecs::*,
    math::{convert, Vector3},
    transform::Transform,
};
use glsl_layout::*;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    light::Light,
    pod::{self, IntoPod},
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{
            self,
            adapter::PhysicalDevice,
            device::Device,
            pso::{CreationError, Descriptor},
        },
        memory::Write as _,
        resource::{Buffer, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    },
    submodules::gather::{AmbientGatherer, CameraGatherer},
    types::Backend,
    util::{self, TapCountIter},
};

const MAX_POINT_LIGHTS: usize = 128;
const MAX_DIR_LIGHTS: usize = 16;
const MAX_SPOT_LIGHTS: usize = 128;

/// Submodule for loading and binding descriptor sets for a 3D, lit environment.
/// This also abstracts away the need for handling multiple images in flight, as it provides
/// per-image submissions.
#[derive(Debug)]
pub struct EnvironmentSub<B: Backend> {
    layout: RendyHandle<DescriptorSetLayout<B>>,
    per_image: Vec<PerImageEnvironmentSub<B>>,
}

/// Submodule for loading and binding descriptor sets for a 3D, lit environment.
/// This is the actual implementation for a given environment, but multiple instances may exist
/// for each image in flight.
#[derive(Debug)]
struct PerImageEnvironmentSub<B: Backend> {
    buffer: Option<Escape<Buffer<B>>>,
    set: Escape<DescriptorSet<B>>,
}

impl<B: Backend> EnvironmentSub<B> {
    /// Create and allocate a new `EnvironmentSub` with the provided rendy `Factory`
    /// Allocate to the supplied shader.
    pub fn new(
        factory: &Factory<B>,
        flags: [hal::pso::ShaderStageFlags; 2],
    ) -> Result<Self, CreationError> {
        use rendy::hal::pso::*;

        let layout = factory
            .create_descriptor_set_layout(util::set_layout_bindings(vec![
                (
                    1,
                    DescriptorType::Buffer {
                        ty: BufferDescriptorType::Uniform,
                        format: BufferDescriptorFormat::Structured {
                            dynamic_offset: false,
                        },
                    },
                    flags[0],
                ),
                (
                    4,
                    DescriptorType::Buffer {
                        ty: BufferDescriptorType::Uniform,
                        format: BufferDescriptorFormat::Structured {
                            dynamic_offset: false,
                        },
                    },
                    flags[1],
                ),
            ]))?
            .into();

        Ok(Self {
            layout,
            per_image: Vec::new(),
        })
    }

    /// Returns the raw `DescriptorSetLayout` for this environment
    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    /// Performs any re-allocation and GPU memory writing required for this environment set.
    pub fn process(
        &mut self,
        factory: &Factory<B>,
        index: usize,
        world: &World,
        resources: &Resources,
    ) -> bool {
        #[cfg(feature = "profiler")]
        profile_scope!("process");

        let this_image = {
            while self.per_image.len() <= index {
                self.per_image
                    .push(PerImageEnvironmentSub::new(factory, &self.layout));
            }
            &mut self.per_image[index]
        };
        this_image.process(factory, world, resources)
    }

    /// Binds this environment set for all images.
    #[inline]
    pub fn bind(
        &self,
        index: usize,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        self.per_image[index].bind(pipeline_layout, set_id, encoder);
    }
}

impl<B: Backend> PerImageEnvironmentSub<B> {
    fn new(factory: &Factory<B>, layout: &RendyHandle<DescriptorSetLayout<B>>) -> Self {
        Self {
            buffer: None,
            set: factory.create_descriptor_set(layout.clone()).unwrap(),
        }
    }

    #[inline]
    fn bind(
        &self,
        pipeline_layout: &B::PipelineLayout,
        set_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                pipeline_layout,
                set_id,
                Some(self.set.raw()),
                std::iter::empty(),
            );
        }
    }

    fn process(&mut self, factory: &Factory<B>, world: &World, resources: &Resources) -> bool {
        let align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;

        let projview_size = util::align_size::<pod::ViewArgs>(align, 1);
        let env_buf_size = util::align_size::<pod::Environment>(align, 1);
        let plight_buf_size = util::align_size::<pod::PointLight>(align, MAX_POINT_LIGHTS);
        let dlight_buf_size = util::align_size::<pod::DirectionalLight>(align, MAX_DIR_LIGHTS);
        let slight_buf_size = util::align_size::<pod::SpotLight>(align, MAX_SPOT_LIGHTS);

        let projview_range = 0..projview_size;
        let env_range = util::next_range(&projview_range, env_buf_size);
        let plight_range = util::next_range(&env_range, plight_buf_size);
        let dlight_range = util::next_range(&plight_range, dlight_buf_size);
        let slight_range = util::next_range(&dlight_range, slight_buf_size);

        let whole_range = 0..slight_range.end;

        let new_buffer = util::ensure_buffer(
            &factory,
            &mut self.buffer,
            hal::buffer::Usage::UNIFORM,
            rendy::memory::Dynamic,
            whole_range.end,
        )
        .unwrap();
        if let Some(buffer) = self.buffer.as_mut() {
            if new_buffer {
                use util::{desc_write, sub_range};
                let buffer = buffer.raw();
                let env_set = self.set.raw();

                let desc_projview = Descriptor::Buffer(buffer, sub_range(projview_range.clone()));
                let desc_env = Descriptor::Buffer(buffer, sub_range(env_range.clone()));
                let desc_plight = Descriptor::Buffer(buffer, sub_range(plight_range.clone()));
                let desc_dlight = Descriptor::Buffer(buffer, sub_range(dlight_range.clone()));
                let desc_slight = Descriptor::Buffer(buffer, sub_range(slight_range.clone()));

                unsafe {
                    factory.write_descriptor_sets(vec![
                        desc_write(env_set, 0, desc_projview),
                        desc_write(env_set, 1, desc_env),
                        desc_write(env_set, 2, desc_plight),
                        desc_write(env_set, 3, desc_dlight),
                        desc_write(env_set, 4, desc_slight),
                    ]);
                }
            }

            let CameraGatherer {
                camera_position,
                projview,
            } = CameraGatherer::gather(world, resources);

            let mut mapped = buffer.map(factory, whole_range.clone()).unwrap();
            let mut writer = unsafe { mapped.write::<u8>(factory, whole_range).unwrap() };
            let dst_slice = unsafe { writer.slice() };

            let mut env = pod::Environment {
                ambient_color: AmbientGatherer::gather(resources),
                camera_position,
                point_light_count: 0,
                directional_light_count: 0,
                spot_light_count: 0,
            }
            .std140();

            let mut point_lights_query = <(Read<Light>, Read<Transform>)>::query();
            let point_lights = point_lights_query
                .iter(world)
                .filter_map(|(light, transform)| {
                    match &*light {
                        Light::Point(light) => {
                            Some(
                                pod::PointLight {
                                    position: convert::<_, Vector3<f32>>(
                                        transform.global_matrix().column(3).xyz(),
                                    )
                                    .into_pod(),
                                    color: light.color.into_pod(),
                                    intensity: light.intensity,
                                }
                                .std140(),
                            )
                        }
                        _ => None,
                    }
                })
                .take(MAX_POINT_LIGHTS);

            let mut dir_lights_query = <Read<Light>>::query();
            let dir_lights = dir_lights_query
                .iter(world)
                .filter_map(|light| {
                    match &*light {
                        Light::Directional(light) => {
                            Some(
                                pod::DirectionalLight {
                                    color: light.color.into_pod(),
                                    intensity: light.intensity,
                                    direction: light.direction.into_pod(),
                                }
                                .std140(),
                            )
                        }
                        _ => None,
                    }
                })
                .take(MAX_DIR_LIGHTS);

            let mut spot_lights_query = <(Read<Light>, Read<Transform>)>::query();
            let spot_lights = spot_lights_query
                .iter(world)
                .filter_map(|(light, transform)| {
                    match &*light {
                        Light::Spot(light) => {
                            Some(
                                pod::SpotLight {
                                    position: convert::<_, Vector3<f32>>(
                                        transform.global_matrix().column(3).xyz(),
                                    )
                                    .into_pod(),
                                    color: light.color.into_pod(),
                                    direction: light.direction.into_pod(),
                                    angle: light.angle.cos(),
                                    intensity: light.intensity,
                                    range: light.range,
                                    smoothness: light.smoothness,
                                }
                                .std140(),
                            )
                        }
                        _ => None,
                    }
                })
                .take(MAX_SPOT_LIGHTS);

            use util::{usize_range, write_into_slice};
            write_into_slice(
                &mut dst_slice[usize_range(plight_range)],
                point_lights.tap_count(&mut env.point_light_count),
            );
            write_into_slice(
                &mut dst_slice[usize_range(dlight_range)],
                dir_lights.tap_count(&mut env.directional_light_count),
            );
            write_into_slice(
                &mut dst_slice[usize_range(slight_range)],
                spot_lights.tap_count(&mut env.spot_light_count),
            );
            write_into_slice(&mut dst_slice[usize_range(projview_range)], Some(projview));
            write_into_slice(&mut dst_slice[usize_range(env_range)], Some(env));
        }

        new_buffer
    }
}
