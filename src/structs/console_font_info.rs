use crate::structs::coord::Coord;
use winapi::um::wincon::CONSOLE_FONT_INFO;

/// Represents a [CONSOLE_FONT_INFO] which contains information for a console font.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConsoleFontInfo{
    /// The index of the font in the system's console font table.
    pub font_index: u32,
    /// Font size where x is width and y height.
    pub font_size: Coord,
}

impl From<CONSOLE_FONT_INFO> for ConsoleFontInfo{
    #[inline]
    fn from(info: CONSOLE_FONT_INFO) -> Self {
        ConsoleFontInfo{
            font_index: info.nFont,
            font_size: Coord::from(info.dwFontSize)
        }
    }
}

impl Into<CONSOLE_FONT_INFO> for ConsoleFontInfo{
    #[inline]
    fn into(self) -> CONSOLE_FONT_INFO {
        CONSOLE_FONT_INFO{
            nFont: self.font_index,
            dwFontSize: self.font_size.into()
        }
    }
}