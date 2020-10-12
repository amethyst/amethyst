use err_derive::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(display = "Failed to load asset with name {:?}", _0)]
    Asset(String),
    #[error(display = "Failed to load bytes from source")]
    Source,
    #[error(display = "Format {:?} could not load asset", _0)]
    Format(&'static str),
    #[error(display = "Asset was loaded but no handle to it was saved.")]
    UnusedHandle,
}
