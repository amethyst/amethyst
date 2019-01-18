use std::{str::Utf8Error, string::FromUtf8Error};

use error_chain::*;

error_chain! {
    foreign_links {
        FromUtf8(FromUtf8Error) #[doc = "Wraps a UTF-8 error"];
        Utf8(Utf8Error) #[doc = "Wraps a UTF-8 error"];
    }

    errors {
        /// Returned if an asset with a given name failed to load.
        Asset(name: String) {
            description("Failed to load asset")
            display("Failed to load asset with name {:?}", name)
        }

        /// Returned if a source could not retrieve something.
        Source {
            description("Failed to load bytes from source")
        }

        /// Returned if a format failed to load the asset data.
        Format(format: &'static str) {
            description("Format could not load asset")
            display("Format {:?} could not load asset", format)
        }

        /// Returned if an asset is loaded and never used.
        UnusedHandle {
            description("Asset was loaded but no handle to it was saved.")
        }
    }
}
