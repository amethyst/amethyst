use winit::{CursorState,MouseCursor};
use amethyst_renderer::{ScreenDimensions,WindowMessages};
use specs::{Fetch, FetchMut, System};

/// Hide the cursor, so it's invisible while playing.
pub fn hide_cursor(msg: &mut WindowMessages) {
    change_cursor_state(msg,CursorState::Hide);
}

pub fn release_cursor(msg: &mut WindowMessages){
    change_cursor_state(msg,CursorState::Normal);
}

pub fn grab_cursor(msg: &mut WindowMessages){
    change_cursor_state(msg,CursorState::Grab);
}

fn change_cursor_state(msg: &mut WindowMessages, state: CursorState){
    msg.send_command(move |win| {
        if let Err(err) = win.set_cursor_state(state) {
            eprintln!("Unable to change the cursor state! Error: {:?}", err);
        }
    });
}

pub fn set_mouse_cursor_none(msg: &mut WindowMessages) {
    set_mouse_cursor(msg,MouseCursor::NoneCursor);
}

pub fn set_mouse_cursor(msg: &mut WindowMessages,cursor:MouseCursor){
    msg.send_command(move |win| {
        win.set_cursor(cursor);
    });
}

pub struct MouseCenterLockSystem;

impl<'a> System<'a> for MouseCenterLockSystem<> {
    type SystemData = (
        Fetch<'a, ScreenDimensions>,
        FetchMut<'a, WindowMessages>,
    );

    fn run(&mut self, (dim,mut msg): Self::SystemData){
        let half_x = dim.width() as i32 / 2;
        let half_y = dim.height() as i32 / 2;
        msg.send_command(move |win|{
            if let Err(err) = win.set_cursor_position(half_x,half_y) {
                eprintln!("Unable to set the cursor position! Error: {:?}", err);
            }
        });
    }
}