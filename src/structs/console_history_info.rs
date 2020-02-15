use winapi::um::wincon::{CONSOLE_HISTORY_INFO};

/// Represents a `CONSOLE_HISTORY_INFO` which contains information about the console history.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/console-history-info`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConsoleHistoryInfo{
    /// The size of the structure, in bytes.
    pub size: u32,
    /// The number of commands kept in each history buffer.
    pub history_buffer_size: u32,
    /// The number of history buffers kept for this console process.
    pub number_of_history_buffers: u32,
    /// Determines if duplicate entries will be stored in the history buffer.
    pub allow_duplicate_entries: bool
}

impl Into<CONSOLE_HISTORY_INFO> for ConsoleHistoryInfo{
    #[inline]
    fn into(self) -> CONSOLE_HISTORY_INFO {
        CONSOLE_HISTORY_INFO{
            cbSize: self.size,
            HistoryBufferSize: self.history_buffer_size,
            NumberOfHistoryBuffers: self.number_of_history_buffers,
            dwFlags: self.allow_duplicate_entries.into()
        }
    }
}

impl From<CONSOLE_HISTORY_INFO> for ConsoleHistoryInfo{
    #[inline]
    fn from(info: CONSOLE_HISTORY_INFO) -> Self {
        ConsoleHistoryInfo{
            size: info.HistoryBufferSize,
            history_buffer_size: info.HistoryBufferSize,
            number_of_history_buffers: info.NumberOfHistoryBuffers,
            allow_duplicate_entries: if info.dwFlags == 0 { false } else { true }
        }
    }
}