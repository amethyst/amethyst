#[derive(Debug)]
pub enum Step {
    ClearTarget {
        buffers: String
    },
    DrawGeometry {
        shader: String,
        value: Option<[f32; 4]>,
    },
    UseTarget(String),
}

#[derive(Debug)]
pub struct Stage {
    name: String,
    steps: Vec<Step>,
}

impl Stage {
    pub fn new(name: &str) -> Stage {
        Stage {
            name: name.to_string(),
            steps: Vec::new(),
        }
    }
}
