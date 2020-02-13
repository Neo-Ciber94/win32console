use crate::structs::coord::Coord;
use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO;
use crate::structs::small_rect::SmallRect;

/// Represents a [CONSOLE_SCREEN_BUFFER_INFO], which contains information about the
/// console screen buffer.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/console-screen-buffer-info-str]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConsoleScreenBufferInfo {
    /// Size of the screen buffer in rows and columns.
    pub screen_buffer_size: Coord,
    /// Position of the cursor in the console screen buffer.
    pub cursor_position: Coord,
    /// The attributes of the characters written in the console screen buffer.
    pub attributes: u16,
    /// The rect that contains the console screen buffer.
    pub window: SmallRect,
    /// The maximum size the console window.
    pub maximum_window_size: Coord,
}

impl From<CONSOLE_SCREEN_BUFFER_INFO> for ConsoleScreenBufferInfo {
    #[inline]
    fn from(info: CONSOLE_SCREEN_BUFFER_INFO) -> Self {
        ConsoleScreenBufferInfo {
            screen_buffer_size: Coord::from(info.dwSize),
            cursor_position: Coord::from(info.dwCursorPosition),
            attributes: info.wAttributes,
            window: SmallRect::from(info.srWindow),
            maximum_window_size: Coord::from(info.dwMaximumWindowSize),
        }
    }
}

impl Into<CONSOLE_SCREEN_BUFFER_INFO> for ConsoleScreenBufferInfo {
    #[inline]
    fn into(self) -> CONSOLE_SCREEN_BUFFER_INFO {
        CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: self.screen_buffer_size.into(),
            dwCursorPosition: self.cursor_position.into(),
            wAttributes: self.attributes,
            srWindow: self.window.into(),
            dwMaximumWindowSize: self.maximum_window_size.into(),
        }
    }
}