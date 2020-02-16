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

impl Into<WINDOW_BUFFER_SIZE_RECORD> for WindowBufferSizeRecord{
    fn into(self) -> WINDOW_BUFFER_SIZE_RECORD {
        WINDOW_BUFFER_SIZE_RECORD{
            dwSize: self.size.into()
        }
    }
}

impl From<WINDOW_BUFFER_SIZE_RECORD> for WindowBufferSizeRecord {
    #[inline]
    fn from(record: WINDOW_BUFFER_SIZE_RECORD) -> Self {
        WindowBufferSizeRecord {
            size: record.dwSize.into(),
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn window_buffer_size_into_test(){
        let mut window_event : WindowBufferSizeRecord= unsafe { std::mem::zeroed() };
        window_event.size = Coord::new(3, 4);
        let raw_window_event : WINDOW_BUFFER_SIZE_RECORD = window_event.into();

        assert_eq!(window_event.size, Coord::from(raw_window_event.dwSize));
    }
}
