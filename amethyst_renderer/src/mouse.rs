//! Util functions that change how the mouse moves and looks.

use log::error;
use winit::MouseCursor;

use super::WindowMessages;

/// Hide the cursor, so it's invisible while playing.
pub fn hide_cursor(msg: &mut WindowMessages) {
    msg.send_command(move |win| win.hide_cursor(true));
}

/// Set the cursor back to normal/visible.
pub fn release_cursor(msg: &mut WindowMessages) {
    msg.send_command(move |win| {
        if let Err(err) = win.grab_cursor(false) {
            error!("Unable to release the cursor. Error: {:?}", err);
        }
        win.hide_cursor(false);
    });
}

/// Grab the cursor to prevent it from going outside the screen.
pub fn grab_cursor(msg: &mut WindowMessages) {
    msg.send_command(move |win| {
        if let Err(err) = win.grab_cursor(true) {
            error!("Unable to grab the cursor. Error: {:?}", err);
        }
    });
}

/// Sets the mouse cursor icon.
pub fn set_mouse_cursor(msg: &mut WindowMessages, cursor: MouseCursor) {
    msg.send_command(move |win| {
        win.set_cursor(cursor);
    });
}
