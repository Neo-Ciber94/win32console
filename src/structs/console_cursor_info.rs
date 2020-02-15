use winapi::um::wincon::CONSOLE_CURSOR_INFO;

/// Represents a `CONSOLE_CURSOR_INFO` which contains information about the console cursor.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/console-cursor-info-str`
pub struct ConsoleCursorInfo{
    /// The percentage of the character cell that is filled by the cursor.
    /// This value is between 1 and 100. The cursor appearance varies,
    /// ranging from completely filling the cell to showing up as a horizontal line at the bottom of the cell.
    pub size: u32,
    /// The visibility of the cursor.
    pub visible: bool
}

impl Into<CONSOLE_CURSOR_INFO> for ConsoleCursorInfo{
    #[inline]
    fn into(self) -> CONSOLE_CURSOR_INFO {
        CONSOLE_CURSOR_INFO{
            dwSize: self.size,
            bVisible: self.visible.into()
        }
    }
}

impl From<CONSOLE_CURSOR_INFO> for ConsoleCursorInfo{
    #[inline]
    fn from(info: CONSOLE_CURSOR_INFO) -> Self {
        ConsoleCursorInfo{
            size: info.dwSize,
            visible: if info.bVisible == 0 { false } else { true }
        }
    }
}