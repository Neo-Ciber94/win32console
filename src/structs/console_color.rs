use crate::console::WinConsole;
use std::fmt::{Display, Error, Write, Formatter};
use std::convert::TryFrom;

/// Represents a color for the windows console.
//#[derive(Debug, Copy, Clone, Eq, PartialEq)]
//struct ConsoleColor;
//
//impl ConsoleColor{
//    pub const BLACK: u16 = 0;
//    pub const DARK_BLUE: u16 = 1;
//    pub const DARK_GREEN: u16 = 2;
//    pub const DARK_CYAN: u16 = 3;
//    pub const DARK_RED: u16 = 4;
//    pub const DARK_MAGENTA: u16 = 5;
//    pub const DARK_YELLOW: u16 = 6;
//    pub const GRAY: u16 = 7;
//    pub const DARK_GRAY: u16 = 8;
//    pub const BLUE: u16 = 9;
//    pub const GREEN: u16 = 10;
//    pub const CYAN: u16 = 11;
//    pub const RED: u16 = 12;
//    pub const MAGENTA: u16 = 13;
//    pub const YELLOW: u16 = 14;
//    pub const WHITE: u16 = 15;
//}

/// Represents a color for the windows console.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConsoleColor{
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
    White = 15
}

pub struct ColorParseError(u16);

impl ConsoleColor{
    pub fn as_foreground_color(&self) -> u16{
        *self as u16
    }

    pub fn as_background_color(&self) -> u16{
        (*self as u16) << 4
    }
}

impl From<u16> for ConsoleColor{
    fn from(value: u16) -> Self {
        match value{
            0 => ConsoleColor::Black,
            1 => ConsoleColor::DarkBlue,
            2 => ConsoleColor::DarkGreen,
            3 => ConsoleColor::DarkCyan,
            4 => ConsoleColor::DarkRed,
            5 => ConsoleColor::DarkMagenta,
            6 => ConsoleColor::DarkYellow,
            7 => ConsoleColor::Gray,
            8 => ConsoleColor::DarkGray,
            9 => ConsoleColor::Blue,
            10 => ConsoleColor::Green,
            11 => ConsoleColor::Cyan,
            12 => ConsoleColor::Red,
            13 => ConsoleColor::Magenta,
            14 => ConsoleColor::Yellow,
            15 => ConsoleColor::White,
            _ => panic!(format!("Invalid value: {}", value))
        }
    }
}

impl Into<u16> for ConsoleColor{
    fn into(self) -> u16 {
        self.as_foreground_color()
    }
}

const FG_COLOR_MARK : u16 = 0xF;
const BG_COLOR_MASK : u16 = 0xF0;

impl WinConsole{
    pub fn get_foreground_color(&self) -> std::io::Result<ConsoleColor>{
        let attributes = self.get_text_attribute()?;
        Ok(ConsoleColor::from(attributes & FG_COLOR_MARK))
    }

    pub fn get_background_color(&self) -> std::io::Result<ConsoleColor>{
        let attributes = self.get_text_attribute()?;
        let value = attributes << 4;
        Ok(ConsoleColor::from(value))
    }

    pub fn set_foreground_color(&self, color: ConsoleColor) -> std::io::Result<()>{
        let old_attributes = self.get_text_attribute()?;
        let new_attributes = (old_attributes & !(old_attributes & FG_COLOR_MARK)) | color.as_foreground_color();
        self.set_text_attribute(new_attributes)
    }

    pub fn set_background_color(&self, color: ConsoleColor) -> std::io::Result<()>{
        let old_attributes = self.get_text_attribute()?;
        let new_attributes = (old_attributes & !(old_attributes & BG_COLOR_MASK)) | color.as_background_color();
        self.set_text_attribute(new_attributes)
    }
}