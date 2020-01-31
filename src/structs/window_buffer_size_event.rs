use winapi::um::wincon::WINDOW_BUFFER_SIZE_RECORD;
use crate::structs::coord::Coord;

/// Represents a [WINDOW_BUFFER_SIZE_RECORD] which describes a change in the size of the console screen buffer.
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
