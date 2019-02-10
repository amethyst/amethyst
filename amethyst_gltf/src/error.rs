use err_derive::Error;

/// Format errors
#[derive(Debug, Error)]
pub enum Error {
    /// Importer failed to load the json file
    #[error(display = "Gltf import error")]
    GltfImporterError,

    /// GLTF have no default scene and the number of scenes is not 1
    #[error(
        display = "Gltf has no default scene, and the number of scenes is not 1: {}",
        _0
    )]
    InvalidSceneGltf(usize),

    /// GLTF primitive missing positions
    #[error(display = "Primitive missing positions")]
    MissingPositions,

    /// GLTF animation channel missing input
    #[error(display = "Channel missing inputs")]
    MissingInputs,

    /// GLTF animation channel missing output
    #[error(display = "Channel missing outputs")]
    MissingOutputs,

    /// Not implemented yet
    #[error(display = "Not implemented")]
    NotImplemented,

    /// A loaded glTF buffer is not of the required length.
    #[error(display = "Loaded buffer does not match required length")]
    BufferLength(gltf::json::Path),
}
