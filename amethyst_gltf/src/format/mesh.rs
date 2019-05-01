use std::ops::Range;

use amethyst_error::Error;
use amethyst_renderer::{AnimatedComboMeshCreator, Attribute, MeshData, Separate};
use log::trace;

use super::Buffers;
use crate::{error, GltfSceneOptions};

pub fn load_mesh(
    mesh: &gltf::Mesh<'_>,
    buffers: &Buffers,
    options: &GltfSceneOptions,
) -> Result<Vec<(MeshData, Option<usize>, Range<[f32; 3]>)>, Error> {
    trace!("Loading mesh");
    let mut primitives = vec![];

    for primitive in mesh.primitives() {
        trace!("Loading mesh primitive");
        let reader = primitive.reader(|buffer| buffers.buffer(&buffer));

        trace!("Loading faces");
        let faces = reader
            .read_indices()
            .map(|indices| indices.into_u32())
            .map(|mut indices| {
                let mut faces = vec![];
                while let (Some(a), Some(b), Some(c)) =
                    (indices.next(), indices.next(), indices.next())
                {
                    faces.push(a as usize);
                    faces.push(b as usize);
                    faces.push(c as usize);
                }
                faces
            });

        trace!("Loading positions");
        let positions = reader
            .read_positions()
            .map(|positions| match faces {
                Some(ref faces) => {
                    let vertices = positions.collect::<Vec<_>>();
                    faces.iter().map(|i| vertices[*i]).collect::<Vec<_>>()
                }
                None => positions.collect(),
            })
            .ok_or(error::Error::MissingPositions)?;

        trace!("Loading normals");
        let normals = reader
            .read_normals()
            .map(|normals| match faces {
                Some(ref faces) => {
                    let normals = normals.collect::<Vec<_>>();
                    faces.iter().map(|i| normals[*i]).collect()
                }
                None => normals.collect(),
            })
            .unwrap_or_else(|| {
                use amethyst_core::math::Point3;
                use std::iter::once;
                let f = faces
                    .as_ref()
                    .map(|f| f.clone())
                    .unwrap_or_else(|| (0..positions.len()).collect::<Vec<_>>());
                f.chunks(3)
                    .flat_map(|chunk| {
                        let a = Point3::from(positions[chunk[0]]);
                        let ab = Point3::from(positions[chunk[1]]) - a;
                        let ac = Point3::from(positions[chunk[2]]) - a;
                        let normal: [f32; 3] = ab.cross(&ac).into();
                        once(normal.clone())
                            .chain(once(normal.clone()))
                            .chain(once(normal))
                    })
                    .collect::<Vec<_>>()
            });

        trace!("Loading texture coordinates");
        let tex_coord = reader
            .read_tex_coords(0)
            .map(|tex_coords| tex_coords.into_f32().collect::<Vec<[f32; 2]>>())
            .unwrap_or_else(|| {
                vec![
                    [options.generate_tex_coords.0, options.generate_tex_coords.1];
                    positions.len()
                ]
            });
        let tex_coord: Vec<[f32; 2]> = match faces {
            Some(ref faces) => faces
                .iter()
                .map(|i| flip_check(tex_coord[*i], options.flip_v_coord))
                .collect(),
            None => tex_coord
                .into_iter()
                .map(|t| flip_check(t, options.flip_v_coord))
                .collect(),
        };

        trace!("Loading tangents");
        let tangents = reader
            .read_tangents()
            .map(|tangents| match faces {
                Some(ref faces) => {
                    let tangents = tangents.collect::<Vec<_>>();
                    faces
                        .iter()
                        .map(|i| [tangents[*i][0], tangents[*i][1], tangents[*i][2]])
                        .collect()
                }
                None => tangents.map(|t| [t[0], t[1], t[2]]).collect(),
            })
            .unwrap_or_else(|| calculate_tangents(&positions, &normals, &tex_coord));

        trace!("Loading bounding box");
        let bounds = primitive.bounding_box();
        let bounds = bounds.min..bounds.max;

        trace!("Loading colors");
        let colors = reader
            .read_colors(0)
            .map(|colors| colors.into_rgba_f32())
            .map(|colors| match faces {
                Some(ref faces) => {
                    let colors = colors.collect::<Vec<_>>();
                    faces.iter().map(|i| colors[*i]).collect()
                }
                None => colors.collect(),
            });

        trace!("Loading joint ids");
        let joint_ids = reader
            .read_joints(0)
            .map(|joints| joints.into_u16())
            .map(|joints| match faces {
                Some(ref faces) => {
                    let joints = joints.collect::<Vec<_>>();
                    faces.iter().map(|i| joints[*i]).collect()
                }
                None => joints.collect(),
            });
        trace!("Joint ids: {:?}", joint_ids);

        trace!("Loading joint weights");
        let joint_weights = reader
            .read_weights(0)
            .map(|weights| weights.into_f32())
            .map(|weights| match faces {
                Some(ref faces) => {
                    let weights = weights.collect::<Vec<_>>();
                    faces.iter().map(|i| weights[*i]).collect()
                }
                None => weights.collect(),
            });
        trace!("Joint weights: {:?}", joint_weights);

        let material = primitive.material().index();

        let creator = AnimatedComboMeshCreator::new((
            cast_attribute(positions),
            colors.map(cast_attribute),
            Some(cast_attribute(tex_coord)),
            Some(cast_attribute(normals)),
            Some(cast_attribute(tangents)),
            joint_ids.map(cast_attribute),
            joint_weights.map(cast_attribute),
        ));

        primitives.push((creator.into(), material, bounds));
    }
    trace!("Loaded mesh");
    Ok(primitives)
}

fn calculate_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    tex_coords: &[[f32; 2]],
) -> Vec<[f32; 3]> {
    generate_tangents(positions, normals, tex_coords)
        .iter()
        .map(|(_, t)| [t[0], t[1], t[2]])
        .collect()
}

fn generate_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    tex_coords: &[[f32; 2]],
) -> Vec<(usize, [f32; 4])> {
    let vertices_per_face = || 3;
    let face_count = || positions.len() / 3;
    let position = |face, vert| &positions[face * 3 + vert];
    let normal = |face, vert| &normals[face * 3 + vert];
    let tx = |face, vert| &tex_coords[face * 3 + vert];
    let mut tangents: Vec<(usize, [f32; 4])> = Vec::with_capacity(positions.len());

    {
        let mut set_tangent = |face, vert, tangent| {
            let index = face * 3 + vert;
            if let Err(pos) = tangents.binary_search_by(|probe| probe.0.cmp(&index)) {
                tangents.insert(pos, (index, tangent));
            }
        };
        mikktspace::generate_tangents(
            &vertices_per_face,
            &face_count,
            &position,
            &normal,
            &tx,
            &mut set_tangent,
        );
    }

    tangents
}

fn cast_attribute<T>(mut old: Vec<T::Repr>) -> Vec<Separate<T>>
where
    T: Attribute,
{
    let new = unsafe {
        Vec::from_raw_parts(
            old.as_mut_ptr() as *mut Separate<T>,
            old.len(),
            old.capacity(),
        )
    };
    ::std::mem::forget(old);
    new
}

fn flip_check(uv: [f32; 2], flip_v: bool) -> [f32; 2] {
    if flip_v {
        [uv[0], 1. - uv[1]]
    } else {
        uv
    }
}

#[cfg(test)]
mod tests {
    use crate::format::mesh::calculate_tangents;

    #[test]
    fn test_tangent_calc() {
        let positions = &[
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = &[
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let tex_coords = &[
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];

        let tangents = calculate_tangents(positions, normals, tex_coords);

        assert_eq!(
            tangents,
            vec![
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0]
            ]
        );
    }
}
