use winapi::um::wingdi::LF_FACESIZE;
use winapi::um::wincon::CONSOLE_FONT_INFOEX;
use crate::structs::coord::Coord;

/// Represents a [CONSOLE_FONT_INFOEX]
///
/// link: [https://docs.microsoft.com/en-us/windows/console/console-font-infoex]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConsoleFontInfoEx{
    pub size: u32,
    pub font: u32,
    pub font_size: Coord,
    pub font_family: u32,
    pub font_weight: u32,
    pub face_name: [u16; LF_FACESIZE],
}

impl From<&CONSOLE_FONT_INFOEX> for ConsoleFontInfoEx{
    fn from(info: &CONSOLE_FONT_INFOEX) -> Self {
        ConsoleFontInfoEx{
            size: info.cbSize,
            font: info.nFont,
            font_size: Coord::from(info.dwFontSize),
            font_family: info.FontFamily,
            font_weight: info.FontWeight,
            face_name: info.FaceName
        }
    }
}

impl Into<CONSOLE_FONT_INFOEX> for ConsoleFontInfoEx{
    fn into(self) -> CONSOLE_FONT_INFOEX {
        CONSOLE_FONT_INFOEX{
            cbSize: self.size,
            nFont: self.font,
            dwFontSize: self.font_size.into(),
            FontFamily: self.font_family,
            FontWeight: self.font_weight,
            FaceName: self.face_name
        }
    }
}