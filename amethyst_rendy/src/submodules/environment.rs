use crate::{
    light::LightComponent,
    pod::{self, IntoPod},
    rendy::{
        command::RenderPassEncoder,
        factory::Factory,
        hal::{self, adapter::PhysicalDevice, device::Device, pso::Descriptor},
        memory::Write as _,
        resource::{Buffer, DescriptorSet, DescriptorSetLayout, Escape, Handle as RendyHandle},
    },
    submodules::gather::{AmbientGatherer, CameraGatherer},
    types::Backend,
    util::{self, TapCountIter},
};
use amethyst_core::{
    ecs::{Join, ReadStorage, Resources, SystemData},
    math::{convert, Vector3},
    transform::TransformComponent,
};
use glsl_layout::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

const MAX_POINT_LIGHTS: usize = 128;
const MAX_DIR_LIGHTS: usize = 16;
const MAX_SPOT_LIGHTS: usize = 128;

#[derive(Debug)]
pub struct EnvironmentSub<B: Backend> {
    layout: RendyHandle<DescriptorSetLayout<B>>,
    per_image: Vec<PerImageEnvironmentSub<B>>,
}

#[derive(Debug)]
struct PerImageEnvironmentSub<B: Backend> {
    buffer: Option<Escape<Buffer<B>>>,
    set: Escape<DescriptorSet<B>>,
}

impl<B: Backend> EnvironmentSub<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, failure::Error> {
        Ok(Self {
            layout: set_layout! {factory, [1] UniformBuffer VERTEX, [4] UniformBuffer FRAGMENT},
            per_image: Vec::new(),
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    pub fn process(&mut self, factory: &Factory<B>, index: usize, res: &Resources) -> bool {
        #[cfg(feature = "profiler")]
        profile_scope!("process");

        let this_image = {
            while self.per_image.len() <= index {
                self.per_image
                    .push(PerImageEnvironmentSub::new(factory, &self.layout));
            }
            &mut self.per_image[index]
        };
        this_image.process(factory, res)
    }

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
        encoder.bind_graphics_descriptor_sets(
            pipeline_layout,
            set_id,
            Some(self.set.raw()),
            std::iter::empty(),
        );
    }

    fn process(&mut self, factory: &Factory<B>, res: &Resources) -> bool {
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
                use util::{desc_write, opt_range};
                let buffer = buffer.raw();
                let env_set = self.set.raw();

                let desc_projview = Descriptor::Buffer(buffer, opt_range(projview_range.clone()));
                let desc_env = Descriptor::Buffer(buffer, opt_range(env_range.clone()));
                let desc_plight = Descriptor::Buffer(buffer, opt_range(plight_range.clone()));
                let desc_dlight = Descriptor::Buffer(buffer, opt_range(dlight_range.clone()));
                let desc_slight = Descriptor::Buffer(buffer, opt_range(slight_range.clone()));

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
            } = CameraGatherer::gather(res);

            let mut mapped = buffer.map(factory, whole_range.clone()).unwrap();
            let mut writer = unsafe { mapped.write::<u8>(factory, whole_range.clone()).unwrap() };
            let dst_slice = unsafe { writer.slice() };

            let mut env = pod::Environment {
                ambient_color: AmbientGatherer::gather(res),
                camera_position,
                point_light_count: 0,
                directional_light_count: 0,
                spot_light_count: 0,
            }
            .std140();

            let (lights, transforms) = <(
                ReadStorage<'_, LightComponent>,
                ReadStorage<'_, TransformComponent>,
            )>::fetch(res);

            let point_lights = (&lights, &transforms)
                .join()
                .filter_map(|(light, transform)| match light {
                    LightComponent::Point(light) => Some(
                        pod::PointLight {
                            position: convert::<_, Vector3<f32>>(
                                transform.global_matrix().column(3).xyz(),
                            )
                            .into_pod(),
                            color: light.color.into_pod(),
                            intensity: light.intensity,
                        }
                        .std140(),
                    ),
                    _ => None,
                })
                .take(MAX_POINT_LIGHTS);

            let dir_lights = lights
                .join()
                .filter_map(|light| match light {
                    LightComponent::Directional(ref light) => Some(
                        pod::DirectionalLight {
                            color: light.color.into_pod(),
                            intensity: light.intensity,
                            direction: light.direction.into_pod(),
                        }
                        .std140(),
                    ),
                    _ => None,
                })
                .take(MAX_DIR_LIGHTS);

            let spot_lights = (&lights, &transforms)
                .join()
                .filter_map(|(light, transform)| {
                    if let LightComponent::Spot(ref light) = *light {
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
                    } else {
                        None
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

        return new_buffer;
    }
}
