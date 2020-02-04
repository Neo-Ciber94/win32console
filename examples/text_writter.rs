#[macro_use]
extern crate lazy_static;
extern crate win32console;

use std::io::{Error, Result, Write, ErrorKind};
use std::sync::Mutex;

use win32console::console::WinConsole;
use win32console::structs::console_color::ConsoleColor;
use win32console::structs::input::*;

// Virtual key codes
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
const ESCAPE: u16 = 0x1B;
const BACKSPACE: u16 = 0x08;
const ENTER: u16 = 0x0D;
const SPACE: u16 = 0x20;

lazy_static! {
    static ref BUFFER: Mutex<String> = Mutex::new(String::with_capacity(32));
}

fn main() {
    // Clear the console
    WinConsole::output().clear().unwrap();

    // Write a colored title
    write_color("Press [ESC] for exit\n\n", ConsoleColor::DarkYellow);

    loop {
        // Get the current input event
        if let KeyEvent(key_event) = WinConsole::input().read_single_input().unwrap() {
            // Only check for key down events
            if key_event.key_down {
                let uchar = key_event.u_char;
                // Write only if is alphanumeric or punctuation
                if uchar.is_alphanumeric() || uchar.is_ascii_punctuation() {
                    write_char(uchar);
                } else {
                    match key_event.virtual_key_code {
                        ESCAPE => {
                            break;
                        }
                        ENTER => {
                            WinConsole::output().write_utf8("\n".as_bytes()).unwrap();
                            BUFFER.lock().unwrap().push('\n');
                        }
                        SPACE => {
                            WinConsole::output().write_utf8(" ".as_bytes()).unwrap();
                            BUFFER.lock().unwrap().push(' ');
                        }
                        BACKSPACE => {
                            WinConsole::output().write_utf8(b"\x08 \x08").unwrap();
                            BUFFER.lock().unwrap().pop();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Write a colored title
    write_color("\nSave [S] or Discard [D]?\n", ConsoleColor::DarkYellow);

    // Save or discard?
    if BUFFER.lock().unwrap().is_not_blank() {
        loop {
            if let KeyEvent(key) = WinConsole::input().read_single_input().unwrap() {
                if key.key_down {
                    let uchar = key.u_char;
                    match uchar.to_ascii_lowercase() {
                        's' => {
                            if let Err(e) = save_file(){
                                write_color(
                                    format!("Cannot save the file, error: {:?}", e).as_str(),
                                    ConsoleColor::DarkRed,
                                );
                            }
                            break;
                        }
                        'd' => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn write_char(char_value: char) {
    // utf16 to allow spanish tildes
    let mut value: [u16; 1] = [0];

    char_value.encode_utf16(&mut value);
    WinConsole::output().write_utf16(&value).unwrap();
    BUFFER.lock().unwrap().push(char_value);
}

fn write_color(value: &str, color: ConsoleColor) {
    // Save old color to restore it later
    let old_color = WinConsole::output().get_foreground_color().unwrap();

    // Set the given color
    WinConsole::output().set_foreground_color(color).unwrap();
    // Write the text with the color
    WinConsole::output().write_utf8(value.as_bytes()).unwrap();
    // Restore the old color
    WinConsole::output()
        .set_foreground_color(old_color)
        .unwrap();
}

fn save_file() -> Result<()> {
    write_color("File name: ", ConsoleColor::DarkYellow);

    let temp = WinConsole::input().read().unwrap().trim().to_string();
    if temp.is_not_blank() {
        use std::fs::File;

        let mut file_name = temp.clone();

        if !file_name.ends_with(".txt") {
            file_name.push_str(".txt");
        }

        return match File::create(file_name) {
            Ok(mut f) => {
                f.write_all(BUFFER.lock().unwrap().as_bytes()).unwrap();
                Ok(())
            }
            Err(e) => Err(e),
        };
    }

    Err(Error::new(ErrorKind::InvalidInput, "File name cannot be blank"))
}

trait IsBlank {
    fn is_blank(&self) -> bool;

    fn is_not_blank(&self) -> bool {
        !self.is_blank()
    }
}

impl IsBlank for String {
    fn is_blank(&self) -> bool {
        if self.len() == 0 {
            return true;
        }

        self.chars()
            .filter(|c| !c.is_whitespace())
            .count() == 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_not_blank_test1() {
        let s = String::from("Hello");
        assert!(s.is_not_blank());
    }

    #[test]
    fn is_not_blank_test2() {
        let s = String::from("Hello\n");
        assert!(s.is_not_blank());
    }

    #[test]
    fn is_blank_test1() {
        let s = String::from("  ");
        assert!(s.is_blank());
    }

    #[test]
    fn is_blank_test2() {
        let s = String::from("   \t");
        assert!(s.is_blank());
    }
}
