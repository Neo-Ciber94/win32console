//! Provides a mechanism for interact with the windows console
//!
//! This library implements most of the methods in:
//!
//! `https://docs.microsoft.com/en-us/windows/console/console-functions`
//!
//! # Example
//! ```rust
//! use win32console::console::WinConsole;
//! use win32console::input::*;
//!
//! fn main() {
//!     // Virtual key codes
//!     // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
//!     const ESCAPE : u16 = 0x1B;
//!     const BACKSPACE: u16 = 0x08;
//!     const ENTER : u16 = 0x0D;
//!     const SPACE : u16 = 0x20;
//!
//!     loop{
//!         // Get the current input event
//!         if let KeyEvent(key) = WinConsole::input().read_single_input().unwrap(){
//!             // Only check for key down events
//!             if key.key_down{
//!                 let char_value = key.u_char;
//!                 // Write only if is alphanumeric or punctuation
//!                 if char_value.is_ascii_alphanumeric() || char_value.is_ascii_punctuation(){
//!                     let mut value : [u8; 1] = [0];
//!                     char_value.encode_utf8(&mut value);
//!                     WinConsole::output().write_utf8(&value);
//!                 }
//!                 else{
//!                     match key.virtual_key_code {
//!                         ESCAPE => { break; },
//!                         ENTER => { WinConsole::output().write_utf8("\n".as_bytes()); }
//!                         SPACE => { WinConsole::output().write_utf8(" ".as_bytes()); },
//!                         BACKSPACE => { WinConsole::output().write_utf8(b"\x08 \x08"); },
//!                         _ => {}
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```

/// Provides the `WinConsole` that wraps calls to native methods to interact with the console.
pub mod console;
/// Includes console related structs as `ConsoleColor`, `CharInfo` or `ConsoleCursorInfo`.
pub mod structs;
pub mod input;
