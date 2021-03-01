use amethyst_assets::distill_importer::Error;
use gltf::Document;

pub fn convert_bytes(
    bytes: &Vec<u8>,
) -> Result<(Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>), Error> {
    log::debug!("Starting Gltf import");
    let result = gltf::import_slice(&bytes.as_slice());
    if let Err(err) = result {
        log::error!("Import error: {:?}", err);
        return Err(Error::Boxed(Box::new(err)));
    }
    Ok(result.unwrap())
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use super::*;

    #[test]
    fn should_import_glb_gltf() {
        let mut f = File::open("test/suzanne.glb").expect("suzanne.glb not found");
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer)
            .expect("read_to_end did not work");
        let gltf_import = convert_bytes(&buffer);
        match gltf_import {
            Ok(gltf) => {
                let doc = gltf.0;
                assert_eq!(2, doc.images().len());
                assert_eq!(1, doc.materials().len());
                assert_eq!(1, doc.meshes().len());
                assert_eq!(1, doc.nodes().len());
                assert_eq!(1, doc.scenes().len());
                assert_eq!(2, doc.textures().len());
            }
            Err(e) => {
                panic!("Error during gltf import {:?}", e)
            }
        }
    }
}
