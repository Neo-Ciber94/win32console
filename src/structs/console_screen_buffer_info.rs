use crate::structs::coord::Coord;
use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO;
use winapi::um::wincontypes::{SMALL_RECT };

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

/// Represents a [https://docs.microsoft.com/en-us/windows/console/small-rect-str].
///
/// link: [https://docs.microsoft.com/en-us/windows/console/small-rect-str]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SmallRect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

impl SmallRect {
    /// Creates a new `SmallRect`.
    pub fn new(left: i16, top: i16, right: i16, bottom: i16) -> Self{
        SmallRect{ left, top, right, bottom}
    }
}

impl From<CONSOLE_SCREEN_BUFFER_INFO> for ConsoleScreenBufferInfo {
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

impl From<SMALL_RECT> for SmallRect {
    fn from(rect: SMALL_RECT) -> Self {
        SmallRect {
            left: rect.Left,
            top: rect.Top,
            right: rect.Right,
            bottom: rect.Bottom,
        }
    }
}

impl Into<SMALL_RECT> for SmallRect {
    fn into(self) -> SMALL_RECT {
        SMALL_RECT {
            Left: self.left,
            Top: self.top,
            Right: self.right,
            Bottom: self.bottom,
        }
    }
}