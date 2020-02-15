use crate::structs::focus_event::FocusEventRecord;
use crate::structs::input_event::{KeyEventRecord, MouseEventRecord};
use crate::structs::menu_event::MenuEventRecord;
use crate::structs::window_buffer_size_event::WindowBufferSizeRecord;
use winapi::um::wincon::INPUT_RECORD;
use winapi::um::wincontypes::{
    FOCUS_EVENT, KEY_EVENT, MENU_EVENT, MOUSE_EVENT, WINDOW_BUFFER_SIZE_EVENT,
};

/// Represents an `INPUT_RECORD` which describes an input event in the console input buffer.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/input-record-str`
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputRecord {
    /// The Event member contains a `KEY_EVENT_RECORD` structure with
    /// information about a keyboard event.
    KeyEvent(KeyEventRecord),
    /// The Event member contains a `MOUSE_EVENT_RECORD` structure with
    /// information about a mouse movement or button press event.
    MouseEvent(MouseEventRecord),
    /// The Event member contains a `WINDOW_BUFFER_SIZE_RECORD` structure with
    /// information about the new size of the console screen buffer.
    WindowBufferSizeEvent(WindowBufferSizeRecord),
    /// The Event member contains a `FOCUS_EVENT_RECORD` structure. These
    /// events are used internally and should be ignored.
    FocusEvent(FocusEventRecord),
    /// The Event member contains a `MENU_EVENT_RECORD` structure. These
    /// events are used internally and should be ignored.
    MenuEvent(MenuEventRecord),
}

impl From<INPUT_RECORD> for InputRecord {
    #[inline]
    fn from(record: INPUT_RECORD) -> Self {
        match record.EventType {
            KEY_EVENT => {
                InputRecord::KeyEvent(KeyEventRecord::from(unsafe { record.Event.KeyEvent() }))
            }
            MOUSE_EVENT => InputRecord::MouseEvent(unsafe { *record.Event.MouseEvent() }.into()),
            WINDOW_BUFFER_SIZE_EVENT => InputRecord::WindowBufferSizeEvent(
                unsafe { *record.Event.WindowBufferSizeEvent() }.into(),
            ),
            FOCUS_EVENT => InputRecord::FocusEvent(unsafe { *record.Event.FocusEvent() }.into()),
            MENU_EVENT => InputRecord::MenuEvent(unsafe { *record.Event.MenuEvent() }.into()),
            code => unreachable!("Invalid input record type: {}", code),
        }
    }
}
