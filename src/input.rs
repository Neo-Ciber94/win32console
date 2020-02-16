//! Includes the basic input structs.
pub use crate::structs::{
    coord::Coord,
    input_event::ButtonState,
    input_event::ControlKeyState,
    input_event::EventFlags,
    input_event::KeyEventRecord,
    input_event::MouseEventRecord,
    input_record::InputRecord,
    input_record::InputRecord::KeyEvent,
    input_record::InputRecord::MouseEvent,
    input_record::InputRecord::WindowBufferSizeEvent,
    input_record::InputRecord::FocusEvent,
    input_record::InputRecord::MenuEvent
};
