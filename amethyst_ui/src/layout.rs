use amethyst_core::{GlobalTransform,Transform,Parent};
use amethyst_renderer::ScreenDimensions;
use specs::{Component,Entities,VecStorage,ReadStorage,WriteStorage,Fetch,System,Join};
use super::{UiTransform};
/// Y,X naming
pub enum Anchor{
    TopLeft,
    TopMiddle,
    TopRight,
    MiddleLeft,
    Middle,
    MiddleRight,
    BottomLeft,
    BottomMiddle,
    BottomRight,
}

pub enum Stretch{
    X,
    Y,
    Both,
}

/// Relative to parent
pub struct Anchored{
    anchor: Anchor,
    /// Defaults to none.
    /// While the position value in UiTransform will be changed,
    /// this keeps track of the offset from the anchor.
    /// By default, it will automatically be set to the UiTransform position before it gets moved by the layout system.
    offset: Option<(f32,f32)>,
}

impl Anchored{
    pub fn new(anchor: Anchor) -> Self {
        Anchored{
            anchor,
            offset: None,
        }
    }
}

impl Component for Anchored{
    type Storage = VecStorage<Self>;
}

pub struct Stretched{
    stretch: Stretch,
    /// default to 0,0; in builder use .with_margin
    margin: (f32,f32),
}

impl Stretched{
    pub fn new(stretch: Stretch) -> Self {
        Stretched{
            stretch,
            margin: (0.0,0.0),
        }
    }
}

impl Component for Stretched{
    type Storage = VecStorage<Self>;
}



pub struct UiLayoutSystem {

}

impl UiLayoutSystem{
    /// Creates a new UiLayoutSystem.
    pub fn new() -> Self {
        UiLayoutSystem {

        }
    }
}

impl<'a> System<'a> for UiLayoutSystem{
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, Anchored>,
        ReadStorage<'a, Stretched>,
        ReadStorage<'a, Parent>,
        Fetch<'a, ScreenDimensions>,
    );

    fn run(&mut self, (entities, mut transform, mut anchor, stretch, parent, screen_dim): Self::SystemData) {
        for (mut tr, mut anchor) in (&mut transform, &mut anchor).join(){
            if anchor.offset.is_none(){
                anchor.offset = Some((tr.x,tr.y));
            }

            let norm_offset = match anchor.anchor{
                Anchor::TopLeft => (0.0,0.0),
                Anchor::TopMiddle => (0.5,0.0),
                Anchor::TopRight => (1.0,0.0),
                Anchor::MiddleLeft => (0.0,0.5),
                Anchor::Middle => (0.5,0.5),
                Anchor::MiddleRight => (1.0,0.5),
                Anchor::BottomLeft => (0.0,1.0),
                Anchor::BottomMiddle => (0.5,1.0),
                Anchor::BottomRight => (1.0,1.0),
            };

            let user_offset = anchor.offset.unwrap();

            let new_pos_x = norm_offset.0 * screen_dim.width() + user_offset.0;
            let new_pos_y = norm_offset.1 * screen_dim.height() + user_offset.1;

            tr.x = new_pos_x;
            tr.y = new_pos_y;

        }
    }
}