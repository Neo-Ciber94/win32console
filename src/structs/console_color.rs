use crate::console::WinConsole;
use std::convert::TryFrom;
use std::fmt::{Display, Error, Formatter, Write};
use winapi::_core::convert::TryInto;

/// Represents a color for the windows console.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConsoleColor {
    Black = 0,
    DarkBlue = 1,
    DarkGreen = 2,
    DarkCyan = 3,
    DarkRed = 4,
    DarkMagenta = 5,
    DarkYellow = 6,
    Gray = 7,
    DarkGray = 8,
    Blue = 9,
    Green = 10,
    Cyan = 11,
    Red = 12,
    Magenta = 13,
    Yellow = 14,
    White = 15,
}

impl ConsoleColor {
    /// Gets the `ConsoleTextAttribute` representation of this as foreground color.
    ///
    /// # Example
    /// ```
    /// use win32console::structs::console_color::ConsoleColor;
    /// use win32console::console::WinConsole;
    ///
    /// let color = ConsoleColor::Red.as_foreground_color();
    /// WinConsole::output().set_text_attribute(color);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// ```
    pub fn as_foreground_color(&self) -> u16 {
        *self as u16
    }

    /// Gets the `ConsoleTextAttribute` representation of this as background color.
    ///
    /// # Example
    /// ```
    /// use win32console::structs::console_color::ConsoleColor;
    /// use win32console::console::WinConsole;
    ///
    /// let color = ConsoleColor::Red.as_background_color();
    /// WinConsole::output().set_text_attribute(color);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// ```
    pub fn as_background_color(&self) -> u16 {
        (*self as u16) << 4
    }
}

/// Represents an error when parsing a color, and contains the invalid `ConsoleTextAttribute` value.
pub struct ParseColorError(u16);

impl TryFrom<u16> for ConsoleColor{
    type Error = ParseColorError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ConsoleColor::Black),
            1 => Ok(ConsoleColor::DarkBlue),
            2 => Ok(ConsoleColor::DarkGreen),
            3 => Ok(ConsoleColor::DarkCyan),
            4 => Ok(ConsoleColor::DarkRed),
            5 => Ok(ConsoleColor::DarkMagenta),
            6 => Ok(ConsoleColor::DarkYellow),
            7 => Ok(ConsoleColor::Gray),
            8 => Ok(ConsoleColor::DarkGray),
            9 => Ok(ConsoleColor::Blue),
            10 => Ok(ConsoleColor::Green),
            11 => Ok(ConsoleColor::Cyan),
            12 => Ok(ConsoleColor::Red),
            13 => Ok(ConsoleColor::Magenta),
            14 => Ok(ConsoleColor::Yellow),
            15 => Ok(ConsoleColor::White),
            _ => Err(ParseColorError(value))
        }
    }
}

impl Into<u16> for ConsoleColor {
    fn into(self) -> u16 {
        self.as_foreground_color()
    }
}

impl Display for ConsoleColor{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ConsoleColor::Black => { f.write_str("ConsoleColor::Black") },
            ConsoleColor::DarkBlue => { f.write_str("ConsoleColor::DarkBlue") },
            ConsoleColor::DarkGreen => { f.write_str("ConsoleColor::DarkGreen") },
            ConsoleColor::DarkCyan => { f.write_str("ConsoleColor::DarkCyan") },
            ConsoleColor::DarkRed => { f.write_str("ConsoleColor::DarkRed") },
            ConsoleColor::DarkMagenta => { f.write_str("ConsoleColor::DarkMagenta") },
            ConsoleColor::DarkYellow => { f.write_str("ConsoleColor::DarkYellow") },
            ConsoleColor::Gray => { f.write_str("ConsoleColor::Gray") },
            ConsoleColor::DarkGray => { f.write_str("ConsoleColor::DarkGray") },
            ConsoleColor::Blue => { f.write_str("ConsoleColor::Blue") },
            ConsoleColor::Green => { f.write_str("ConsoleColor::Green") },
            ConsoleColor::Cyan => { f.write_str("ConsoleColor::Cyan") },
            ConsoleColor::Red => { f.write_str("ConsoleColor::Red") },
            ConsoleColor::Magenta => { f.write_str("ConsoleColor::Magenta") },
            ConsoleColor::Yellow => { f.write_str("ConsoleColor::Yellow") },
            ConsoleColor::White => { f.write_str("ConsoleColor::White") },
        }
    }
}