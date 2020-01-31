use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO;
use winapi::um::wincontypes::SMALL_RECT;
use crate::structs::coord::Coord;

pub struct ConsoleScreenBufferInfo{
    pub size: Coord,
    pub cursor_position: Coord,
    pub attributes: u16,
    pub small_rect: SmallRect,
    pub maximum_window_size: Coord
}

pub struct SmallRect{
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16
}

impl From<CONSOLE_SCREEN_BUFFER_INFO> for ConsoleScreenBufferInfo{
    fn from(info: CONSOLE_SCREEN_BUFFER_INFO) -> Self {
        ConsoleScreenBufferInfo{
            size: Coord::from(info.dwSize),
            cursor_position: Coord::from(info.dwCursorPosition),
            attributes: info.wAttributes,
            small_rect: SmallRect::from(info.srWindow),
            maximum_window_size: Coord::from(info.dwMaximumWindowSize)
        }
    }
}

impl From<SMALL_RECT> for SmallRect{
    fn from(rect: SMALL_RECT) -> Self {
        SmallRect{
            left: rect.Left,
            top: rect.Top,
            right: rect.Right,
            bottom: rect.Bottom
        }
    }
}

impl Into<SMALL_RECT> for SmallRect{
    fn into(self) -> SMALL_RECT {
        SMALL_RECT{
            Left: self.left,
            Top: self.top,
            Right: self.right,
            Bottom: self.bottom
        }
    }
}