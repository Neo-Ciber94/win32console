use crate::structs::coord::Coord;
use winapi::um::wincon::WINDOW_BUFFER_SIZE_RECORD;

/// Represents a `WINDOW_BUFFER_SIZE_RECORD` which describes a change in the size of the console screen buffer.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/window-buffer-size-record-str`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WindowBufferSizeRecord {
    /// Contains the size of the console screen buffer, in character cell columns and rows.
    pub size: Coord,
}

impl From<WINDOW_BUFFER_SIZE_RECORD> for WindowBufferSizeRecord {
    #[inline]
    fn from(record: WINDOW_BUFFER_SIZE_RECORD) -> Self {
        WindowBufferSizeRecord {
            size: record.dwSize.into(),
        }
    }
}
