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

impl Into<MENU_EVENT_RECORD> for MenuEventRecord{
    fn into(self) -> MENU_EVENT_RECORD {
        MENU_EVENT_RECORD{
            dwCommandId: self.command_id
        }
    }
}

impl From<MENU_EVENT_RECORD> for MenuEventRecord {
    #[inline]
    fn from(record: MENU_EVENT_RECORD) -> Self {
        MenuEventRecord {
            command_id: record.dwCommandId,
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn menu_event_into_test(){
        let menu_event : MenuEventRecord = unsafe { std::mem::zeroed() };
        let raw_menu_event : MENU_EVENT_RECORD = menu_event.into();

        assert_eq!(menu_event.command_id, raw_menu_event.dwCommandId);
    }
}