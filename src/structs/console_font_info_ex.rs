use crate::structs::coord::Coord;
use winapi::um::wincon::CONSOLE_FONT_INFOEX;
use winapi::um::wingdi::LF_FACESIZE;

/// Represents a [CONSOLE_FONT_INFOEX] which contains extended information about the console font.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/console-font-infoex]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConsoleFontInfoEx {
    /// Size this struct in bytes.
    ///
    /// The size can be obtained using:
    /// ```
    /// use win32console::structs::console_font_info_ex::*;
    /// let size = std::mem::size_of::<ConsoleFontInfoEx>();
    /// ```
    pub size: u32,
    /// The index of the font in the system's console font table.
    pub font_index: u32,
    /// Font size where x is width and y height.
    pub font_size: Coord,
    /// The font pitch and family
    pub font_family: u32,
    /// The font weight, in a range from 100 to 1000 in multiples of 100.
    pub font_weight: u32,
    /// The name of the typeface (such as Courier or Arial).
    pub face_name: [u16; LF_FACESIZE],
}

impl From<&CONSOLE_FONT_INFOEX> for ConsoleFontInfoEx {
    #[inline]
    fn from(info: &CONSOLE_FONT_INFOEX) -> Self {
        ConsoleFontInfoEx {
            size: info.cbSize,
            font_index: info.nFont,
            font_size: Coord::from(info.dwFontSize),
            font_family: info.FontFamily,
            font_weight: info.FontWeight,
            face_name: info.FaceName,
        }
    }
}

impl Into<CONSOLE_FONT_INFOEX> for ConsoleFontInfoEx {
    #[inline]
    fn into(self) -> CONSOLE_FONT_INFOEX {
        CONSOLE_FONT_INFOEX {
            cbSize: self.size,
            nFont: self.font_index,
            dwFontSize: self.font_size.into(),
            FontFamily: self.font_family,
            FontWeight: self.font_weight,
            FaceName: self.face_name,
        }
    }
}
