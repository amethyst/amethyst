//! The components that make up a rendering pipeline.

/// A single, atomic rendering operation.
#[derive(Debug)]
pub enum Step {
    /// Clears the current render target.
    ClearTarget {
        /// Which buffers to clear. Possible values: "all", "color", "stencil".
        buffers: String,
        /// The RGBA value to clear the buffers with. If `None`, this will
        /// default to `[0.0, 0.0, 0.0, 0.0]`.
        value: Option<[f32; 4]>,
    },
    /// Draws all objects in the scene.
    DrawObjects {
        shader: String,
    },
    /// Selects a render target to write to. If the given string is empty
    /// (`""`), we render directly to the window surface.
    UseTarget(String),
}

/// A collection of steps that accomplishes some task in the rendering pipeline.
#[derive(Debug)]
pub struct Stage {
    name: String,
    steps: Vec<Step>,
}

impl Stage {
    /// Defines a new pipeline stage and assigns it a descriptive name.
    pub fn new(name: &str) -> Stage {
        Stage {
            name: name.to_string(),
            steps: Vec::new(),
        }
    }
}
