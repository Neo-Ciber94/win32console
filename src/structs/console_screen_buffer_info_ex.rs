use crate::structs::coord::Coord;
use crate::structs::console_screen_buffer_info::SmallRect;
use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFOEX;

pub struct ConsoleScreenBufferInfoEx{
    pub size: u32,
    pub screen_buffer_size: Coord,
    pub cursor_position: Coord,
    pub attributes: u16,
    pub window: SmallRect,
    pub window_maximum_size: Coord,
    pub popup_attributes: u16,
    pub full_screen_supported: bool,
    pub color_ref: [u32; 16],
}

impl From<CONSOLE_SCREEN_BUFFER_INFOEX> for ConsoleScreenBufferInfoEx{
    fn from(info: CONSOLE_SCREEN_BUFFER_INFOEX) -> Self {
        ConsoleScreenBufferInfoEx{
            size: info.cbSize,
            screen_buffer_size: Coord::from(info.dwSize),
            cursor_position: Coord::from(info.dwCursorPosition),
            attributes: info.wAttributes,
            window: SmallRect::from(info.srWindow),
            window_maximum_size: Coord::from(info.dwMaximumWindowSize),
            popup_attributes: info.wPopupAttributes,
            full_screen_supported: info.bFullscreenSupported != 0,
            color_ref: info.ColorTable
        }
    }
}

impl Into<CONSOLE_SCREEN_BUFFER_INFOEX> for ConsoleScreenBufferInfoEx{
    fn into(self) -> CONSOLE_SCREEN_BUFFER_INFOEX {
        CONSOLE_SCREEN_BUFFER_INFOEX{
            cbSize: self.size,
            dwSize: self.screen_buffer_size.into(),
            dwCursorPosition: self.cursor_position.into(),
            wAttributes: self.attributes,
            srWindow: self.window.into(),
            dwMaximumWindowSize: self.window_maximum_size.into(),
            wPopupAttributes: self.popup_attributes,
            bFullscreenSupported: self.full_screen_supported as i32,
            ColorTable: self.color_ref
        }
    }
}