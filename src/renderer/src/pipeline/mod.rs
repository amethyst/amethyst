mod stage;

pub use self::stage::Step;

pub struct Pipeline {
    name: String,
    steps: Vec<Step>,
}

impl Pipeline {
    pub fn new(name: &str) -> Pipeline {
        Pipeline {
            name: name.to_string(),
            steps: Vec::new(),
        }
    }
}
