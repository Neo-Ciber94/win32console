use crate::structs::coord::Coord;
use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFOEX;
use crate::structs::small_rect::SmallRect;

/// Represents a `CONSOLE_SCREEN_BUFFER_INFOEX` which contains extended information about
/// the console screen buffer.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/console-screen-buffer-infoex`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConsoleScreenBufferInfoEx {
    /// Size this struct in bytes.
    ///
    /// The size can eb obtained using:
    /// ```
    /// use win32console::structs::console_screen_buffer_info_ex::*;
    /// let size = std::mem::size_of::<ConsoleScreenBufferInfoEx>();
    /// ```
    pub size: u32,
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
    /// The fill attribute for console pop-ups.
    pub popup_attributes: u16,
    /// If `true` the full-screen mode is supported; otherwise is not supported.
    pub full_screen_supported: bool,
    /// An array of `COLORREF` values that describe the console's color settings.
    ///
    /// link: `https://docs.microsoft.com/en-us/windows/win32/gdi/colorref`
    pub color_table: [u32; 16],
}

impl From<CONSOLE_SCREEN_BUFFER_INFOEX> for ConsoleScreenBufferInfoEx {
    #[inline]
    fn from(info: CONSOLE_SCREEN_BUFFER_INFOEX) -> Self {
        ConsoleScreenBufferInfoEx {
            size: info.cbSize,
            screen_buffer_size: Coord::from(info.dwSize),
            cursor_position: Coord::from(info.dwCursorPosition),
            attributes: info.wAttributes,
            window: SmallRect::from(info.srWindow),
            maximum_window_size: Coord::from(info.dwMaximumWindowSize),
            popup_attributes: info.wPopupAttributes,
            full_screen_supported: info.bFullscreenSupported != 0,
            color_table: info.ColorTable,
        }
    }
}

impl Into<CONSOLE_SCREEN_BUFFER_INFOEX> for ConsoleScreenBufferInfoEx {
    #[inline]
    fn into(self) -> CONSOLE_SCREEN_BUFFER_INFOEX {
        CONSOLE_SCREEN_BUFFER_INFOEX {
            cbSize: self.size,
            dwSize: self.screen_buffer_size.into(),
            dwCursorPosition: self.cursor_position.into(),
            wAttributes: self.attributes,
            srWindow: self.window.into(),
            dwMaximumWindowSize: self.maximum_window_size.into(),
            wPopupAttributes: self.popup_attributes,
            bFullscreenSupported: self.full_screen_supported as i32,
            ColorTable: self.color_table,
        }
    }
}
