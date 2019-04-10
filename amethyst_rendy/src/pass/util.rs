#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    camera::{ActiveCamera, Camera},
    light::Light,
    pod::{self, IntoPod},
};
use amethyst_core::{
    ecs::{Join, Read, ReadStorage},
    GlobalTransform,
};
use core::ops::{Add, Range};
use glsl_layout::*;
use rendy::{
    factory::Factory,
    hal::{buffer::Usage, format, pso, Backend},
    memory::MemoryUsage,
    resource::{BufferInfo, Escape},
};

pub(crate) fn prepare_camera(
    active_camera: &Option<Read<'_, ActiveCamera>>,
    cameras: &ReadStorage<'_, Camera>,
    global_transforms: &ReadStorage<'_, GlobalTransform>,
) -> (vec3, <pod::ViewArgs as AsStd140>::Std140) {
    let defcam = Camera::standard_2d();
    let identity = GlobalTransform::default();

    let camera = active_camera
        .as_ref()
        .and_then(|ac| {
            cameras.get(ac.entity).map(|camera| {
                (
                    camera,
                    global_transforms.get(ac.entity).unwrap_or(&identity),
                )
            })
        })
        .unwrap_or_else(|| {
            (cameras, global_transforms)
                .join()
                .next()
                .unwrap_or((&defcam, &identity))
        });

    let camera_position = (camera.1).0.column(3).xyz().into_pod();

    let proj: [[f32; 4]; 4] = camera.0.proj.into();
    let view: [[f32; 4]; 4] = (*camera.1)
        .0
        .try_inverse()
        .expect("Unable to get inverse of camera transform")
        .into();

    let viewargs = pod::ViewArgs {
        proj: proj.into(),
        view: view.into(),
    }
    .std140();

    (camera_position, viewargs)
}

pub(crate) fn collect_lights(
    lights: &ReadStorage<'_, Light>,
    global_transforms: &ReadStorage<'_, GlobalTransform>,
    max_point_lights: usize,
    max_dir_lights: usize,
    max_spot_lights: usize,
) -> (
    Vec<<pod::PointLight as AsStd140>::Std140>,
    Vec<<pod::DirectionalLight as AsStd140>::Std140>,
    Vec<<pod::SpotLight as AsStd140>::Std140>,
) {
    let point_lights: Vec<_> = (lights, global_transforms)
        .join()
        .filter_map(|(light, transform)| {
            if let Light::Point(ref light) = *light {
                Some(
                    pod::PointLight {
                        position: transform.0.column(3).xyz().into_pod(),
                        color: light.color.into_pod(),
                        intensity: light.intensity,
                    }
                    .std140(),
                )
            } else {
                None
            }
        })
        .take(max_point_lights)
        .collect();

    let dir_lights: Vec<_> = lights
        .join()
        .filter_map(|light| {
            if let Light::Directional(ref light) = *light {
                Some(
                    pod::DirectionalLight {
                        color: light.color.into_pod(),
                        direction: light.direction.into_pod(),
                    }
                    .std140(),
                )
            } else {
                None
            }
        })
        .take(max_dir_lights)
        .collect();

    let spot_lights: Vec<_> = (lights, global_transforms)
        .join()
        .filter_map(|(light, transform)| {
            if let Light::Spot(ref light) = *light {
                Some(
                    pod::SpotLight {
                        position: transform.0.column(3).xyz().into_pod(),
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
        .take(max_spot_lights)
        .collect();
    (point_lights, dir_lights, spot_lights)
}

pub fn next_range_opt<T: Add<Output = T> + Clone>(
    prev: &Range<Option<T>>,
    length: T,
) -> Range<Option<T>> {
    prev.end.clone()..prev.end.clone().map(|x| x + length)
}

pub fn ensure_buffer<B: Backend>(
    factory: &Factory<B>,
    buffer: &mut Option<Escape<rendy::resource::Buffer<B>>>,
    usage: Usage,
    memory_usage: impl MemoryUsage,
    min_size: u64,
) -> Result<bool, failure::Error> {
    if buffer.as_ref().map(|b| b.size()).unwrap_or(0) < min_size {
        let new_size = min_size.next_power_of_two();
        let new_buffer = factory.create_buffer(
            BufferInfo {
                size: new_size,
                usage,
            },
            memory_usage,
        )?;
        *buffer = Some(new_buffer);
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn align_size<T: AsStd140>(align: u64, array_len: usize) -> u64
where
    T::Std140: Sized,
{
    let size = (std::mem::size_of::<T::Std140>() * array_len) as u64;
    ((size + align - 1) / align) * align
}

pub fn simple_shader_set<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    simple_shader_set_ext(vertex, fragment, None, None, None)
}

pub fn simple_shader_set_ext<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
    hull: Option<&'a B::ShaderModule>,
    domain: Option<&'a B::ShaderModule>,
    geometry: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    fn map_entry_point<'a, B: Backend>(module: &'a B::ShaderModule) -> pso::EntryPoint<'a, B> {
        pso::EntryPoint {
            entry: "main",
            module,
            specialization: pso::Specialization::default(),
        }
    }

    pso::GraphicsShaderSet {
        vertex: map_entry_point(vertex),
        fragment: fragment.map(map_entry_point),
        hull: hull.map(map_entry_point),
        domain: domain.map(map_entry_point),
        geometry: geometry.map(map_entry_point),
    }
}

pub fn push_vertex_desc<'a>(
    (elements, stride, rate): (
        impl IntoIterator<Item = pso::Element<format::Format>>,
        pso::ElemStride,
        pso::InstanceRate,
    ),
    vertex_buffers: &mut Vec<pso::VertexBufferDesc>,
    attributes: &mut Vec<pso::AttributeDesc>,
) {
    let index = vertex_buffers.len() as pso::BufferIndex;
    vertex_buffers.push(pso::VertexBufferDesc {
        binding: index,
        stride,
        rate,
    });

    let mut location = attributes.last().map_or(0, |a| a.location + 1);
    for element in elements.into_iter() {
        attributes.push(pso::AttributeDesc {
            location,
            binding: index,
            element,
        });
        location += 1;
    }
}
