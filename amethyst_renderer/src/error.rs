//! Renderer error types.

use gfx;
use gfx_core;
use glutin;

error_chain! {

    foreign_links {

        BufferCreation(gfx::buffer::CreationError)
            #[doc="Error occured during buffer creation"];
        PipelineState(gfx::PipelineStateError<String>)
            #[doc="Error occured in pipeline state"];
        PsoCreation(gfx_core::pso::CreationError)
            #[doc="Error occured during pipeline state object creation"];
        ShaderProgram(gfx::shade::ProgramError)
            #[doc="Error occured during shader compilation"];
        ResourceView(gfx::ResourceViewError)
            #[doc="Error occured in resource view"];
        GfxCombined(gfx::CombinedError)
            #[doc="Gfx combined error type"];
        TextureCreation(gfx::texture::CreationError)
            #[doc="Error occured during texture creation"];
        GlutinContext(glutin::ContextError) // todo #[cfg(glutin)] only
            #[doc="(gl specific) Error occured in gl context"];
    }

    errors {
        /// A render target with the given name does not exist.
        NoSuchTarget(e: String) {
            description("Target with this name does not exist!")
            display("Nonexistent target: {}", e)
        }
        /// Failed to create thread pool.
        PoolCreation(e: String) {
            description("Failed to create thread pool!")
            display("Thread pool creation failed: {}", e)
        }
        /// An error occuring in buffer/texture updates.
        BufTexUpdate {
            description("An error occured during buffer/texture update")
        }
        /// No global with given name could be found.
        MissingGlobal(name: String) {
            description("No global was found with the given name")
            display(r#"No global was found with the name "{}""#, name)
        }
        /// No buffer with given name could be found.
        MissingBuffer(name: String) {
            description("No buffer was found with the given name")
            display(r#"No buffer was found with the name "{}""#, name)
        }
        /// No constant buffer with given name could be found.
        MissingConstBuffer(name: String) {
            description("No constant buffer was found with the given name")
            display(r#"No constant buffer was found with the name "{}""#, name)
        }
        /// A list of all errors that occured during render
        DrawErrors(errors: Vec<Error>) {
            description("One or more errors occured during drawing")
            display("One or more errors occured during drawing: {:?}", errors)
        }
        /// The window handle associated with the renderer has been destroyed.
        WindowDestroyed {
            description("Window has been destroyed!")
        }
    }
}

