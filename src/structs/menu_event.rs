use winapi::um::wincon::MENU_EVENT_RECORD;

/// Represents a `MENU_EVENT_RECORD` which describes a menu event in a console `INPUT_RECORD` structure.
/// These events are used internally and should be ignored.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/menu-event-record-str`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MenuEventRecord {
    /// Reserved.
    pub command_id: u32,
}

impl From<MENU_EVENT_RECORD> for MenuEventRecord {
    #[inline]
    fn from(record: MENU_EVENT_RECORD) -> Self {
        MenuEventRecord {
            command_id: record.dwCommandId,
        }
    }
}
