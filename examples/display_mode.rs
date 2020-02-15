use win32console::console::{WinConsole, DisplayMode};
use winapi::_core::time::Duration;
use win32console::structs::input::InputRecord::KeyEvent;

// Virtual key codes
// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
const SPACE : u16 = 0x20;
const ESCAPE: u16 = 0x1B;

fn main(){
    // Get if the window is fullscreen or not
    let mut full_screen = WinConsole::get_actual_display_mode().unwrap() == DisplayMode::FullScreen;
    // Flag to update the console
    let mut update = false;

    loop{
        // prints the actual display mode
        let mode = WinConsole::get_actual_display_mode().unwrap();
        println!("Current console mode: {:?}", mode);

        if let KeyEvent(key) = WinConsole::input().read_single_input().unwrap(){
            // Only check key down events
            if key.key_down{
                // If space if press change the display mode
                if key.virtual_key_code == SPACE{
                    full_screen = !full_screen;
                    update = true;
                }

                // If escape is press exit the process
                if key.virtual_key_code == ESCAPE{
                    break;
                }
            }

            // Change the console mode
            if update{
                update = false;

                let mode = match full_screen{
                    true => DisplayMode::FullScreen,
                    false => DisplayMode::Windowed
                };

                WinConsole::output().set_display_mode(mode).expect("Cannot change display mode");
                println!("Console mode changed to: {:?}", mode);
            }
        }
    }
}

