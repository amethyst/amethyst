//! Util functions that change how the mouse moves and looks.

use super::WindowMessages;
use winit::{CursorState, MouseCursor};

/// Hide the cursor, so it's invisible while playing. Can't be used at the same time as grab_cursor.
pub fn hide_cursor(msg: &mut WindowMessages) {
    change_cursor_state(msg, CursorState::Hide);
}

/// Set the cursor back to normal/visible.
pub fn release_cursor(msg: &mut WindowMessages) {
    change_cursor_state(msg, CursorState::Normal);
}

/// Grab the cursor to prevent it from going outside the screen.
pub fn grab_cursor(msg: &mut WindowMessages) {
    change_cursor_state(msg, CursorState::Grab);
}

fn change_cursor_state(msg: &mut WindowMessages, state: CursorState) {
    msg.send_command(move |win| {
        if let Err(err) = win.set_cursor_state(state) {
            error!("Unable to change the cursor state! Error: {:?}", err);
        }
    });
}

/// Hide the cursor, so it's invisible while player. Can be used at the same time as grab_cursor.
pub fn set_mouse_cursor_none(msg: &mut WindowMessages) {
    set_mouse_cursor(msg, MouseCursor::NoneCursor);
}

/// Sets the mouse cursor icon.
pub fn set_mouse_cursor(msg: &mut WindowMessages, cursor: MouseCursor) {
    msg.send_command(move |win| {
        win.set_cursor(cursor);
    });
}
