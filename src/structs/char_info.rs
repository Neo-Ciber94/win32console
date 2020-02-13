use winapi::um::wincon::{ CHAR_INFO };
use std::convert::TryFrom;

/// Represents a [CHAR_INFO] which is used by console functions to read from and write to a console screen buffer.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/char-info-str]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CharInfo{
    /// The char value
    pub char_value: char,
    /// The character attributes
    /// link: [https://docs.microsoft.com/en-us/windows/console/char-info-str#members]
    pub attributes: u16
}

impl CharInfo{
    /// Creates a new `CharInfo`.
    #[inline]
    pub fn new(char_value: char, attributes: u16) -> Self {
        CharInfo{
            char_value, attributes
        }
    }
}

impl From<CHAR_INFO> for CharInfo{
    fn from(info: CHAR_INFO) -> Self {
        CharInfo{
            char_value: {
                char::try_from(unsafe { *info.Char.UnicodeChar() } as u32).unwrap()
            },
            attributes: info.Attributes
        }
    }
}

impl Into<CHAR_INFO> for CharInfo{
    fn into(self) -> CHAR_INFO {
        CHAR_INFO{
            Char: {
                let mut buf : [u16; 1] = [0];
                self.char_value.encode_utf16(buf.as_mut());
                unsafe { std::mem::transmute(buf) }
            },
            Attributes: self.attributes
        }
    }
}