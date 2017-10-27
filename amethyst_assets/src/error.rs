
error_chain! {
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
    }
}
