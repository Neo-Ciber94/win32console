use crate::structs::input_event::ControlKeyState;
use winapi::um::wincon::CONSOLE_READCONSOLE_CONTROL;

/// Represents a [CONSOLE_READCONSOLE_CONTROL] which contains information for a console read operation.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/console-readconsole-control]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct ConsoleReadControl{
    // Size of this struct in bytes.
    pub size: u32,
    // The number of characters to skip (and thus preserve) before writing newly read input in the buffer passed to the [ReadConsole] function.
    pub initial_chars: u32,
    // A user-defined control character used to signal that the read is complete.
    // See: https://en.wikipedia.org/wiki/Control_character
    pub ctrl_wakeup_mask: u32,
    // The state of the control keys.
    pub control_key_state: ControlKeyState
}

impl ConsoleReadControl{
    /// Creates a new `ConsoleReadControl` with the given values.
    #[inline]
    pub fn new(initial_chars: u32, ctrl_wakeup_mask: u32, control_key_state: ControlKeyState) -> Self{
        ConsoleReadControl{
            size: std::mem::size_of::<ConsoleReadControl>() as u32,
            initial_chars,
            ctrl_wakeup_mask,
            control_key_state
        }
    }

    /// Creates a new `ConsoleReadControl` with the given control mask.
    /// see: [https://en.wikipedia.org/wiki/Control_character]
    ///
    /// # Examples
    /// ```
    /// use win32console::structs::console_read_control::ConsoleReadControl;
    ///
    /// const ESC : u32 = 27;
    /// const CTRL_Z : u32 = 26;
    ///
    /// // A mask that allow escape on `ESC` or `Ctrl+Z` press.
    /// const MASK : u32 = 1 << (ESC | CTRL_Z);
    /// let control = ConsoleReadControl::new_with_mask(MASK);
    /// ```
    #[inline]
    pub fn new_with_mask(ctrl_wakeup_mask: u32) -> Self{
        ConsoleReadControl{
            size: std::mem::size_of::<ConsoleReadControl>() as u32,
            initial_chars: 0,
            ctrl_wakeup_mask,
            control_key_state: ControlKeyState::new(0)
        }
    }
}

impl From<CONSOLE_READCONSOLE_CONTROL> for ConsoleReadControl{
    #[inline]
    fn from(control: CONSOLE_READCONSOLE_CONTROL) -> Self {
        ConsoleReadControl{
            size: control.nLength,
            initial_chars: control.nInitialChars,
            ctrl_wakeup_mask: control.dwCtrlWakeupMask,
            control_key_state: ControlKeyState::new(control.dwControlKeyState)
        }
    }
}

impl Into<CONSOLE_READCONSOLE_CONTROL> for ConsoleReadControl{
    #[inline]
    fn into(self) -> CONSOLE_READCONSOLE_CONTROL {
        CONSOLE_READCONSOLE_CONTROL{
            nLength: std::mem::size_of::<ConsoleReadControl>() as u32,
            nInitialChars: self.initial_chars,
            dwCtrlWakeupMask: self.ctrl_wakeup_mask,
            dwControlKeyState: self.control_key_state.get_state()
        }
    }
}