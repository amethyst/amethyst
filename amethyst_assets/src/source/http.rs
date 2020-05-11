use js_sys::Uint8Array;
use std::path::{Path, PathBuf};
use wasm_bindgen::prelude::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_error::Error;

use crate::source::Source;

#[wasm_bindgen]
extern "C" {
    fn asset_fetch(path: &str) -> Uint8Array;
}

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

        // IMPORTANT!
        //
        // In WASM, functions that return `Result<_, JsValue>` can leak stack space.
        // Therefore we should not return `Err(_)`s in this function in order for WASM applications to be "stable".
        //
        // See <https://github.com/rustwasm/wasm-bindgen/issues/1963>

        let path = self.path(path);
        let path_str = path
            .to_str()
            .unwrap_or_else(|| panic!("Path contains non-unicode characters: {:?}", path));

        let arr = asset_fetch(path_str);
        let mut v = vec![0; arr.length() as usize];
        arr.copy_to(&mut v);

        Ok(v)
    }
}
