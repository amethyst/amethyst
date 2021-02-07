use std::{iter::repeat, ops::Range};

use amethyst_assets::error::Error;
use amethyst_core::math::{zero, Vector3};
use amethyst_rendy::{
    rendy::mesh::{Color, MeshBuilder, Normal, Position, Tangent, TexCoord},
    skinning::JointCombined,
};
use gltf::buffer::Data;
use log::{debug, trace, warn};
use mikktspace::{generate_tangents, Geometry};

use crate::GltfSceneOptions;

pub fn load_mesh(
    mesh: &gltf::Mesh<'_>,
    buffers: &Vec<Data>,
    options: &GltfSceneOptions,
) -> Result<Vec<(String, MeshBuilder<'static>, Option<usize>, Range<[f32; 3]>)>, Error> {
    debug!("Loading mesh");
    let mut primitives = vec![];
    for primitive in mesh.primitives() {
        debug!("Loading mesh primitive");
        let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|x| &**x));

        debug!("Loading indices");
        use gltf::mesh::util::ReadIndices;
        let indices = match reader.read_indices() {
            Some(ReadIndices::U8(iter)) => Indices::U16(iter.map(u16::from).collect()),
            Some(ReadIndices::U16(iter)) => Indices::U16(iter.collect()),
            Some(ReadIndices::U32(iter)) => Indices::U32(iter.collect()),
            None => Indices::None,
        };

        debug!("Loading positions");
        let positions = reader
            .read_positions()
            .expect("Missing position !")
            .map(Position)
            .collect::<Vec<_>>();

        let normals = compute_if(options.load_normals || options.load_tangents, || {
            debug!("Loading normals");
            if let Some(normals) = reader.read_normals() {
                normals.map(Normal).collect::<Vec<_>>()
            } else {
                debug!("Calculating normals");
                calculate_normals(&positions, &indices)
            }
        });

        let tex_coords = compute_if(options.load_texcoords || options.load_tangents, || {
            debug!("Loading texture coordinates");
            if let Some(tex_coords) = reader.read_tex_coords(0).map(|t| t.into_f32()) {
                if options.flip_v_coord {
                    tex_coords
                        .map(|[u, v]| TexCoord([u, 1. - v]))
                        .collect::<Vec<_>>()
                } else {
                    tex_coords.map(TexCoord).collect::<Vec<_>>()
                }
            } else {
                let (u, v) = options.generate_tex_coords;
                let v = if options.flip_v_coord { v } else { 1.0 - v };
                repeat(TexCoord([u, v]))
                    .take(positions.len())
                    .collect::<Vec<_>>()
            }
        });

        let tangents = compute_if(options.load_tangents, || {
            debug!("Loading tangents");
            let tangents = reader.read_tangents();
            match tangents {
                Some(tangents) => tangents.map(Tangent).collect::<Vec<_>>(),
                None => {
                    debug!("Calculating tangents");
                    calculate_tangents(
                        &positions,
                        normals.as_ref().unwrap(),
                        tex_coords.as_ref().unwrap(),
                        &indices,
                    )
                }
            }
        });

        let colors = try_compute_if(options.load_colors, || {
            debug!("Loading colors");
            if let Some(colors) = reader.read_colors(0) {
                Some(colors.into_rgba_f32().map(Color).collect::<Vec<_>>())
            } else {
                None
            }
        });

        let joints = try_compute_if(options.load_animations, || {
            debug!("Loading animations");
            if let (Some(ids), Some(weights)) = (reader.read_joints(0), reader.read_weights(0)) {
                let zip = ids.into_u16().zip(weights.into_f32());
                let joints = zip
                    .map(|(ids, weights)| JointCombined::new(ids, weights))
                    .collect::<Vec<_>>();

                Some(joints)
            } else {
                None
            }
        });

        let mut builder = MeshBuilder::new();

        match indices {
            Indices::U16(vec) => {
                builder.set_indices(vec);
            }
            Indices::U32(vec) => {
                builder.set_indices(vec);
            }
            Indices::None => {}
        };

        builder.add_vertices(positions);
        normals.map(|v| builder.add_vertices(v));
        tangents.map(|v| builder.add_vertices(v));
        tex_coords.map(|v| builder.add_vertices(v));
        colors.map(|v| builder.add_vertices(v));
        joints.map(|v| builder.add_vertices(v));

        trace!("Loading bounding box");
        let bounds = primitive.bounding_box();
        let bounds = bounds.min..bounds.max;
        let material = primitive.material().index();

        primitives.push((
            mesh.name().expect("Meshes must have a name").to_string(),
            builder,
            material,
            bounds,
        ));
    }
    trace!("Loaded mesh");
    Ok(primitives)
}

fn compute_if<T, F: Fn() -> T>(predicate: bool, func: F) -> Option<T> {
    if predicate {
        Some(func())
    } else {
        None
    }
}

fn try_compute_if<T, F: Fn() -> Option<T>>(predicate: bool, func: F) -> Option<T> {
    if predicate {
        func()
    } else {
        None
    }
}

enum Indices {
    None,
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Indices {
    fn len(&self) -> Option<usize> {
        match self {
            Indices::None => None,
            Indices::U16(vec) => Some(vec.len()),
            Indices::U32(vec) => Some(vec.len()),
        }
    }

    fn map(&self, face: usize, vert: usize) -> usize {
        match self {
            Indices::None => face * 3 + vert,
            Indices::U16(vec) => vec[face * 3 + vert] as usize,
            Indices::U32(vec) => vec[face * 3 + vert] as usize,
        }
    }
}

fn calculate_normals(positions: &[Position], indices: &Indices) -> Vec<Normal> {
    let mut normals = vec![zero::<Vector3<f32>>(); positions.len()];
    let num_faces = indices.len().unwrap_or_else(|| positions.len()) / 3;
    for face in 0..num_faces {
        let i0 = indices.map(face, 0);
        let i1 = indices.map(face, 1);
        let i2 = indices.map(face, 2);
        let a = Vector3::from(positions[i0].0);
        let b = Vector3::from(positions[i1].0);
        let c = Vector3::from(positions[i2].0);
        let n = (b - a).cross(&(c - a));
        normals[i0] += n;
        normals[i1] += n;
        normals[i2] += n;
    }
    normals
        .into_iter()
        .map(|n| Normal(n.normalize().into()))
        .collect::<Vec<_>>()
}

struct TangentsGeometry<'a> {
    tangents: Vec<Tangent>,
    num_faces: usize,
    positions: &'a [Position],
    normals: &'a [Normal],
    tex_coords: &'a [TexCoord],
    indices: &'a Indices,
}

impl<'a> Geometry for TangentsGeometry<'a> {
    fn num_faces(&self) -> usize {
        self.num_faces
    }
    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }
    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.positions[self.indices.map(face, vert)].0
    }
    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.normals[self.indices.map(face, vert)].0
    }
    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.tex_coords[self.indices.map(face, vert)].0
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let [x, y, z, w] = tangent;
        self.tangents[self.indices.map(face, vert)] = Tangent([x, y, z, -w]);
    }
}

fn calculate_tangents(
    positions: &[Position],
    normals: &[Normal],
    tex_coords: &[TexCoord],
    indices: &Indices,
) -> Vec<Tangent> {
    let mut geometry = TangentsGeometry {
        tangents: vec![Tangent([0.0, 0.0, 0.0, 0.0]); positions.len()],
        num_faces: indices.len().unwrap_or_else(|| positions.len()) / 3,
        positions,
        normals,
        tex_coords,
        indices,
    };

    if !generate_tangents(&mut geometry) {
        warn!("Could not generate tangents!");
    }

    geometry.tangents
}

#[cfg(test)]
mod tests {
    use amethyst_rendy::rendy::mesh::{Normal, Position, Tangent, TexCoord};

    use super::{calculate_tangents, Indices};

    const POSITIONS: &[Position] = &[
        Position([0.0, 0.0, 0.0]),
        Position([0.0, 1.0, 0.0]),
        Position([1.0, 1.0, 0.0]),
        Position([0.0, 1.0, 0.0]),
        Position([1.0, 1.0, 0.0]),
        Position([1.0, 0.0, 0.0]),
    ];
    const NORMALS: &[Normal] = &[
        Normal([0.0, 0.0, 1.0]),
        Normal([0.0, 0.0, 1.0]),
        Normal([0.0, 0.0, 1.0]),
        Normal([0.0, 0.0, 1.0]),
        Normal([0.0, 0.0, 1.0]),
        Normal([1.0, 0.0, 0.0]),
    ];
    const TEX_COORDS: &[TexCoord] = &[
        TexCoord([0.0, 0.0]),
        TexCoord([0.0, 1.0]),
        TexCoord([1.0, 1.0]),
        TexCoord([0.0, 1.0]),
        TexCoord([1.0, 1.0]),
        TexCoord([1.0, 0.0]),
    ];

    #[test]
    fn test_tangent_calc() {
        let tangents = calculate_tangents(POSITIONS, NORMALS, TEX_COORDS, &Indices::None);
        assert_eq!(
            tangents,
            vec![
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([0.0, 0.0, 0.0, 1.0]),
            ]
        );
    }

    #[test]
    fn test_indexed_tangent_calc() {
        let tangents = calculate_tangents(
            POSITIONS,
            NORMALS,
            TEX_COORDS,
            &Indices::U32(vec![3, 4, 5, 0, 1, 2]),
        );
        assert_eq!(
            tangents,
            vec![
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([1.0, 0.0, 0.0, 1.0]),
                Tangent([0.0, 0.0, 0.0, 1.0]),
            ]
        );
    }
}
