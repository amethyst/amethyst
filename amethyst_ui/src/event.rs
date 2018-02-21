use shrev::{EventChannel, ReaderId};
use specs::{Entities,Entity,ReadStorage,Component, DenseVecStorage,NullStorage, Fetch, Join, System, WriteStorage,FetchMut};
use specs::saveload::U64Marker;
use winit::{Event, WindowEvent};
use amethyst_renderer::MouseButton;
use amethyst_input::InputHandler;
use transform::UiTransform;
use resize::UiResize;

#[derive(Debug)]
pub enum UiEventType{
    Click,
    ClickStart,
    ClickStop,
    HoverStart,
    HoverStop
}

#[derive(Debug)]
pub struct UiEvent{
    pub event_type: UiEventType,
    pub target: Entity,
}

impl UiEvent{
    pub fn new(event_type: UiEventType, target: Entity) -> Self {
        UiEvent{
            event_type,
            target,
        }
    }
}

#[derive(Default)]
pub struct Clickable;

impl Component for Clickable{
    type Storage = NullStorage<Clickable>;
}

#[derive(Default)]
pub struct MouseReactive;

impl Component for MouseReactive{
    type Storage = NullStorage<MouseReactive>;
}

pub struct UiMouseSystem{
    was_down: bool,
    old_pos: (f32,f32),
}

impl UiMouseSystem{
    pub fn new() -> Self{
        UiMouseSystem{
            was_down: false,
            old_pos: (0.0,0.0),
        }
    }

    fn pos_in_rect(&self, pos_x: f32, pos_y: f32, rect_x: f32, rect_y: f32, width: f32, height: f32) -> bool {
        pos_x > rect_x && pos_x < rect_x + width && pos_y > rect_y && pos_y < rect_y + height
    }
}

impl<'a> System<'a> for UiMouseSystem{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, UiTransform>,
        ReadStorage<'a, MouseReactive>,
        Fetch<'a, InputHandler<String,String>>,
        FetchMut<'a, EventChannel<UiEvent>>,
    );

    fn run(&mut self, (entities, transform, react, input, mut events): Self::SystemData) {
        let down = input.mouse_button_is_down(MouseButton::Left);

        // to replace on InputHandler generate OnMouseDown and OnMouseUp events
        let click_started = down && !self.was_down;
        let click_stopped = !down && self.was_down;

        for (tr,e,_) in (&transform, &*entities, &react).join(){
            if let Some((pos_x,pos_y)) = input.mouse_position() {
                let x = pos_x as f32;
                let y = pos_y as f32;
                let is_in_rect = self.pos_in_rect(x,y,tr.x,tr.y,tr.width,tr.height);
                let was_in_rect = self.pos_in_rect(self.old_pos.0,self.old_pos.1,tr.x,tr.y,tr.width,tr.height);


                // I think the way it usually works is that you have to both start and end the click on the same element.
                // This could be done by selecting the element on click down, and when releasing the click, check if we are on the selected element.
                // Need a selection system for that to work tho
                if is_in_rect{
                    if click_started {
                        println!("Clicked something");
                        // Missing check for clickable component


                        // Use UiTransform or Entity ref?

                        events.single_write(UiEvent::new(UiEventType::ClickStart,e));
                    }
                }
            }
        }

        self.was_down = down;
    }
}


// Just to show how to handle element clicks. I'm not actually sure where we want the on_click() code that is specific to each clickable element.
// Could make a system looping through, but you'd have to know which button is which.
pub struct ClickableSystem{
    reader_id: Option<ReaderId<U64Marker>>,
}

impl ClickableSystem{
    pub fn new() -> Self {
        ClickableSystem{
            reader_id: None,
        }
    }
}

impl<'a> System<'a> for ClickableSystem{
    type SystemData = Fetch<'a, EventChannel<UiEvent>>;

    fn run(&mut self, events: Self::SystemData) {
        if self.reader_id.is_none(){
            self.reader_id = Some(events.register_reader());
        }
        for ev in events.read(self.reader_id.as_mut().unwrap()){
            println!("You clicked a clickable(comp WIP) element! ;)");
        }
    }
}