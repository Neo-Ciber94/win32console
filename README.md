# Win32Console

Expose functions to interact with the windows console from **Rust**.

See: https://docs.microsoft.com/en-us/windows/console/console-functions

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
win32console = ""
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
List of the native methods of `Wincon` implemented in this library:

  Native Function                   | WinConsole Function           | Description                                                                                                                 
  :---------------------------------|:------------------------------|:-----------------------------------------------------------------------------------------------------------------------------
   AddConsoleAlias                  | -                             | Defines a console alias for the specified executable.                                                                       
   AllocConsole                     | alloc_console                 | Allocates a new console for the calling process.                                                                            
   AttachConsole                    | attach_console                | Attaches the calling process to the console of the specified process.                                                       
   ClosePseudoConsole               | -                             | Closes a pseudoconsole from the given handle.                                                                               
   CreatePseudoConsole              | -                             | Allocates a new pseudoconsole for the calling process.                                                                      
   CreateConsoleScreenBuffer        | create_console_buffer         | Creates a console screen buffer.                                                                                            
   FillConsoleOutputAttribute       | fill_with_attribute           | Sets the text and background color attributes for a specified number of character cells.                                    
   FillConsoleOutputCharacter       | fill_with_char                | Writes a character to the console screen buffer a specified number of times.                                                
   FlushConsoleInputBuffer          | flush                         | Flushes the console input buffer.                                                                                           
   FreeConsole                      | free_console                  | Detaches the calling process from its console.                                                                              
   GenerateConsoleCtrlEvent         | -                             | Sends a specified signal to a console process group that shares the console associated with the calling process.            
   GetConsoleAlias                  | -                             | Retrieves the specified alias for the specified executable.                                                                 
   GetConsoleAliases                | -                             | Retrieves all defined console aliases for the specified executable.                                                         
   GetConsoleAliasesLength          | -                             | Returns the size, in bytes, of the buffer needed to store all of the console aliases for the specified executable.          
   GetConsoleAliasExes              | -                             | Retrieves the names of all executables with console aliases defined.                                                        
   GetConsoleAliasExesLength        | -                             | Returns the size, in bytes, of the buffer needed to store the names of all executables that have console aliases defined.   
   GetConsoleCP                     | get_input_code_point          | Retrieves the input code page used by the console associated with the calling process.                                      
   GetConsoleCursorInfo             | get_cursor_info               | Retrieves information about the size and visibility of the cursor for the specified console screen buffer.                  
   GetConsoleDisplayMode            | get_display_mode              | Retrieves the display mode of the current console.                                                                          
   GetConsoleFontSize               | get_font_size                 | Retrieves the size of the font used by the specified console screen buffer.                                                 
   GetConsoleHistoryInfo            | get_history_info              | Retrieves the history settings for the calling process's console.                                                           
   GetConsoleMode                   | get_mode                      | Retrieves the current input mode of a console's input buffer or the current output mode of a console screen buffer.         
   GetConsoleOriginalTitle          | get_original_title            | Retrieves the original title for the current console window.                                                                
   GetConsoleOutputCP               | get_input_code_page           | Retrieves the output code page used by the console associated with the calling process.                                     
   GetConsoleProcessList            | get_process_list              | Retrieves a list of the processes attached to the current console.                                                          
   GetConsoleScreenBufferInfo       | get_screen_buffer_info        | Retrieves information about the specified console screen buffer.                                                            
   GetConsoleScreenBufferInfoEx     | get_screen_buffer_info_ex     | Retrieves extended information about the specified console screen buffer.                                                   
   GetConsoleSelectionInfo          | get_selection_info            | Retrieves information about the current console selection.                                                                  
   GetConsoleTitle                  | get_title                     | Retrieves the title for the current console window.                                                                         
   GetConsoleWindow                 | get_window                    | Retrieves the window handle used by the console associated with the calling process.                                        
   GetCurrentConsoleFont            | get_font                      | Retrieves information about the current console font.                                                                       
   GetCurrentConsoleFontEx          | get_font_ex                   | Retrieves extended information about the current console font.                                                              
   GetLargestConsoleWindowSize      | get_largest_window_size       | Retrieves the size of the largest possible console window.                                                                  
   GetNumberOfConsoleInputEvents    | get_number_of_input_events    | Retrieves the number of unread input records in the console's input buffer.                                                 
   GetNumberOfConsoleMouseButtons   | get_number_of_mouse_buttons   | Retrieves the number of buttons on the mouse used by the current console.                                                   
   GetStdHandle                     | get_std_handle                | Retrieves a handle for the standard input, standard output, or standard error device.                                       
   HandlerRoutine                   | -                             | An application-defined function used with the SetConsoleCtrlHandler function.                                               
   PeekConsoleInput                 | peek_input                    | Reads data from the specified console input buffer without removing it from the buffer.                                     
   ReadConsole                      | read_utf8/read_utf16          | Reads character input from the console input buffer and removes it from the buffer.                                         
   ReadConsoleInput                 | read_input                    | Reads data from a console input buffer and removes it from the buffer.                                                      
   ReadConsoleOutput                | read_output                   | Reads character and color attribute data from a rectangular block of character cells in a console screen buffer.            
   ReadConsoleOutputAttribute       | read_output_attribute         | Copies a specified number of foreground and background color attributes from consecutive cells of a console screen buffer.  
   ReadConsoleOutputCharacter       | read_output_character         | Copies a number of characters from consecutive cells of a console screen buffer.                                            
   ResizePseudoConsole              | -                             | Resizes the internal buffers for a pseudoconsole to the given size.                                                         
   ScrollConsoleScreenBuffer        | scroll_screen_buffer          | Moves a block of data in a screen buffer.                                                                                   
   SetConsoleActiveScreenBuffer     | set_active_screen_buffer      | Sets the specified screen buffer to be the currently displayed console screen buffer.                                       
   SetConsoleCP                     | set_input_code_page           | Sets the input code page used by the console associated with the calling process.                                           
   SetConsoleCtrlHandler            | -                             | Adds or removes an application-defined HandlerRoutine from the list of handler functions for the calling process.           
   SetConsoleCursorInfo             | set_cursor_info               | Sets the size and visibility of the cursor for the specified console screen buffer.                                         
   SetConsoleCursorPosition         | set_cursor_position           | Sets the cursor position in the specified console screen buffer.                                                            
   SetConsoleDisplayMode            | set_display_mode              | Sets the display mode of the specified console screen buffer.                                                               
   SetConsoleHistoryInfo            | set_history_info              | Sets the history settings for the calling process's console.                                                                
   SetConsoleMode                   | set_mode                      | Sets the input mode of a console's input buffer or the output mode of a console screen buffer.                              
   SetConsoleOutputCP               | set_output_code_page          | Sets the output code page used by the console associated with the calling process.                                          
   SetConsoleScreenBufferInfoEx     | set_screen_buffer_info        | Sets extended information about the specified console screen buffer.                                                        
   SetConsoleScreenBufferSize       | set_screen_buffer_size        | Changes the size of the specified console screen buffer.                                                                    
   SetConsoleTextAttribute          | set_text_attribute            | Sets the foreground (text) and background color attributes of characters written to the console screen buffer.              
   SetConsoleTitle                  | set_title                     | Sets the title for the current console window.                                                                              
   SetConsoleWindowInfo             | set_window_info               | Sets the current size and position of a console screen buffer's window.                                                     
   SetCurrentConsoleFontEx          | set_font_ex                   | Sets extended information about the current console font.                                                                   
   SetStdHandle                     | set_std_handle                | Sets the handle for the standard input, standard output, or standard error device.                                          
   WriteConsole                     | write_utf8/write_utf16        | Writes a character string to a console screen buffer beginning at the current cursor location.                              
   WriteConsoleInput                | write_input                   | Writes data directly to the console input buffer.                                                                           
   WriteConsoleOutput               | write_output                  | Writes character and color attribute data to a specified rectangular block of character cells in a console screen buffer.   
   WriteConsoleOutputAttribute      | write_output_attribute        | Copies a number of foreground and background color attributes to consecutive cells of a console screen buffer.              
   WriteConsoleOutputCharacter      | write_output_character        | Copies a number of characters to consecutive cells of a console screen buffer.                                              

Also provides functions as:
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