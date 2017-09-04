//! Renderer error types.

use gfx;
use gfx_core;
use glutin;

error_chain! {
    errors {
        /// Failed to create a buffer.
        BufferCreation(e: gfx::buffer::CreationError) {
            description("Failed to create buffer!")
            display("Buffer creation failed: {}", e)
        }
        /// A render target with the given name does not exist.
        NoSuchTarget(e: String) {
            description("Target with this name does not exist!")
            display("Nonexistent target: {}", e)
        }
        /// Failed to initialize a render pass.
        PassInit(e: gfx::PipelineStateError<String>) {
            description("Failed to initialize render pass!")
            display("Pass initialization failed: {}", e)
        }
        /// Failed to create a pipeline state object (PSO).
        PipelineCreation(e: gfx_core::pso::CreationError) {
            description("Failed to create PSO!")
            display("PSO creation failed: {}", e)
        }
        /// Failed to create thread pool.
        PoolCreation(e: String) {
            description("Failed to create thread pool!")
            display("Thread pool creation failed: {}", e)
        }
        /// Failed to create and link a shader program.
        ProgramCreation(e: gfx::shade::ProgramError) {
            description("Failed to create shader program!")
            display("Program compilation failed: {}", e)
        }
        /// Failed to create a resource view.
        ResViewCreation(e: gfx::ResourceViewError) {
            description("Failed to create resource view!")
            display("Resource view creation failed: {}", e)
        }

        /// Failed to create a render target.
        TargetCreation(e: gfx::CombinedError) {
            description("Failed to create render target!")
            display("Target creation failed: {}", e)
        }
        /// Failed to create a texture resource.
        TextureCreation(e: gfx::texture::CreationError) {
            description("Failed to create texture!")
            display("Texture creation failed: {}", e)
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
        /// (GL only) An error occured swapping buffers
        BufferSwapFailed(e: glutin::ContextError) {
            description("An error occured swapping the buffers")
            display("An error occured swapping the buffers: {}", e)
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

impl From<gfx::CombinedError> for Error {
    fn from(e: gfx::CombinedError) -> Error {
        ErrorKind::TargetCreation(e).into()
    }
}

impl From<gfx::PipelineStateError<String>> for Error {
    fn from(e: gfx::PipelineStateError<String>) -> Error {
        ErrorKind::PassInit(e).into()
    }
}

impl From<gfx::ResourceViewError> for Error {
    fn from(e: gfx::ResourceViewError) -> Error {
        ErrorKind::ResViewCreation(e).into()
    }
}

impl From<gfx::buffer::CreationError> for Error {
    fn from(e: gfx::buffer::CreationError) -> Error {
        ErrorKind::BufferCreation(e).into()
    }
}

impl From<gfx::shade::ProgramError> for Error {
    fn from(e: gfx::shade::ProgramError) -> Error {
        ErrorKind::ProgramCreation(e).into()
    }
}

impl From<gfx::texture::CreationError> for Error {
    fn from(e: gfx::texture::CreationError) -> Error {
        ErrorKind::TextureCreation(e).into()
    }
}

impl From<gfx_core::pso::CreationError> for Error {
    fn from(e: gfx_core::pso::CreationError) -> Error {
        ErrorKind::PipelineCreation(e).into()
    }
}
