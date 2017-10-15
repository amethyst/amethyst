//! MODULE DISABLED, will be reused later!

use Source;

pub trait Reload {
    fn needs_reload(&self, source: &Source) -> bool;
}

pub struct SingleFile {
    modified: u64,
    path: String,
}

impl Reload for SingleFile {
    fn needs_reload(&self, source: &Source) -> bool {
        self.modified != 0 && source.modified(&self.path).unwrap_or(0) > self.modified
    }
}
