use crate::structs::coord::Coord;
use winapi::shared::minwindef::DWORD;
use winapi::um::wincon::{
    FROM_LEFT_1ST_BUTTON_PRESSED, FROM_LEFT_2ND_BUTTON_PRESSED, FROM_LEFT_3RD_BUTTON_PRESSED,
    FROM_LEFT_4TH_BUTTON_PRESSED, KEY_EVENT_RECORD, MOUSE_EVENT_RECORD, RIGHTMOST_BUTTON_PRESSED,
};
use std::convert::TryFrom;

/// Represents a [KEY_EVENT_RECORD] which describes a keyboard input event
/// in a console [INPUT_RECORD] structure.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/key-event-record-str]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct KeyEventRecord {
    /// If the key is pressed, this member is TRUE. Otherwise, this member is
    /// FALSE (the key is released).
    pub key_down: bool,
    /// The repeat count, which indicates that a key is being held down.
    /// For example, when a key is held down, you might get five events with
    /// this member equal to 1, one event with this member equal to 5, or
    /// multiple events with this member greater than or equal to 1.
    pub repeat_count: u16,
    /// A virtual-key code that identifies the given key in a
    /// device-independent manner.
    pub virtual_key_code: u16,
    /// The virtual scan code of the given key that represents the
    /// device-dependent value generated by the keyboard hardware.
    pub virtual_scan_code: u16,
    /// The translated Unicode character (as a WCHAR, or utf-16 value)
    pub u_char: char,
    /// The state of the control keys.
    pub control_key_state: ControlKeyState,
}

/// Represents a [MOUSE_EVENT_RECORD] which describes a mouse input event
/// in a console [INPUT_RECORD] structure.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/mouse-event-record-str]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MouseEventRecord {
    /// Contains the location of the cursor, in terms of the console screen buffer's character-cell coordinates.
    pub mouse_position: Coord,
    /// The status of the mouse buttons.
    /// The least significant bit corresponds to the leftmost mouse button. T
    /// he next least significant bit corresponds to the rightmost mouse button.
    /// The next bit indicates the next-to-leftmost mouse button. The bits then correspond left to right to the mouse buttons.
    /// A bit is 1 if the button was pressed.
    pub button_state: ButtonState,
    /// The state of the control keys.
    pub control_key_state: ControlKeyState,
    /// The type of mouse event.
    /// If this value is zero, it indicates a mouse button being pressed or released.
    pub event_flags: EventFlags,
}

/// Represents the state of the mouse buttons.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/mouse-event-record-str#members]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ButtonState(i32);

/// Represents the state of the control keys.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/mouse-event-record-str#members]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ControlKeyState(u32);

/// Represents the type of mouse event.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/mouse-event-record-str#members]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventFlags {
    /// The button is being pressed or released.
    PressOrRelease = 0x0000,
    /// If the high word of the dwButtonState member contains a positive value, the wheel was rotated to the right.
    /// Otherwise, the wheel was rotated to the left.
    MouseMoved = 0x0001,
    /// The second click (button press) of a double-click occurred.
    /// The first click is returned as a regular button-press event.
    DoubleClick = 0x0002,
    /// A change in mouse position occurred.
    /// The vertical mouse wheel was moved,
    /// if the high word of the dwButtonState member contains a positive value,
    /// the wheel was rotated forward, away from the user.
    /// Otherwise, the wheel was rotated backward, toward the user.
    MouseWheeled = 0x0004,
    /// The horizontal mouse wheel was moved.
    MouseHwheeled = 0x0008,
}

impl ControlKeyState {
    /// The right ALT key is pressed.
    pub const RIGHT_ALT_PRESSED: u32 = 0x0001;
    /// The left ALT key is pressed.
    pub const LEFT_ALT_PRESSED: u32 = 0x0002;
    /// The right CTRL key is pressed.
    pub const RIGHT_CTRL_PRESSED: u32 = 0x0004;
    /// The left CTRL key is pressed.
    pub const LEFT_CTRL_PRESSED: u32 = 0x0008;
    /// The SHIFT key is pressed.
    pub const SHIFT_PRESSED: u32 = 0x0010;
    /// The NUM LOCK light is on.
    pub const NUM_LOCK_ON: u32 = 0x0020;
    /// The SCROLL LOCK light is on.
    pub const SCROLL_LOCK_ON: u32 = 0x0040;
    /// The CAPS LOCK light is on.
    pub const CAPS_LOCK_ON: u32 = 0x0080;
    /// The key is enhanced.
    pub const ENHANCED_KEY: u32 = 0x0100;

    /// Checks whether this state contains the specified.
    ///
    /// # Example
    ///
    /// Basic usages:
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    /// use win32console::structs::input_event::ControlKeyState;
    /// let input = WinConsole::output().read_single_input().unwrap();
    ///
    /// match input{
    ///     KeyEvent(e) => {
    ///         if e.control_key_state.has_state(ControlKeyState::CAPS_LOCK_ON){
    ///             println!("{}", "CapsLock is on")
    ///         }
    ///         else{
    ///             println!("{}", "CapsLock not on")
    ///         }
    ///     }
    ///     _ => {}
    /// }
    /// ```
    #[inline]
    pub fn has_state(&self, state: u32) -> bool {
        (state & self.0) != 0
    }

    #[inline]
    pub fn get_state(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn is_alt_pressed(&self) -> bool {
        self.has_state(ControlKeyState::RIGHT_ALT_PRESSED)
            || self.has_state(ControlKeyState::LEFT_ALT_PRESSED)
    }

    #[inline]
    pub fn is_ctrl_pressed(&self) -> bool {
        self.has_state(ControlKeyState::RIGHT_CTRL_PRESSED)
            || self.has_state(ControlKeyState::LEFT_CTRL_PRESSED)
    }

    #[inline]
    pub fn is_shift_pressed(&self) -> bool {
        self.has_state(ControlKeyState::SHIFT_PRESSED)
    }

    #[inline]
    pub fn is_num_lock_on(&self) -> bool {
        self.has_state(ControlKeyState::NUM_LOCK_ON)
    }

    #[inline]
    pub fn is_caps_lock_on(&self) -> bool {
        self.has_state(ControlKeyState::CAPS_LOCK_ON)
    }

    #[inline]
    pub fn is_scroll_lock_on(&self) -> bool {
        self.has_state(ControlKeyState::SCROLL_LOCK_ON)
    }

    #[inline]
    pub fn is_enhanced_key(&self) -> bool {
        self.has_state(ControlKeyState::ENHANCED_KEY)
    }
}

impl ButtonState {
    /// Returns whether the button is released or pressed.
    pub fn release_button(&self) -> bool {
        self.0 == 0
    }

    /// Returns whether the left button was pressed.
    pub fn left_button(&self) -> bool {
        self.0 as u32 & FROM_LEFT_1ST_BUTTON_PRESSED != 0
    }

    /// Returns whether the right button was pressed.
    pub fn right_button(&self) -> bool {
        self.0 as u32
            & (RIGHTMOST_BUTTON_PRESSED
                | FROM_LEFT_3RD_BUTTON_PRESSED
                | FROM_LEFT_4TH_BUTTON_PRESSED)
            != 0
    }

    /// Returns whether the right button was pressed.
    pub fn middle_button(&self) -> bool {
        self.0 as u32 & FROM_LEFT_2ND_BUTTON_PRESSED != 0
    }

    /// Returns whether there is a down scroll.
    pub fn scroll_down(&self) -> bool {
        self.0 < 0
    }

    /// Returns whether there is a up scroll.
    pub fn scroll_up(&self) -> bool {
        self.0 > 0
    }

    /// Returns the raw state.
    pub fn get_state(&self) -> i32 {
        self.0
    }
}

impl From<&KEY_EVENT_RECORD> for KeyEventRecord {
    #[inline]
    fn from(record: &KEY_EVENT_RECORD) -> Self {
        KeyEventRecord {
            key_down: record.bKeyDown != 0,
            repeat_count: record.wRepeatCount,
            virtual_key_code: record.wVirtualKeyCode,
            virtual_scan_code: record.wVirtualScanCode,
            u_char: unsafe{ char::try_from(*record.uChar.UnicodeChar() as u32).ok().unwrap() },
            control_key_state: ControlKeyState(record.dwControlKeyState),
        }
    }
}

impl From<u32> for EventFlags {
    fn from(event: u32) -> Self {
        match event {
            0x0000 => EventFlags::PressOrRelease,
            0x0001 => EventFlags::MouseMoved,
            0x0002 => EventFlags::DoubleClick,
            0x0004 => EventFlags::MouseWheeled,
            0x0008 => EventFlags::MouseHwheeled,
            _ => panic!("Event flag {} does not exist.", event),
        }
    }
}

impl From<MOUSE_EVENT_RECORD> for MouseEventRecord {
    #[inline]
    fn from(event: MOUSE_EVENT_RECORD) -> Self {
        MouseEventRecord {
            mouse_position: event.dwMousePosition.into(),
            button_state: event.dwButtonState.into(),
            control_key_state: ControlKeyState(event.dwControlKeyState),
            event_flags: event.dwEventFlags.into(),
        }
    }
}

impl From<u32> for ButtonState {
    #[inline]
    fn from(state: u32) -> Self {
        ButtonState(state as i32)
    }
}