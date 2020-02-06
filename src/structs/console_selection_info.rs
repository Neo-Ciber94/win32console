use crate::structs::console_screen_buffer_info::SmallRect;
use crate::structs::coord::Coord;
use winapi::um::wincon::{CONSOLE_SELECTION_INFO};

/// Represents a [CONSOLE_SELECTION_INFO] which contains information for a console selection.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/console-selection-info-str]
pub struct ConsoleSelectionInfo {
    // The selection indicator
    pub selection_indicator: SelectionState,
    // Specifies the selection anchor, in characters.
    pub selection_anchor: Coord,
    // Specifies the selection rectangle.
    pub selection_rect: SmallRect,
}

/// Represents the selection indicator state
pub struct SelectionState(u32);

impl SelectionState {
    /// No selection
    pub const CONSOLE_NO_SELECTION: u32 = 0x0000;
    /// Selection has begun
    pub const CONSOLE_SELECTION_IN_PROGRESS: u32 = 0x0001;
    /// Selection rectangle is not empty
    pub const CONSOLE_SELECTION_NOT_EMPTY: u32 = 0x0002;
    /// Selecting with the mouse
    pub const CONSOLE_MOUSE_SELECTION: u32 = 0x0004;
    /// Mouse is down
    pub const CONSOLE_MOUSE_DOWN: u32 = 0x0008;

    /// Gets the state of the selection.
    #[inline]
    pub fn get_state(&self) -> u32 {
        self.0
    }

    /// Checks if the selection have the specified state.
    #[inline]
    pub fn has_state(&self, state: u32) -> bool {
        (self.0 & state) != 0
    }

    /// Checks if there is not selection.
    #[inline]
    pub fn no_selection(&self) -> bool {
        self.0 == SelectionState::CONSOLE_NO_SELECTION
    }

    /// Checks if the selection has begun.
    #[inline]
    pub fn selection_in_progress(&self) -> bool {
        self.has_state(SelectionState::CONSOLE_SELECTION_IN_PROGRESS)
    }

    /// Checks if the selection rect is not empty.
    #[inline]
    pub fn selection_not_empty(&self) -> bool {
        self.has_state(SelectionState::CONSOLE_SELECTION_NOT_EMPTY)
    }

    /// Checks if is selecting with the mouse.
    #[inline]
    pub fn mouse_selection(&self) -> bool {
        self.has_state(SelectionState::CONSOLE_MOUSE_SELECTION)
    }

    /// Checks if the mouse button is down.
    #[inline]
    pub fn is_mouse_down(&self) -> bool {
        self.has_state(SelectionState::CONSOLE_MOUSE_DOWN)
    }
}

impl Into<CONSOLE_SELECTION_INFO> for ConsoleSelectionInfo {
    fn into(self) -> CONSOLE_SELECTION_INFO {
        CONSOLE_SELECTION_INFO {
            dwFlags: self.selection_indicator.0,
            dwSelectionAnchor: self.selection_anchor.into(),
            srSelection: self.selection_rect.into(),
        }
    }
}

impl From<CONSOLE_SELECTION_INFO> for ConsoleSelectionInfo {
    fn from(info: CONSOLE_SELECTION_INFO) -> Self {
        ConsoleSelectionInfo {
            selection_indicator: SelectionState(info.dwFlags),
            selection_anchor: Coord::from(info.dwSelectionAnchor),
            selection_rect: SmallRect::from(info.srSelection),
        }
    }
}
