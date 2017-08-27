use amethyst::audio::
use amethyst::prelude::*;

struct DjState;

impl State for DjState {}

fn main() {
    let app = Application::build(DjState).unwrap()
        .with::<Dj>
}
