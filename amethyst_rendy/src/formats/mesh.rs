use crate::types::Mesh;
use amethyst_assets::SimpleFormat;
use amethyst_error::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct ObjFormat;

impl SimpleFormat<Mesh> for ObjFormat {
    const NAME: &'static str = "WAVEFRONT_OBJ";
    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<rendy::mesh::MeshBuilder<'static>, Error> {
        rendy::mesh::obj::load_from_obj(&bytes, ()).map_err(|e| e.compat().into())
    }
}
