use amethyst_assets::{
    erased_serde::private::serde::{de, de::SeqAccess, ser::SerializeSeq},
    prefab::{
        register_component_type,
        serde_diff::{ApplyContext, DiffContext},
        SerdeDiff,
    },
    DefaultLoader, Handle, Loader,
};
use amethyst_core::ecs::{component, Entity, EntityStore, IntoQuery, Resources, World};
use rendy::mesh::{MeshBuilder, Normal, Position, Tangent, TexCoord};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use crate::{shape::Shape, types::MeshData, Mesh};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Vertex format to use when generating the mesh.
pub enum VertexFormat {
    /// Use a tuple with Positions and Textures
    VecPosTex,
    /// Tuple with Positions, Normals and Textures
    VecPosNormTex,
    /// Tuple with Positions, Normals Tangents and Textures
    VecPosNormTangTex,
}

/// Component provided to create a Mesh from a simple Shape from inside a Prefab
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "046b3348-424f-4db2-8888-f01206aa1dad"]
pub struct ShapePrefab {
    /// Generate a Mesh from a basic type
    shape: Shape,
    /// Vertex format to use
    format: VertexFormat,
    /// scale to apply to the shape
    scale: Option<(f32, f32, f32)>,
}

impl SerdeDiff for ShapePrefab {
    fn diff<'a, S: SerializeSeq>(
        &self,
        ctx: &mut DiffContext<'a, S>,
        other: &Self,
    ) -> Result<bool, <S as SerializeSeq>::Error> {
        unimplemented!()
    }

    fn apply<'de, A>(
        &mut self,
        seq: &mut A,
        ctx: &mut ApplyContext,
    ) -> Result<bool, <A as SeqAccess<'de>>::Error>
    where
        A: de::SeqAccess<'de>,
    {
        unimplemented!()
    }
}

impl Default for ShapePrefab {
    fn default() -> Self {
        unimplemented!()
    }
}

register_component_type!(ShapePrefab);

/// Attaches a Handle<Mesh> to any Entity with a MeshPrefab
pub fn shape_prefab_spawning_tick(world: &mut World, resources: &mut Resources) {
    let loader = resources.get::<DefaultLoader>().unwrap();
    let mut query = <(Entity, &ShapePrefab)>::query().filter(!component::<Handle<Mesh>>());
    let mut accumulator = Vec::new();

    query.for_each(world, |(e, shape)| {
        let handle = loader.load_from_data::<Mesh, (), MeshData>(
            generate_shape(&shape).into(),
            (),
            &resources.get().unwrap(),
        );
        accumulator.push((*e, handle));
    });

    while let Some((e, h)) = accumulator.pop() {
        world
            .entry(e)
            .expect("Unreachable, this entity is currently requested")
            .add_component(h);
    }
}

fn generate_shape(shape_prefab: &ShapePrefab) -> MeshBuilder<'static> {
    match shape_prefab.format {
        VertexFormat::VecPosTex => {
            shape_prefab
                .shape
                .generate::<(Vec<Position>, Vec<TexCoord>)>(shape_prefab.scale)
        }
        VertexFormat::VecPosNormTex => {
            shape_prefab
                .shape
                .generate::<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>(shape_prefab.scale)
        }
        VertexFormat::VecPosNormTangTex => {
            shape_prefab
                .shape
                .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(
                    shape_prefab.scale,
                )
        }
    }
}
