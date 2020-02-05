use win32console::console::WinConsole;
use win32console::structs::input::*;
use win32console::structs::console_color::ConsoleColor;

fn main() {
    // Virtual key codes
    // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    const ESCAPE : u16 = 0x1B;
    const BACKSPACE: u16 = 0x08;
    const ENTER : u16 = 0x0D;
    const SPACE : u16 = 0x20;

    WinConsole::output().clear();

    write_color("Hello, can you write something?\n", ConsoleColor::DarkYellow);

    loop{
        // Get the current input event
        if let KeyEvent(key) = WinConsole::input().read_single_input().unwrap(){
            // Only check for key down events
            if key.key_down{
                let uchar = key.u_char;
                // Write only if is alphanumeric or punctuation
                if uchar.is_ascii_alphanumeric() || uchar.is_ascii_punctuation(){
                    write_char(uchar);
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

fn write_char(char_value: char){
    let mut value : [u8; 1] = [0];
    char_value.encode_utf8(&mut value);
    WinConsole::output().write_utf8(&value);
}

fn write_color(value: &str, color: ConsoleColor){
    let old_color = WinConsole::output().get_foreground_color().unwrap();
    WinConsole::output().set_foreground_color(color);
    WinConsole::output().write_utf8(value.as_bytes());
    WinConsole::output().set_foreground_color(old_color);
}