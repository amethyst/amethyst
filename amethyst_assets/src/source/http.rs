use js_sys::Uint8Array;
use std::path::{Path, PathBuf};
use web_sys::{XmlHttpRequest, XmlHttpRequestResponseType};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_error::{format_err, Error, ResultExt};

use crate::{error, source::Source};

/// HTTP source.
///
/// Loads assets inside web worker using XmlHttpRequest.
/// Used as a default source for WASM target.
#[derive(Debug)]
pub struct HttpSource {
    loc: PathBuf,
}

impl HttpSource {
    /// Creates a new http source.
    pub fn new<P>(loc: P) -> Self
    where
        P: Into<PathBuf>,
    {
        HttpSource { loc: loc.into() }
    }

    fn path(&self, s_path: &str) -> PathBuf {
        let mut path = self.loc.clone();
        path.extend(Path::new(s_path).iter());

        path
    }
}

impl Source for HttpSource {
    fn modified(&self, _path: &str) -> Result<u64, Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("http_modified_asset");

        // Unimplemented. Maybe possible to tie into webpack hot module reloading?
        Ok(0)
    }

    fn load(&self, path: &str) -> Result<Vec<u8>, Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("http_load_asset");

        let path = self.path(path);
        let path_str = path
            .to_str()
            .ok_or_else(|| format_err!("Path contains non-unicode characters: {:?}", path))
            .with_context(|_| error::Error::Source)?;

        let xhr = XmlHttpRequest::new()
            .map_err(|_| format_err!("Failed to construct XmlHttpRequest"))
            .with_context(|_| error::Error::Source)?;

        // Synchronous GET request. Should only be run in web worker.
        xhr.open_with_async("GET", path_str, false)
            .map_err(|e| format_err!("XmlHttpRequest open failed: `{:?}`", e))
            .with_context(|_| error::Error::Source)?;
        xhr.set_response_type(XmlHttpRequestResponseType::Arraybuffer);

        // We block here and wait for http fetch to complete
        xhr.send()
            .map_err(|e| format_err!("XmlHttpRequest send failed: `{:?}`", e))
            .with_context(|_| error::Error::Source)?;

        // Status returns a result but according to javascript spec it should never return error.
        // Returns 0 if request was not completed.
        let status = xhr
            .status()
            .map_err(|e| format_err!("XmlHttpRequest `status` read failed: `{:?}`", e))
            .with_context(|_| error::Error::Source)?;

        if status != 200 {
            let msg = xhr
                .status_text()
                .map_err(|e| format_err!("XmlHttpRequest `status_text` read failed: `{:?}`", e))
                .with_context(|_| error::Error::Source)?;
            return Err(format_err!(
                "XmlHttpRequest failed with code {}. Error: {}",
                status,
                msg
            ))
            .with_context(|_| error::Error::Source);
        }

        let resp = xhr
            .response()
            .map_err(|e| format_err!("XmlHttpRequest `response` read failed: `{:?}`", e))
            .with_context(|_| error::Error::Source)?;

        // Convert javascript ArrayBuffer into Vec<u8>
        let arr = Uint8Array::new(&resp);
        let mut v = vec![0; arr.length() as usize];
        arr.copy_to(&mut v);

        Ok(v)
    }
}
