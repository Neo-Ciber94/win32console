use win32console::console::WinConsole;
use win32console::structs::input::*;
use win32console::structs::console_color::ConsoleColor;

// Virtual key codes
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
const ESCAPE : u16 = 0x1B;
const BACKSPACE: u16 = 0x08;
const ENTER : u16 = 0x0D;
const SPACE : u16 = 0x20;

fn main() {
    // Clears the screen
    WinConsole::output().clear().expect("Cannot clear the screen");

    // Writes to the screen using the color 'Dark Red'
    write_color_str("What's your name? ", ConsoleColor::DarkRed);

    // Stores the result of the read_string() function
    let name = with_color(ConsoleColor::Green, read_string);

    // Writes to the screen a message depending if 'name' is blank or not
    if name.is_not_blank(){
        write_color_string(format!("\nHello {}!", name), ConsoleColor::DarkBlue);
    }
    else{
        write_color_str("\nWait, what?", ConsoleColor::DarkBlue);
    }
}

fn read_string() -> String{
    // The buffer to store the string to write
    let mut buffer: String = String::new();

    loop{
        // Get the current input event
        if let KeyEvent(key) = WinConsole::input().read_single_input().unwrap(){
            // Only check for key down events
            if key.key_down{
                let uchar = key.u_char;
                // Write only if is alphanumeric or punctuation
                if uchar.is_ascii_alphanumeric() || uchar.is_ascii_punctuation(){
                    // Writes the char to the screen and push it to the string buffer
                    write_char(uchar);
                    buffer.push(uchar);
                }
                else{
                    match key.virtual_key_code {
                        ESCAPE => { break; },   // Exit on [ESC] press
                        ENTER => { break; }     // Exit on [Enter] press
                        SPACE => {
                            // Write the whitespace to the screen and string buffer
                            WinConsole::output().write_utf8(" ".as_bytes()).unwrap();
                            buffer.push(' ');
                        },
                        BACKSPACE => {
                            // If the buffer is not empty removes the last char from the screen
                            // and string buffer.
                            if !buffer.is_empty(){
                                WinConsole::output().write_utf8(b"\x08 \x08").unwrap();
                                buffer.pop();
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    // String ends with a newline
    buffer.trim().to_string()
}

fn write_char(char_value: char){
    // The buffer to store the char
    let mut buffer = [0u8; 1];
    // Store the encode value to teh buffer
    char_value.encode_utf8(&mut buffer);
    // Write the utf8 buffer to the screen
    WinConsole::output().write_utf8(&buffer).unwrap();
}

fn write_color_str(value: &str, color: ConsoleColor){
    // Stores the old color
    let old_color = WinConsole::output().get_foreground_color().unwrap();

    // Sets the new color
    WinConsole::output().set_foreground_color(color).unwrap();
    // Write with the new color
    WinConsole::output().write_utf8(value.as_bytes()).unwrap();
    // Resets the color
    WinConsole::output().set_foreground_color(old_color).unwrap();
}

fn write_color_string(value: String, color: ConsoleColor){
    // Stores the old color
    let old_color = WinConsole::output().get_foreground_color().unwrap();

    // Sets the new color
    WinConsole::output().set_foreground_color(color).unwrap();
    // Write with the new color
    WinConsole::output().write_utf8(value.as_bytes()).unwrap();
    // Resets the color
    WinConsole::output().set_foreground_color(old_color).unwrap();
}

fn with_color<R : Sized, F : Fn() -> R>(color: ConsoleColor, func: F) -> R{
    // Stores the old color
    let old_color = WinConsole::output().get_foreground_color().unwrap();

    // Sets the new color
    WinConsole::output().set_foreground_color(color).unwrap();
    // Calls the function that may try to write using WinConsole
    let result = func();
    // Resets the color
    WinConsole::output().set_foreground_color(old_color).unwrap();

    // Returns the result provided by the function
    result
}

trait IsBlank{
    fn is_blank(&self) -> bool;

    #[inline]
    fn is_not_blank(&self) -> bool{
        !self.is_blank()
    }
}

impl IsBlank for String{
    fn is_blank(&self) -> bool {
        self.len() == 0 || self.chars().all(|c| c.is_whitespace())
    }
}

impl IsBlank for str{
    fn is_blank(&self) -> bool {
        self.len() == 0 || self.chars().all(|c| c.is_whitespace())
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn is_blank_str_test(){
        assert!(!"Hello\n".is_blank());
        assert!("    ".is_blank());
        assert!("\t\t  ".is_blank());
    }

    #[test]
    fn is_blank_string_test(){
        assert!(!String::from("Hello\n").is_blank());
        assert!(String::from("    ").is_blank());
        assert!(String::from("\t\t  ").is_blank());
    }
}