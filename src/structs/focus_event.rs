use winapi::um::wincon::FOCUS_EVENT_RECORD;

/// Represents a `FOCUS_EVENT_RECORD` which Describes a focus event in a console `INPUT_RECORD` structure.
/// These events are used internally and should be ignored.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/focus-event-record-str`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct FocusEventRecord {
    /// Reserved.
    pub set_focus: bool,
}

impl From<FOCUS_EVENT_RECORD> for FocusEventRecord {
    #[inline]
    fn from(record: FOCUS_EVENT_RECORD) -> Self {
        FocusEventRecord {
            set_focus: record.bSetFocus != 0,
        }
    }
}
