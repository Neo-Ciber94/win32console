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

impl Into<FOCUS_EVENT_RECORD> for FocusEventRecord{
    fn into(self) -> FOCUS_EVENT_RECORD {
        FOCUS_EVENT_RECORD{
            bSetFocus: if self.set_focus { 1 } else { 0 }
        }
    }
}

impl From<FOCUS_EVENT_RECORD> for FocusEventRecord {
    #[inline]
    fn from(record: FOCUS_EVENT_RECORD) -> Self {
        FocusEventRecord {
            set_focus: record.bSetFocus != 0,
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn menu_event_into_test(){
        let mut focus_event : FocusEventRecord = unsafe { std::mem::zeroed() };
        focus_event.set_focus = true;
        let raw_focus_event : FOCUS_EVENT_RECORD = focus_event.into();

        assert_eq!(focus_event.set_focus, raw_focus_event.bSetFocus != 0);
    }
}
