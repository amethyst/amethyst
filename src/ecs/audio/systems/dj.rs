
use std::mem::replace;

use audio::Dj;
use ecs::{FetchMut, System};

/// Calls a Dj's picker as soon as the Dj runs out of music to play.
///
/// This will only operate on a Dj if it has been added to the world as a resource with id 0.
pub struct DjSystem;

impl<'a> System<'a> for DjSystem  {
    type SystemData = FetchMut<'a, Dj>;
    fn run(&mut self, mut dj: Self::SystemData) {
        // Process Dj picker
        if dj.empty() {
            if let Some(mut picker) = replace(&mut dj.picker, None) {
                if picker(&mut dj) {
                    dj.picker = Some(picker);
                }
            }
        }
    }
}
