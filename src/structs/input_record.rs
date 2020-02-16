use crate::structs::focus_event::FocusEventRecord;
use crate::structs::input_event::{KeyEventRecord, MouseEventRecord};
use crate::structs::menu_event::MenuEventRecord;
use crate::structs::window_buffer_size_event::WindowBufferSizeRecord;
use winapi::um::wincon::{INPUT_RECORD, KEY_EVENT_RECORD, MOUSE_EVENT_RECORD, WINDOW_BUFFER_SIZE_RECORD, MENU_EVENT_RECORD, FOCUS_EVENT_RECORD};
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
    fn from(record: INPUT_RECORD) -> Self {
        match record.EventType {
            KEY_EVENT => {
                InputRecord::KeyEvent(KeyEventRecord::from(unsafe { *record.Event.KeyEvent() }))
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

impl Into<INPUT_RECORD> for InputRecord{
    fn into(self) -> INPUT_RECORD {
        match self {
            InputRecord::KeyEvent(event) => event.into(),
            InputRecord::MouseEvent(event) => event.into(),
            InputRecord::WindowBufferSizeEvent(event) => event.into(),
            InputRecord::FocusEvent(event) => event.into(),
            InputRecord::MenuEvent(event) => event.into(),
        }
    }
}

impl Into<INPUT_RECORD> for KeyEventRecord{
    fn into(self) -> INPUT_RECORD {
        INPUT_RECORD{
            EventType: KEY_EVENT,
            Event: unsafe {
                let event : KEY_EVENT_RECORD = self.into();
                std::mem::transmute_copy(&event)
            }
        }
    }
}

impl Into<INPUT_RECORD> for MouseEventRecord{
    fn into(self) -> INPUT_RECORD {
        INPUT_RECORD{
            EventType: MOUSE_EVENT,
            Event: unsafe {
                let event : MOUSE_EVENT_RECORD = self.into();
                std::mem::transmute_copy(&event)
            }
        }
    }
}

impl Into<INPUT_RECORD> for WindowBufferSizeRecord{
    fn into(self) -> INPUT_RECORD {
        INPUT_RECORD{
            EventType: WINDOW_BUFFER_SIZE_EVENT,
            Event: unsafe {
                let event : WINDOW_BUFFER_SIZE_RECORD = self.into();
                std::mem::transmute_copy(&event)
            }
        }
    }
}

impl Into<INPUT_RECORD> for MenuEventRecord{
    fn into(self) -> INPUT_RECORD {
        INPUT_RECORD{
            EventType: MENU_EVENT,
            Event: unsafe {
                let event : MENU_EVENT_RECORD = self.into();
                std::mem::transmute_copy(&event)
            }
        }
    }
}

impl Into<INPUT_RECORD> for FocusEventRecord{
    fn into(self) -> INPUT_RECORD {
        INPUT_RECORD{
            EventType: FOCUS_EVENT,
            Event: unsafe {
                let event : FOCUS_EVENT_RECORD = self.into();
                std::mem::transmute_copy(&event)
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::structs::input_event::ControlKeyState;

    #[test]
    fn key_event_into_input_record_test(){
        let mut key_event : KeyEventRecord = unsafe { std::mem::zeroed() };
        key_event.control_key_state = ControlKeyState::new(1);
        key_event.u_char = 'a';
        key_event.key_down = true;
        key_event.virtual_key_code = 2;
        key_event.repeat_count = 4;
        key_event.virtual_scan_code = 8;

        let record : INPUT_RECORD = key_event.into();
        assert_eq!(record.EventType, KEY_EVENT);

        let from_record = KeyEventRecord::from(unsafe { *record.Event.KeyEvent() });

        assert_eq!(key_event.virtual_scan_code, from_record.virtual_scan_code);
        assert_eq!(key_event.repeat_count, from_record.repeat_count);
        assert_eq!(key_event.key_down, from_record.key_down);
        assert_eq!(key_event.u_char, from_record.u_char);
        assert_eq!(key_event.control_key_state, from_record.control_key_state);
    }
}