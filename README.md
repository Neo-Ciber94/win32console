# Win32Console
![https://crates.io/crates/win32console](https://img.shields.io/badge/crates.io-v1.3-orange)
![https://docs.rs/win32console/0.1.3/win32console/](https://img.shields.io/badge/docs-v1.3-yellow)

Expose functions to interact with the windows console from **Rust**.

See: https://docs.microsoft.com/en-us/windows/console/console-functions

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
win32console = "0.1.3"
```

## Example
```rust
use win32console::console::WinConsole;
use win32console::structs::input::*;

fn main() {
    // Virtual key codes
    // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    const ESCAPE : u16 = 0x1B;
    const BACKSPACE: u16 = 0x08;
    const ENTER : u16 = 0x0D;
    const SPACE : u16 = 0x20;

    loop{
        // Get the current input event
        if let KeyEvent(key) = WinConsole::input().read_single_input().unwrap(){
            // Only check for key down events
            if key.key_down{
                let char_value = key.u_char;
                // Write only if is alphanumeric or punctuation
                if char_value.is_ascii_alphanumeric() || char_value.is_ascii_punctuation(){
                    let mut value : [u8; 1] = [0];
                    char_value.encode_utf8(&mut value);
                    WinConsole::output().write_utf8(&value);
                }
                else{
                    match key.virtual_key_code {
                        ESCAPE => { break; },
                        ENTER => { WinConsole::output().write_utf8("\n".as_bytes()); }
                        SPACE => { WinConsole::output().write_utf8(" ".as_bytes()); },
                        BACKSPACE => { WinConsole::output().write_utf8(b"\x08 \x08"); },
                        _ => {}
                    }
                }
            }
        }
    }
}
```

## Implementation
[Here](Win32Functions.md) a list of the console methods implemented in this library.

And also this library provides functions as:
```c++
// Clears the screen
WinConsole::output().clear();

// Reads a 'String' from the console
WinConsole::input().read_string();

// Makes and tone sound
WinConsole::beep(u32, u32);

// Sets the foreground color
WinConsole::output().set_foreground_color(ConsoleColor);

// Sets the background color
WinConsole::output().set_background_color(ConsoleColor);

// Gets the foreground color
WinConsole::output().get_foreground_color();

// Gets the background color
WinConsole::output().get_background_color();
```