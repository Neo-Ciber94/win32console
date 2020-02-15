# Win32Console

Expose functions to interact with the windows console from **Rust**.

See: https://docs.microsoft.com/en-us/windows/console/console-functions

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

  Native Function                  | WinConsole function      | Description                                                                                                                 
 :---------------------------------|:------------------------:|:-----------------------------------------------------------------------------------------------------------------------------
  AddConsoleAlias                  | -                        | Defines a console alias for the specified executable.                                                                       
  AllocConsole                     | alloc_console            | Allocates a new console for the calling process.                                                                            
  AttachConsole                    | attach_console           | Attaches the calling process to the console of the specified process.                                                       
  ClosePseudoConsole               | -                        | Closes a pseudoconsole from the given handle.                                                                               
  CreatePseudoConsole              | -                        | Allocates a new pseudoconsole for the calling process.                                                                      
  CreateConsoleScreenBuffer        | create_screen_console_buffer                        | Creates a console screen buffer.                                                                                            
  FillConsoleOutputAttribute       |                          | Sets the text and background color attributes for a specified number of character cells.                                    
  FillConsoleOutputCharacter       |                          | Writes a character to the console screen buffer a specified number of times.                                                
  FlushConsoleInputBuffer          |                          | Flushes the console input buffer.                                                                                           
  FreeConsole                      |                          | Detaches the calling process from its console.                                                                              
  GenerateConsoleCtrlEvent         |                          | Sends a specified signal to a console process group that shares the console associated with the calling process.            
  GetConsoleAlias                  |                          | Retrieves the specified alias for the specified executable.                                                                 
  GetConsoleAliases                |                          | Retrieves all defined console aliases for the specified executable.                                                         
  GetConsoleAliasesLength          |                          | Returns the size, in bytes, of the buffer needed to store all of the console aliases for the specified executable.          
  GetConsoleAliasExes              |                          | Retrieves the names of all executables with console aliases defined.                                                        
  GetConsoleAliasExesLength        |                          | Returns the size, in bytes, of the buffer needed to store the names of all executables that have console aliases defined.   
  GetConsoleCP                     |                          | Retrieves the input code page used by the console associated with the calling process.                                      
  GetConsoleCursorInfo             |                          | Retrieves information about the size and visibility of the cursor for the specified console screen buffer.                  
  GetConsoleDisplayMode            |                          | Retrieves the display mode of the current console.                                                                          
  GetConsoleFontSize               |                          | Retrieves the size of the font used by the specified console screen buffer.                                                 
  GetConsoleHistoryInfo            |                          | Retrieves the history settings for the calling process's console.                                                           
  GetConsoleMode                   |                          | Retrieves the current input mode of a console's input buffer or the current output mode of a console screen buffer.         
  GetConsoleOriginalTitle          |                          | Retrieves the original title for the current console window.                                                                
  GetConsoleOutputCP               |                          | Retrieves the output code page used by the console associated with the calling process.                                     
  GetConsoleProcessList            |                          | Retrieves a list of the processes attached to the current console.                                                          
  GetConsoleScreenBufferInfo       |                          | Retrieves information about the specified console screen buffer.                                                            
  GetConsoleScreenBufferInfoEx     |                          | Retrieves extended information about the specified console screen buffer.                                                   
  GetConsoleSelectionInfo          |                          | Retrieves information about the current console selection.                                                                  
  GetConsoleTitle                  |                          | Retrieves the title for the current console window.                                                                         
  GetConsoleWindow                 |                          | Retrieves the window handle used by the console associated with the calling process.                                        
  GetCurrentConsoleFont            |                          | Retrieves information about the current console font.                                                                       
  GetCurrentConsoleFontEx          |                          | Retrieves extended information about the current console font.                                                              
  GetLargestConsoleWindowSize      |                          | Retrieves the size of the largest possible console window.                                                                  
  GetNumberOfConsoleInputEvents    |                          | Retrieves the number of unread input records in the console's input buffer.                                                 
  GetNumberOfConsoleMouseButtons   |                          | Retrieves the number of buttons on the mouse used by the current console.                                                   
  GetStdHandle                     |                          | Retrieves a handle for the standard input, standard output, or standard error device.                                       
  HandlerRoutine                   |                          | An application-defined function used with the SetConsoleCtrlHandler function.                                               
  PeekConsoleInput                 |                          | Reads data from the specified console input buffer without removing it from the buffer.                                     
  ReadConsole                      |                          | Reads character input from the console input buffer and removes it from the buffer.                                         
  ReadConsoleInput                 |                          | Reads data from a console input buffer and removes it from the buffer.                                                      
  ReadConsoleOutput                |                          | Reads character and color attribute data from a rectangular block of character cells in a console screen buffer.            
  ReadConsoleOutputAttribute       |                          | Copies a specified number of foreground and background color attributes from consecutive cells of a console screen buffer.  
  ReadConsoleOutputCharacter       |                          | Copies a number of characters from consecutive cells of a console screen buffer.                                            
  ResizePseudoConsole              |                          | Resizes the internal buffers for a pseudoconsole to the given size.                                                         
  ScrollConsoleScreenBuffer        |                          | Moves a block of data in a screen buffer.                                                                                   
  SetConsoleActiveScreenBuffer     |                          | Sets the specified screen buffer to be the currently displayed console screen buffer.                                       
  SetConsoleCP                     |                          | Sets the input code page used by the console associated with the calling process.                                           
  SetConsoleCtrlHandler            |                          | Adds or removes an application-defined HandlerRoutine from the list of handler functions for the calling process.           
  SetConsoleCursorInfo             |                          | Sets the size and visibility of the cursor for the specified console screen buffer.                                         
  SetConsoleCursorPosition         |                          | Sets the cursor position in the specified console screen buffer.                                                            
  SetConsoleDisplayMode            |                          | Sets the display mode of the specified console screen buffer.                                                               
  SetConsoleHistoryInfo            |                          | Sets the history settings for the calling process's console.                                                                
  SetConsoleMode                   |                          | Sets the input mode of a console's input buffer or the output mode of a console screen buffer.                              
  SetConsoleOutputCP               |                          | Sets the output code page used by the console associated with the calling process.                                          
  SetConsoleScreenBufferInfoEx     |                          | Sets extended information about the specified console screen buffer.                                                        