//! Encodes information about how to draw the scene.

mod stage;

pub use self::stage::{Stage, Step};

/// A set of stages that describes how to draw a frame.
#[derive(Debug)]
pub struct Pipeline {
    pub name: String,
    pub stages: Vec<Stage>,
}

impl Pipeline {
    /// Creates an empty pipeline and assigns it a descriptive name.
    pub fn new(name: &str) -> Pipeline {
        Pipeline {
            name: name.to_string(),
            stages: Vec::new(),
        }
    }

    pub fn build(name: &str) -> PipelineBuilder {
        PipelineBuilder::new(name)
    }
}

pub struct PipelineBuilder {
    cur_stage: Option<Stage>,
    pipe: Pipeline,
}

impl PipelineBuilder {
    pub fn new(name: &str) -> PipelineBuilder {
        PipelineBuilder {
            cur_stage: None,
            pipe: Pipeline::new(name),
        }
    }

    pub fn new_stage(mut self, name: &str) -> PipelineBuilder {
        if let Some(s) = self.cur_stage {
            self.pipe.stages.push(s);
        }

        self.cur_stage = Some(Stage::new(name));
        self
    }

    pub fn done(self) -> Pipeline {
        self.pipe
    }
}
