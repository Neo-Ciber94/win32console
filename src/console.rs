use std::{
    convert::TryFrom,
    io::{Error, ErrorKind, Result},
    iter,
    mem::{MaybeUninit},
    slice,
    str,
    ptr::null_mut
};

use winapi::{
    um::{
        consoleapi::{
            WriteConsoleA,
            AllocConsole,
            GetConsoleCP,
            GetConsoleMode,
            GetConsoleOutputCP,
            GetNumberOfConsoleInputEvents,
            ReadConsoleInputW,
            ReadConsoleW,
            SetConsoleMode,
            WriteConsoleW
        },
        fileapi::{CreateFileW, OPEN_EXISTING, ReadFile, WriteFile},
        handleapi::INVALID_HANDLE_VALUE,
        processenv::{GetStdHandle, SetStdHandle},
        winbase::{STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
        wincon::{
            CONSOLE_FONT_INFOEX,
            FillConsoleOutputAttribute,
            FillConsoleOutputCharacterW,
            GetConsoleScreenBufferInfo,
            GetConsoleScreenBufferInfoEx,
            GetConsoleTitleW,
            GetCurrentConsoleFontEx,
            GetLargestConsoleWindowSize,
            GetNumberOfConsoleMouseButtons,
            INPUT_RECORD,
            PeekConsoleInputW,
            SetConsoleCursorPosition,
            SetConsoleScreenBufferInfoEx,
            SetConsoleTextAttribute,
            SetConsoleTitleW,
            SetCurrentConsoleFontEx,
            CONSOLE_FONT_INFO,
            GetCurrentConsoleFont,
            AttachConsole,
            CHAR_INFO,
            CONSOLE_READCONSOLE_CONTROL,
            CONSOLE_SCREEN_BUFFER_INFO,
            CONSOLE_SCREEN_BUFFER_INFOEX,
            FreeConsole,
            GetConsoleOriginalTitleW,
            SetConsoleCP,
            SetConsoleOutputCP,
            SetConsoleScreenBufferSize,
            SetConsoleWindowInfo,
            SMALL_RECT,
            WriteConsoleOutputW,
            CONSOLE_SELECTION_INFO,
            CONSOLE_TEXTMODE_BUFFER,
            CreateConsoleScreenBuffer,
            GetConsoleSelectionInfo,
            SetConsoleActiveScreenBuffer,
            ReadConsoleOutputW,
            FlushConsoleInputBuffer,
            ScrollConsoleScreenBufferW
        },
        wincontypes::{PCHAR_INFO, PSMALL_RECT},
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
    },
    ctypes::c_void,
    shared::minwindef::MAX_PATH,
};

use crate::{
    structs::char_info::CharInfo,
    structs::console_color::ConsoleColor,
    structs::console_font_info::ConsoleFontInfo,
    structs::console_font_info_ex::ConsoleFontInfoEx,
    structs::console_read_control::ConsoleReadControl,
    structs::console_screen_buffer_info::{ConsoleScreenBufferInfo},
    structs::console_screen_buffer_info_ex::ConsoleScreenBufferInfoEx,
    structs::coord::Coord,
    structs::handle::Handle,
    structs::input_record::InputRecord,
    structs::console_selection_info::ConsoleSelectionInfo,
    structs::small_rect::SmallRect
};
use winapi::um::wincon::{GetConsoleProcessList, SetConsoleHistoryInfo, CONSOLE_HISTORY_INFO, GetConsoleHistoryInfo};
use crate::structs::console_history_info::ConsoleHistoryInfo;

/// Provides an access to the windows console of the current process and provides methods for
/// interact with it.
///
/// # Example
/// ```
/// use win32console::console::WinConsole;
///
/// WinConsole::output().write_utf8("What's your name?".as_bytes()).unwrap();
/// let name = WinConsole::input().read_string().unwrap();
/// WinConsole::output().write_utf8(format!("Oh, Hello {}!", name.trim()).as_ref()).unwrap();
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WinConsole(Handle);

/// Type of a console handle, you can use this enum to get a handle by calling: [`get_std_handle`].
///
/// # Example
/// ```
/// use win32console::console::{WinConsole, HandleType};
///
/// let handle = WinConsole::get_std_handle(HandleType::Input).unwrap();
/// assert!(handle.is_valid());
/// ```
///
/// [`get_std_handle`]: struct.WinConsole.html#method.get_std_handle
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum HandleType {
    /// Represents the `STD_INPUT_HANDLE`.
    Input = STD_INPUT_HANDLE,
    /// Represents the `STD_OUTPUT_HANDLE`.
    Output = STD_OUTPUT_HANDLE,
    /// Represents the `STD_ERROR_HANDLE`.
    Error = STD_ERROR_HANDLE
}

/// Wraps constants values of the console modes.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/getconsolemode`
pub struct ConsoleMode;

/// Wraps constants values of the console text attributes.
///
/// link: `https://docs.microsoft.com/en-us/windows/console/console-screen-buffers`
pub struct ConsoleTextAttribute;

/// Wraps basics options to create a console.
///
/// See: `https://docs.microsoft.com/en-us/windows/console/createconsolescreenbuffer`
///
/// # Example
/// ```
/// use win32console::console::{ConsoleOptions, WinConsole};
/// let options = ConsoleOptions::new()
///     .generic_read()
///     .generic_write()
///     .shared_read()
///     .shared_write();
///
/// let handle = WinConsole::create_console_screen_buffer_with_options(options).unwrap();
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ConsoleOptions{
    // The access to the console screen buffer.
    desired_access: u32,
    // Indicate how the buffer can be shared.
    share_mode: u32
}

impl ConsoleMode {
    /// CTRL+C is processed by the system and is not placed in the input buffer.
    /// If the input buffer is being read by `ReadFile` or `ReadConsole`,
    /// other control keys are processed by the system and are not returned in the `ReadFile` or `ReadConsole` buffer
    pub const ENABLE_PROCESSED_INPUT: u32 = 0x0001;

    /// The ReadFile or ReadConsole function returns only when a carriage return character is read.
    /// If this mode is disabled, the functions return when one or more characters are available.
    pub const ENABLE_LINE_INPUT: u32 = 0x0002;

    /// Characters read by the `ReadFile` or `ReadConsole` function are written to the active screen buffer as they are read.
    /// This mode can be used only if the `ENABLE_LINE_INPUT` mode is also enabled.
    pub const ENABLE_ECHO_INPUT: u32 = 0x0004;

    /// User interactions that change the size of the console screen buffer are reported in the console's input buffer.
    pub const ENABLE_WINDOW_INPUT: u32 = 0x0008;

    /// If the mouse pointer is within the borders of the console window and the window has the keyboard focus,
    /// mouse events generated by mouse movement and button presses are placed in the input buffer.
    pub const ENABLE_MOUSE_INPUT: u32 = 0x0010;

    /// When enabled, text entered in a console window will be inserted at the current cursor location and
    /// all text following that location will not be overwritten.
    /// When disabled, all following text will be overwritten.
    pub const ENABLE_INSERT_MODE: u32 = 0x0020;

    /// This flag enables the user to use the mouse to select and edit text.
    pub const ENABLE_QUICK_EDIT_MODE: u32 = 0x0040;

    /// Required to enable or disable extended flags. See `ENABLE_INSERT_MODE` and `ENABLE_QUICK_EDIT_MODE`.
    /// link: `https://docs.microsoft.com/en-us/windows/console/setconsolemode#parameters`
    pub const ENABLE_EXTENDED_FLAGS: u32 = 0x0080;

    /// Setting this flag directs the Virtual Terminal processing engine to convert user input received by the console window
    /// into Console Virtual Terminal Sequences that can be retrieved by a supporting application
    /// through `ReadFile` or `ReadConsole` functions.
    pub const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;
}

impl ConsoleTextAttribute {
    /// Text color contains blue.
    pub const FOREGROUND_BLUE: u16 = 0x0001;
    /// Text color contains green.
    pub const FOREGROUND_GREEN: u16 = 0x0002;
    /// Text color contains red.
    pub const FOREGROUND_RED: u16 = 0x0004;
    /// Text color is intensified.
    pub const FOREGROUND_INTENSITY: u16 = 0x0008;
    /// Background color contains blue.
    pub const BACKGROUND_BLUE: u16 = 0x0010;
    /// Background color contains green.
    pub const BACKGROUND_GREEN: u16 = 0x0020;
    /// Background color contains red.
    pub const BACKGROUND_RED: u16 = 0x0040;
    /// Background color is intensified.
    pub const BACKGROUND_INTENSITY: u16 = 0x0080;
    /// Leading byte.
    pub const COMMON_LVB_LEADING_BYTE: u16 = 0x0100;
    /// Trailing byte.
    pub const COMMON_LVB_TRAILING_BYTE: u16 = 0x0200;
    /// Top horizontal
    pub const COMMON_LVB_GRID_HORIZONTAL: u16 = 0x0400;
    /// Left vertical.
    pub const COMMON_LVB_GRID_LVERTICAL: u16 = 0x0800;
    /// Right vertical.
    pub const COMMON_LVB_GRID_RVERTICAL: u16 = 0x1000;
    /// Reverse foreground and background attribute.
    pub const COMMON_LVB_REVERSE_VIDEO: u16 = 0x4000;
    /// Underscore.
    pub const COMMON_LVB_UNDERSCORE: u16 = 0x8000;
}

impl ConsoleOptions{
    /// Create a new `ConsoleOptions`.
    #[inline]
    pub fn new() -> Self{
        ConsoleOptions{ desired_access: 0, share_mode: 0 }
    }

    /// Gets the desired access of the console.
    #[inline]
    pub fn get_desired_access(&self) -> u32{
        self.desired_access
    }

    /// Gets the share mode of the console.
    #[inline]
    pub fn get_share_mode(&self) -> u32{
        self.share_mode
    }

    /// Set the console as `GENERIC_READ`.
    pub fn generic_read(mut self) -> ConsoleOptions{
        self.desired_access |= GENERIC_READ;
        self
    }

    /// Set the console as `GENERIC_WRITE`.
    pub fn generic_write(mut self) -> ConsoleOptions{
        self.desired_access |= GENERIC_WRITE;
        self
    }

    /// Set the console as `FILE_SHARE_READ`.
    pub fn shared_read(mut self) -> ConsoleOptions{
        self.share_mode |= FILE_SHARE_READ;
        self
    }

    /// Set the console as `FILE_SHARE_WRITE`.
    pub fn shared_write(mut self) -> ConsoleOptions{
        self.share_mode |= FILE_SHARE_WRITE;
        self
    }
}

// Get console handle associative methods
impl WinConsole {
    /// Gets the specified handle by type.
    ///
    /// Wraps a call to [GetStdHandle](https://docs.microsoft.com/en-us/windows/console/getstdhandle).
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, HandleType};
    /// let input = WinConsole::get_std_handle(HandleType::Input).unwrap();
    /// ```
    pub fn get_std_handle(handle_type: HandleType) -> Result<Handle> {
        unsafe {
            let raw_handle = GetStdHandle(handle_type as u32);
            if raw_handle == INVALID_HANDLE_VALUE {
                // Invalid handle
                // https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-
                return Err(Error::from_raw_os_error(0x6));
            }

            Ok(Handle::new(raw_handle))
        }
    }

    /// Sets the specified handle by type.
    ///
    /// Wraps a call to [SetStdHandle](https://docs.microsoft.com/en-us/windows/console/setstdhandle).
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, HandleType};
    /// let input = WinConsole::get_std_handle(HandleType::Input).unwrap();
    /// WinConsole::set_std_handle(HandleType::Input, input);
    /// ```
    pub fn set_std_handle(handle_type: HandleType, handle: Handle) -> Result<()> {
        unsafe {
            if SetStdHandle(handle_type as u32, *handle) == 0 {
                return Err(Error::last_os_error());
            }

            Ok(())
        }
    }

    /// Creates a Handle to the standard input file `CONIN$`, if the input
    /// is being redirected the value returned by [`get_std_handle`] cannot be used
    /// in functions that requires the console handle, but the returned `Handle`
    /// of this method can be used even if the input is being redirected.
    ///
    /// More info about console handles: `https://docs.microsoft.com/en-us/windows/console/console-handles`
    ///
    /// Wraps a call to [CreateFileW](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let handle = WinConsole::get_current_input_handle().unwrap();
    /// ```
    ///
    /// [`get_std_handle`]: #method.get_std_handle
    pub fn get_current_input_handle() -> Result<Handle> {
        // Rust strings are no null terminated
        let file_name: Vec<u16> = "CONIN$\0".encode_utf16().collect();

        let raw_handle = unsafe {
            CreateFileW(
                file_name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };

        if raw_handle == INVALID_HANDLE_VALUE {
            // Invalid handle
            // https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-
            return Err(Error::from_raw_os_error(0x6));
        }

        Ok(Handle::new_owned(raw_handle))
    }

    /// Creates a Handle to the standard output file `CONOUT$`, if the input
    /// is being redirected the value returned by [`get_std_handle`] cannot be used
    /// in functions that requires the console handle, but the returned `Handle`
    /// of this method can be used even if the output is being redirected.
    ///
    /// More info about console handles: `https://docs.microsoft.com/en-us/windows/console/console-handles`
    ///
    /// Wraps a call to [CreateFileW](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let handle = WinConsole::get_current_output_handle().unwrap();
    /// ```
    /// [`get_std_handle`]: #method.get_std_handle
    pub fn get_current_output_handle() -> Result<Handle> {
        // Rust strings are no null terminated
        let file_name: Vec<u16> = "CONOUT$\0".encode_utf16().collect();

        let raw_handle = unsafe {
            CreateFileW(
                file_name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };

        if raw_handle == INVALID_HANDLE_VALUE {
            // Invalid handle
            // https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-
            return Err(Error::from_raw_os_error(0x6));
        }

        Ok(Handle::new_owned(raw_handle))
    }
}

// Factory methods
impl WinConsole {
    /// Gets a console with the `STD_INPUT_HANDLE`.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::input();
    /// ```
    #[inline]
    pub fn input() -> WinConsole {
        WinConsole(WinConsole::get_std_handle(HandleType::Input).unwrap())
    }

    /// Gets a console with the `STD_OUTPUT_HANDLE`.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::output();
    /// ```
    #[inline]
    pub fn output() -> WinConsole {
        WinConsole(WinConsole::get_std_handle(HandleType::Output).unwrap())
    }

    /// Gets a console with the `STD_ERROR_HANDLE`.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::error();
    /// ```
    #[inline]
    pub fn error() -> WinConsole {
        WinConsole(WinConsole::get_std_handle(HandleType::Error).unwrap())
    }

    /// Gets a console with current input handle.
    /// The handle will be always the current input handle even is the input is being redirected.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::current_input();
    /// ```
    #[inline]
    pub fn current_input() -> WinConsole {
        WinConsole(WinConsole::get_current_input_handle().unwrap())
    }

    /// Gets a console with the current output handle.
    /// The handle will be always the current input handle even is the output is being redirected.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::current_output();
    /// ```
    #[inline]
    pub fn current_output() -> WinConsole {
        WinConsole(WinConsole::get_current_output_handle().unwrap())
    }

    /// Gets a console with the given handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, HandleType};
    /// let handle = WinConsole::get_std_handle(HandleType::Input).unwrap();
    /// let console = WinConsole::with_handle(handle);
    /// ```
    #[inline]
    pub fn with_handle(handle: Handle) -> WinConsole{
        WinConsole(handle)
    }
}

// Public methods
impl WinConsole {
    // Associative methods

    /// Allocates a new console for the calling process.
    ///
    /// # Errors
    /// - If the calling process have a console attached, `free_console` should be called first.
    ///
    /// Wraps a call to [AllocConsole](https://docs.microsoft.com/en-us/windows/console/allocconsole).
    pub fn alloc_console() -> Result<()> {
        unsafe {
            if AllocConsole() == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Attaches the calling process to the console of the specified process.
    ///
    /// - `proccess_id`: The identifier of the process whose console is to be used.
    /// This parameter can be one of the following values:
    /// * pid: Use the console of the specified process.
    /// * ATTACH_PARENT_PROCESS (0xFFFFFFFF): Use the console of the parent of the current process.
    ///
    /// Wraps a call to [AttachConsole](https://docs.microsoft.com/en-us/windows/console/attachconsole).
    ///
    /// # Errors
    /// - If the calling process is already attached to a console.
    /// - If the specified process does not have a console.
    /// - If the specified process does not exist.
    pub fn attach_console(process_id: u32) -> Result<()> {
        unsafe {
            if AttachConsole(process_id) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Detaches the calling process from its console.
    ///
    /// Wraps a call to [FreeConsole](https://docs.microsoft.com/en-us/windows/console/freeconsole).
    ///
    /// # Errors
    /// - If the calling process is not already attached to a console.
    pub fn free_console() -> Result<()> {
        unsafe {
            if FreeConsole() == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Sets the title of the current console.
    ///
    /// Wraps a call to [SetConsoleTitle](https://docs.microsoft.com/en-us/windows/console/setconsoletitle).
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    ///
    /// // Either `output` or `input` handle can be used
    /// WinConsole::set_title("Cool App!").unwrap();
    /// let title = WinConsole::get_title().unwrap();
    /// WinConsole::output().write_utf8(title.as_bytes()); // Cool App!
    ///
    /// loop {
    /// // We need a loop to see the title, when the process end the console will go back
    /// // to the original title
    /// if let KeyEvent(e) = WinConsole::input().read_single_input().unwrap() {
    ///     // when press escape, exit
    ///     // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    ///     if e.virtual_key_code == 0x1B {
    ///         break;
    ///     }
    ///   }
    /// }
    /// ```
    pub fn set_title(title: &str) -> Result<()> {
        let buffer = if title.ends_with('\0') {
            title.encode_utf16().collect::<Vec<u16>>()
        } else {
            let mut temp = title.to_string();
            temp.push('\0');
            temp.encode_utf16().collect::<Vec<u16>>()
        };

        unsafe {
            if SetConsoleTitleW(buffer.as_ptr()) == 0 {
                return Err(Error::last_os_error());
            }
            Ok(())
        }
    }

    /// Gets the title of the current console.
    ///
    /// Wraps a call to [GetConsoleTitle](https://docs.microsoft.com/en-us/windows/console/getconsoletitle).
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    ///
    /// // Either `output` or `input` handle can be used
    /// WinConsole::set_title("Cool App!").unwrap();
    /// let title = WinConsole::get_title().unwrap();
    /// WinConsole::output().write_utf8(title.as_bytes()); // Cool App!
    ///
    /// loop {
    /// // We need a loop to see the title, when the process end the console will go back
    /// // to the original title
    /// if let KeyEvent(e) = WinConsole::input().read_single_input().unwrap() {
    ///     // when press escape, exit
    ///     // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    ///     if e.virtual_key_code == 0x1B {
    ///         break;
    ///     }
    ///   }
    /// }
    /// ```
    pub fn get_title() -> Result<String> {
        let mut buffer: [u16; MAX_PATH as usize] = unsafe { MaybeUninit::zeroed().assume_init() };

        unsafe {
            let length = GetConsoleTitleW(buffer.as_mut_ptr(), MAX_PATH as u32) as usize;

            if length == 0 {
                Err(Error::last_os_error())
            } else {
                match String::from_utf16(&buffer) {
                    Ok(string) => Ok(string),
                    Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
                }
            }
        }
    }

    /// Retrieves the original title for the current console window.
    ///
    /// Wraps a call to [GetConsoleOriginalTitleW](https://docs.microsoft.com/en-us/windows/console/getconsoleoriginaltitle).
    ///
    /// # Errors
    /// - If f the buffer is not large enough to store the title.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let title = WinConsole::get_original_title().unwrap();
    /// WinConsole::output().write_utf8(title.as_bytes());
    /// ```
    pub fn get_original_title() -> Result<String> {
        let mut buffer: [u16; MAX_PATH as usize] = unsafe { MaybeUninit::zeroed().assume_init() };

        unsafe {
            if GetConsoleOriginalTitleW(buffer.as_mut_ptr(), buffer.len() as u32) == 0 {
                Err(Error::last_os_error())
            } else {
                match String::from_utf16(&buffer) {
                    Ok(string) => Ok(string),
                    Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
                }
            }
        }
    }

    /// Gets the input code page used by the console associated with the calling process.
    /// A console uses its input code page to translate keyboard input into the corresponding character value.
    ///
    /// See code pages: [`https://docs.microsoft.com/en-us/windows/win32/intl/code-page-identifiers`]
    ///
    /// Wraps a call to [GetConsoleCP](https://docs.microsoft.com/en-us/windows/console/getconsolecp).
    pub fn get_input_code_page() -> Result<u32> {
        unsafe {
            let code_page = GetConsoleCP();
            if code_page == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(code_page)
            }
        }
    }

    /// Gets the output code page used by the console associated with the calling process.
    /// A console uses its output code page to translate the character values written by the various output
    /// functions into the images displayed in the console window.
    ///
    /// See code pages: [`https://docs.microsoft.com/en-us/windows/win32/intl/code-page-identifiers`]
    ///
    /// Wraps a call to [GetConsoleOutputCP](https://docs.microsoft.com/en-us/windows/console/getconsoleoutputcp).
    pub fn get_output_code_page() -> Result<u32> {
        unsafe {
            let code_page = GetConsoleOutputCP();
            if code_page == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(code_page)
            }
        }
    }

    /// Sets the input code page used by the console associated with the calling process.
    /// A console uses its input code page to translate keyboard input into the corresponding character value.
    ///
    /// Wraps a call to [SetConsoleCP](https://docs.microsoft.com/en-us/windows/console/setconsolecp).
    pub fn set_input_code(code_page: u32) -> Result<()> {
        unsafe {
            if SetConsoleCP(code_page) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Sets the output code page used by the console associated with the calling process.
    /// A console uses its output code page to translate the character values written by the various output functions
    /// into the images displayed in the console window.
    ///
    /// Wraps a call to [SetConsoleOutputCP](https://docs.microsoft.com/en-us/windows/console/setconsoleoutputcp).
    pub fn set_output_code(code_page: u32) -> Result<()> {
        unsafe {
            if SetConsoleOutputCP(code_page) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Gets information about the current console selection.
    ///
    /// Wraps a call to [GetConsoleSelectionInfo](https://docs.microsoft.com/en-us/windows/console/getconsoleselectioninfo).
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let info = WinConsole::get_selection_info().unwrap();
    /// ```
    pub fn get_selection_info() -> Result<ConsoleSelectionInfo> {
        unsafe {
            let mut info: CONSOLE_SELECTION_INFO = std::mem::zeroed();

            if GetConsoleSelectionInfo(&mut info) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(ConsoleSelectionInfo::from(info))
            }
        }
    }

    /// Creates a new console screen buffer with:
    /// - `dwDesiredAccess` = GENERIC_READ | GENERIC_WRITE`
    /// - `dwShareMode` = FILE_SHARE_READ | FILE_SHARE_WRITE
    ///
    /// Wraps a call to [CreateConsoleScreenBuffer](https://docs.microsoft.com/en-us/windows/console/createconsolescreenbuffer).
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, HandleType};
    /// use std::time::Duration;
    ///
    /// let std_output_handle = WinConsole::get_std_handle(HandleType::Output).unwrap();
    /// let new_handle = WinConsole::create_console_screen_buffer().unwrap();
    ///
    /// assert!(std_output_handle.is_valid());
    /// assert!(new_handle.is_valid());
    ///
    /// // Write to the new handle
    /// WinConsole::with_handle(new_handle).write_utf8("Hola amigos!".as_bytes());
    ///
    /// // Write to the std output handle
    /// WinConsole::with_handle(std_output_handle).write_utf8("Hello Friends!".as_bytes());
    ///
    /// // Sets a new active screen buffer to display the message
    /// WinConsole::set_active_console_screen_buffer(&new_handle);
    /// // Keep the message visible for 3 secs
    /// std::thread::sleep(Duration::from_secs(3));
    /// // Restore the std output handle
    /// WinConsole::set_active_console_screen_buffer(&std_output_handle);
    /// ```
    pub fn create_console_screen_buffer() -> Result<Handle> {
        unsafe {
            let raw_handle = CreateConsoleScreenBuffer(
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                null_mut(),
                CONSOLE_TEXTMODE_BUFFER,
                null_mut(),
            );

            if raw_handle == INVALID_HANDLE_VALUE {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    "Cannot create a screen buffer",
                ))
            } else {
                Ok(Handle::new_owned(raw_handle))
            }
        }
    }

    /// Creates a new console screen buffer with the given options.
    pub fn create_console_screen_buffer_with_options(options: ConsoleOptions) -> Result<Handle>{
        unsafe {
            let raw_handle = CreateConsoleScreenBuffer(
                options.get_desired_access(),
                options.get_share_mode(),
                null_mut(),
                CONSOLE_TEXTMODE_BUFFER,
                null_mut(),
            );

            if raw_handle == INVALID_HANDLE_VALUE {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    "Cannot create a screen buffer",
                ))
            } else {
                Ok(Handle::new_owned(raw_handle))
            }
        }
    }

    /// Sets the specified screen buffer to be the currently displayed console screen buffer.
    ///
    /// Wraps a call to [SetConsoleActiveScreenBuffer](https://docs.microsoft.com/en-us/windows/console/setconsoleactivescreenbuffer).
    pub fn set_active_console_screen_buffer(handle: &Handle) -> Result<()> {
        unsafe {
            if SetConsoleActiveScreenBuffer(**handle) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Retrieves a list of the processes attached to the current console.
    ///
    /// Wraps a call to [GetConsoleProcessList](https://docs.microsoft.com/en-us/windows/console/getconsoleprocesslist).
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let process_id = WinConsole::get_process_list().unwrap();
    /// for pid in &process_id{
    ///     WinConsole::output().write_utf8(format!("{}", pid).as_bytes()).unwrap();
    /// }
    /// ```
    pub fn get_process_list() -> Result<Vec<u32>>{
        const BUFFER_SIZE : usize = 4;

        unsafe {
            let mut buffer = vec![u32::default(); BUFFER_SIZE];
            let mut process_count = GetConsoleProcessList(buffer.as_mut_ptr(), buffer.len() as u32);

            if process_count == 0{
                Err(Error::last_os_error())
            }
            else{
                if process_count > buffer.len() as u32 {
                    let required = process_count - buffer.len() as u32;
                    buffer.repeat(required as usize);
                    process_count = GetConsoleProcessList(buffer.as_mut_ptr(), buffer.len() as u32);
                }

                buffer.shrink_to_fit();
                Ok(buffer)
            }
        }
    }

    /// Sets the history settings for the calling process's console.
    ///
    /// Wraps a call to [SetConsoleHistoryInfo](https://docs.microsoft.com/en-us/windows/console/setconsolehistoryinfo).
    ///
    /// # Errors
    /// - If the calling process is not a console process, the function fails and sets the last error code to `ERROR_ACCESS_DENIED`.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let mut  info = WinConsole::get_history_info().unwrap();
    /// info.allow_duplicate_entries = false;
    /// WinConsole::set_history_info(info).unwrap();
    /// ```
    pub fn set_history_info(info: ConsoleHistoryInfo) -> Result<()>{
        unsafe{
            if SetConsoleHistoryInfo(&mut (info.into())) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Retrieves the history settings for the calling process's console.
    ///
    /// Wraps a call to [GetConsoleHistoryInfo](https://docs.microsoft.com/en-us/windows/console/getconsolehistoryinfo).
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let mut  info = WinConsole::get_history_info().unwrap();
    /// info.allow_duplicate_entries = false;
    /// WinConsole::set_history_info(info).unwrap();
    /// ```
    pub fn get_history_info() -> Result<ConsoleHistoryInfo>{
        let mut info : CONSOLE_HISTORY_INFO = unsafe { std::mem::zeroed() };

        unsafe{
            if GetConsoleHistoryInfo(&mut info) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(info.into())
            }
        }
    }

    // Instance methods

    /// Gets the handle used for this console, which will be provided by the `handle_provider`.
    pub fn get_handle(&self) -> &Handle {
        &self.0
    }

    /// Gets the current mode of the console
    ///
    /// Wraps a call to [GetConsoleMode](https://docs.microsoft.com/en-us/windows/console/getconsolemode).
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, ConsoleMode};
    ///
    /// let old_mode = WinConsole::input().get_mode().unwrap();
    /// let new_mode = ConsoleMode::ENABLE_PROCESSED_INPUT | ConsoleMode::ENABLE_LINE_INPUT;
    /// // We change the input mode so the characters are not displayed
    /// WinConsole::input().set_mode(new_mode);
    ///
    /// let value = WinConsole::input().read_string().unwrap(); // Don't will be displayed due new mode
    /// WinConsole::output().write_utf8(value.as_bytes());
    /// WinConsole::input().set_mode(old_mode); // Reset the mode
    /// ```
    pub fn get_mode(&self) -> Result<u32> {
        let handle = self.get_handle();
        let mut mode = 0;

        unsafe {
            if GetConsoleMode(**handle, &mut mode) != 0 {
                Ok(mode)
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Sets the current mode of the console
    ///
    /// Wraps a call to [GetConsoleMode](https://docs.microsoft.com/en-us/windows/console/setconsolemode).
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, ConsoleMode};
    ///
    /// let old_mode = WinConsole::input().get_mode().unwrap();
    /// let new_mode = ConsoleMode::ENABLE_PROCESSED_INPUT | ConsoleMode::ENABLE_LINE_INPUT;
    /// // We change the input mode so the characters are not displayed
    /// WinConsole::input().set_mode(new_mode);
    ///
    /// let value = WinConsole::input().read_string().unwrap(); // Don't will be displayed due new mode
    /// WinConsole::output().write_utf8(value.as_bytes());
    /// WinConsole::input().set_mode(old_mode); // Reset the mode
    /// ```
    pub fn set_mode(&self, mode: u32) -> Result<()> {
        let handle = self.get_handle();

        unsafe {
            if SetConsoleMode(**handle, mode) != 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
    }

    /// Checks if the console have the specified mode.
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, ConsoleMode};
    /// assert!(WinConsole::input().has_mode(ConsoleMode::ENABLE_PROCESSED_INPUT).unwrap());
    /// ```
    pub fn has_mode(&self, mode: u32) -> Result<bool> {
        match self.get_mode() {
            Ok(state) => Ok((state & mode) != 0),
            Err(error) => Err(error),
        }
    }

    /// Sets extended information about the console font.
    /// This function change the font into of all the current values in the console.
    ///
    /// Wraps a call to [SetCurrentConsoleFontEx](https://docs.microsoft.com/en-us/windows/console/setcurrentconsolefontex).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole};
    ///
    /// let old_info = WinConsole::output().get_font_info_ex(false).unwrap();
    /// let mut new_info = old_info;
    /// new_info.font_weight = 800; //Bold font
    /// WinConsole::output().set_font_info_ex(new_info, false).unwrap();
    /// WinConsole::output().write_utf8("Hello World".as_bytes()).unwrap();
    ///
    /// //  WinConsole::output().set_console_font_info(old_info).unwrap();
    /// // DON'T WILL SHOW BOTH `BOLD` AND `NORMAL` FONT!!
    ///
    /// // If we try to restore the old_info the new changes don't will be visible due
    /// // this method changes the font info of all the characters being displayed in the console
    /// ```
    ///
    /// If changes are not visibles in your current IDE try to execute directly the `.exe` in the folder.
    pub fn set_font_info_ex(&self, info: ConsoleFontInfoEx, maximum_window: bool) -> Result<()> {
        let handle = self.get_handle();
        let mut info = info.into();

        unsafe {
            if SetCurrentConsoleFontEx(**handle, maximum_window.into(), &mut info) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Gets information about the console font.
    ///
    /// Wraps a call to [GetCurrentConsoleFont](https://docs.microsoft.com/en-us/windows/console/getcurrentconsolefont).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole};
    /// let info = WinConsole::output().get_font_info(true).unwrap();
    /// ```
    pub fn get_font_info(&self, maximum_window: bool) -> Result<ConsoleFontInfo> {
        let handle = self.get_handle();

        unsafe {
            let mut info: CONSOLE_FONT_INFO = std::mem::zeroed();
            if GetCurrentConsoleFont(**handle, maximum_window.into(), &mut info) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(info.into())
            }
        }
    }

    /// Gets extended information about the console font.
    ///
    /// Wraps a call to [GetCurrentConsoleFontEx](https://docs.microsoft.com/en-us/windows/console/getcurrentconsolefontex).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole};
    ///
    /// let old_info = WinConsole::output().get_font_info_ex(false).unwrap();
    /// let mut new_info = old_info;
    /// new_info.font_weight = 800; //Bold font
    /// WinConsole::output().set_font_info_ex(new_info, false).unwrap();
    /// WinConsole::output().write_utf8("Hello World".as_bytes()).unwrap();
    /// ```
    pub fn get_font_info_ex(&self, maximum_window: bool) -> Result<ConsoleFontInfoEx> {
        let handle = self.get_handle();

        unsafe {
            let mut info: CONSOLE_FONT_INFOEX = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<ConsoleFontInfoEx>() as u32;

            let ptr: *mut CONSOLE_FONT_INFOEX = &mut info;

            if GetCurrentConsoleFontEx(**handle, maximum_window.into(), ptr) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(ConsoleFontInfoEx::from(&info))
            }
        }
    }

    /// Gets the current screen buffer info.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// Wraps a call to [GetConsoleScreenBufferInfo](https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfo).
    ///
    /// ```
    /// use win32console::console::{WinConsole, ConsoleTextAttribute};
    /// let info = WinConsole::output().get_screen_buffer_info().unwrap();
    /// ```
    pub fn get_screen_buffer_info(&self) -> Result<ConsoleScreenBufferInfo> {
        let handle = self.get_handle();

        unsafe {
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
            if GetConsoleScreenBufferInfo(**handle, &mut info) != 0 {
                Ok(ConsoleScreenBufferInfo::from(info))
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Gets extended information of the console screen buffer.
    ///
    /// Wraps a call to [GetConsoleScreenBufferInfoEx](https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfoex).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, ConsoleTextAttribute};
    ///
    /// let old_info = WinConsole::output().get_screen_buffer_info_ex().unwrap();
    /// let mut  new_info = old_info.clone();
    /// new_info.attributes = ConsoleTextAttribute::FOREGROUND_RED | ConsoleTextAttribute::FOREGROUND_INTENSITY;
    /// WinConsole::output().set_screen_buffer_info_ex(new_info); // Set foreground color red
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// WinConsole::output().set_screen_buffer_info_ex(old_info); // Restore old info
    /// ```
    pub fn get_screen_buffer_info_ex(&self) -> Result<ConsoleScreenBufferInfoEx> {
        let handle = self.get_handle();

        unsafe {
            let mut buffer_info: CONSOLE_SCREEN_BUFFER_INFOEX = std::mem::zeroed();
            buffer_info.cbSize = std::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFOEX>() as u32;

            if GetConsoleScreenBufferInfoEx(**handle, &mut buffer_info) != 0 {
                Ok(ConsoleScreenBufferInfoEx::from(buffer_info))
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Sets the extended console screen buffer information.
    ///
    /// Wraps a call to [SetConsoleScreenBufferInfoEx](https://docs.microsoft.com/en-us/windows/console/setconsolescreenbufferinfoex).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, ConsoleTextAttribute};
    ///
    /// let old_info = WinConsole::output().get_screen_buffer_info_ex().unwrap();
    /// let mut  new_info = old_info.clone();
    /// new_info.attributes = ConsoleTextAttribute::FOREGROUND_RED | ConsoleTextAttribute::FOREGROUND_INTENSITY;
    /// WinConsole::output().set_screen_buffer_info_ex(new_info); // Set foreground color red
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// WinConsole::output().set_screen_buffer_info_ex(old_info); // Restore old info
    /// ```
    pub fn set_screen_buffer_info_ex(&self, info: ConsoleScreenBufferInfoEx) -> Result<()> {
        let handle = self.get_handle();

        unsafe {
            let mut buffer_info = info.into();
            if SetConsoleScreenBufferInfoEx(**handle, &mut buffer_info) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Set the size of the console screen buffer.
    ///
    /// Wraps a call to [SetConsoleScreenBufferSize](https://docs.microsoft.com/en-us/windows/console/setconsolescreenbuffersize).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::coord::Coord;
    /// const WIDTH : i16 = 30;
    /// const HEIGHT : i16 = 40;
    ///
    /// WinConsole::output().set_screen_buffer_size(Coord::new(WIDTH, HEIGHT));
    /// ```
    pub fn set_screen_buffer_size(&self, size: Coord) -> Result<()> {
        let handle = self.get_handle();

        unsafe {
            if SetConsoleScreenBufferSize(**handle, size.into()) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Sets the current size and position of the console screen buffer window.
    ///
    /// - `absolute`: If this parameter is `TRUE`, the coordinates specify the new upper-left and lower-right corners of the window.
    /// If it is `FALSE`, the coordinates are relative to the current window-corner coordinates.
    /// - `window`: specifies the new upper-left and lower-right corners of the window.
    ///
    /// Wraps a call to [SetConsoleWindowInfo](https://docs.microsoft.com/en-us/windows/console/setconsolewindowinfo).
    ///
    /// # Remarks
    /// - The function may return an error when using the console of an IDE.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    /// - If the `window` parameter is too big.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::small_rect::SmallRect;
    /// let window = SmallRect::new(0, 0, 40, 50);
    /// WinConsole::output().set_window_info(true, &window);
    /// ```
    pub fn set_window_info(&self, absolute: bool, window: &SmallRect) -> Result<()> {
        let handle = self.get_handle();
        let small_rect: &SMALL_RECT = &(*window).into();

        unsafe {
            if SetConsoleWindowInfo(**handle, absolute.into(), small_rect) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Sets the position of the cursor. don't confuse with mouse cursor.
    ///
    /// Wraps a call to [SetConsoleCursorPosition](https://docs.microsoft.com/en-us/windows/console/setconsolecursorposition).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    ///
    /// fn remove_last(){
    ///        let pos = WinConsole::output().get_cursor_position().unwrap();
    ///        if pos.x > 0{
    ///            const WHITE_SPACE : &[u8; 1] = b" ";
    ///            let new_pos = pos.with_x(pos.x - 1);
    ///
    ///            // Move back, write a whitespace and move back again
    ///            // to remove the last written char
    ///            WinConsole::output().set_cursor_position(new_pos);
    ///            WinConsole::output().write_utf8(WHITE_SPACE);
    ///            WinConsole::output().set_cursor_position(new_pos);
    ///        }
    ///    }
    ///
    ///    // A simple alphanumeric reader from the std input
    ///    loop{
    ///        if let KeyEvent(event) = WinConsole::input().read_single_input().unwrap(){
    ///             // Only enter when the key is pressed down
    ///            if event.key_down{
    ///                // Only alphanumeric are allowed so any other is ignore
    ///                if event.u_char.is_ascii_alphanumeric() {
    ///                    let mut buf = [0];
    ///                    event.u_char.encode_utf8(&mut buf);
    ///                    // Write the character
    ///                    WinConsole::output().write_utf8(&buf);
    ///                }
    ///                else{
    ///                    match event.virtual_key_code{
    ///                        0x08 => { remove_last(); } // Remove last on backspace press
    ///                        0x1B => { break; }         // Exit when escape is press
    ///                        _ => {}
    ///                    }
    ///                }
    ///            }
    ///        }
    ///    }
    /// ```
    pub fn set_cursor_position(&self, coord: Coord) -> Result<()> {
        unsafe {
            let handle = self.get_handle();
            if SetConsoleCursorPosition(**handle, coord.into()) != 0 {
                Ok(())
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Gets the current position of the cursor. don't confuse with mouse cursor.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    ///
    /// fn remove_last(){
    ///        let pos = WinConsole::output().get_cursor_position().unwrap();
    ///        if pos.x > 0{
    ///            const WHITE_SPACE : &[u8; 1] = b" ";
    ///            let new_pos = pos.with_x(pos.x - 1);
    ///
    ///            // Move back, write a whitespace and move back again
    ///            // to remove the last written char
    ///            WinConsole::output().set_cursor_position(new_pos);
    ///            WinConsole::output().write_utf8(WHITE_SPACE);
    ///            WinConsole::output().set_cursor_position(new_pos);
    ///        }
    ///    }
    ///
    ///    // A simple alphanumeric reader from the std input
    ///    loop{
    ///        if let KeyEvent(event) = WinConsole::input().read_single_input().unwrap(){
    ///             // Only enter when the key is pressed down
    ///            if event.key_down{
    ///                // Only alphanumeric are allowed so any other is ignore
    ///                if event.u_char.is_ascii_alphanumeric() {
    ///                    let mut buf = [0];
    ///                    event.u_char.encode_utf8(&mut buf);
    ///                    // Write the character
    ///                    WinConsole::output().write_utf8(&buf);
    ///                }
    ///                else{
    ///                    match event.virtual_key_code{
    ///                        0x08 => { remove_last(); } // Remove last on backspace press
    ///                        0x1B => { break; }         // Exit when escape is press
    ///                        _ => {}
    ///                    }
    ///                }
    ///            }
    ///        }
    ///    }
    /// ```
    pub fn get_cursor_position(&self) -> Result<Coord> {
        self.get_screen_buffer_info()
            .map(|value| value.cursor_position)
    }

    /// Clears the content of the console screen buffer and set the cursor to (0, 0)
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// WinConsole::output().clear();
    /// ```
    pub fn clear(&self) -> Result<()> {
        // Gets the size of the current screen buffer
        let info = self.get_screen_buffer_info()?;
        let size = info.screen_buffer_size;
        let length: u32 = size.x as u32 * size.y as u32;

        // Fills the console with a whitespace
        self.fill_with_char(Coord::default(), length, ' ')?;

        // Fills with the current attribute.
        // TODO: Use 0 as attribute?
        self.fill_with_attribute(Coord::default(), length, info.attributes)?;

        // Set the cursor position to (0, 0)
        self.set_cursor_position(Coord::default())?;

        Ok(())
    }

    /// Fills the content of the console with the specified [`char`].
    ///
    /// Wraps a call to [FillConsoleOutputCharacterW](https://docs.microsoft.com/en-us/windows/console/fillconsoleoutputcharacter).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let current_pos = WinConsole::output().get_cursor_position().unwrap();
    /// WinConsole::output().fill_with_char(current_pos, 10, 'x').unwrap();
    /// ```
    pub fn fill_with_char(
        &self,
        start_location: Coord,
        cells_to_write: u32,
        value: char,
    ) -> Result<u32> {
        let handle = self.get_handle();
        let mut chars_written = 0;

        unsafe {
            if FillConsoleOutputCharacterW(
                **handle,
                value as u16,
                cells_to_write,
                start_location.into(),
                &mut chars_written,
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                Ok(chars_written)
            }
        }
    }

    /// Fills the content of the console with the specified attribute.
    ///
    /// Wraps a call to [FillConsoleOutputAttribute](https://docs.microsoft.com/en-us/windows/console/fillconsoleoutputattribute).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let len = 100;
    /// let current_pos = WinConsole::output().get_cursor_position().unwrap();
    /// WinConsole::output().fill_with_char(current_pos, len, ' ').unwrap();
    ///
    /// for i in 0..len{
    ///    let mut pos = current_pos.clone();
    ///    pos.x += i as i16;
    ///    let color : u16 = (16 << (i % 3)) as u16; // Apply colors to the characters
    ///    WinConsole::output().fill_with_attribute(pos, 1, color);
    ///}
    /// ```
    pub fn fill_with_attribute(
        &self,
        start_location: Coord,
        cells_to_write: u32,
        attribute: u16,
    ) -> Result<u32> {
        let handle = self.get_handle();
        let mut att_written = 0;

        unsafe {
            if FillConsoleOutputAttribute(
                **handle,
                attribute,
                cells_to_write,
                start_location.into(),
                &mut att_written,
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                Ok(att_written)
            }
        }
    }

    /// Sets the text attribute of the characters in the console.
    ///
    /// - `attribute`: the attributes to use, those attributes can be access using `ConsoleTextAttribute` struct.
    ///
    /// Wraps a call to [SetConsoleTextAttribute](https://docs.microsoft.com/en-us/windows/console/setconsoletextattribute).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, ConsoleTextAttribute};
    ///
    /// let old_attributes = WinConsole::output().get_text_attribute().unwrap();
    /// let new_attributes = ConsoleTextAttribute::BACKGROUND_BLUE;
    ///
    /// WinConsole::output().set_text_attribute(new_attributes);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// WinConsole::output().set_text_attribute(old_attributes);
    /// ```
    pub fn set_text_attribute(&self, attribute: u16) -> Result<()> {
        let handle = self.get_handle();

        unsafe {
            if SetConsoleTextAttribute(**handle, attribute) != 0 {
                Ok(())
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Gets the text attributes of the characters in the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::{WinConsole, ConsoleTextAttribute};
    ///
    /// let old_attributes = WinConsole::output().get_text_attribute().unwrap();
    /// let new_attributes = ConsoleTextAttribute::BACKGROUND_BLUE;
    ///
    /// WinConsole::output().set_text_attribute(new_attributes);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// WinConsole::output().set_text_attribute(old_attributes);
    /// ```
    pub fn get_text_attribute(&self) -> Result<u16> {
        Ok(self.get_screen_buffer_info()?.attributes)
    }

    /// Gets the largest size the console window can get.
    ///
    /// Wraps a call to [GetLargestConsoleWindowSize](https://docs.microsoft.com/en-us/windows/console/getlargestconsolewindowsize).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let max_size = WinConsole::output().get_largest_window_size().unwrap();
    /// ```
    pub fn get_largest_window_size(&self) -> Result<Coord> {
        let handle = self.get_handle();

        unsafe {
            let coord: Coord = GetLargestConsoleWindowSize(**handle).into();

            if coord == Coord::ZERO {
                Err(Error::last_os_error())
            } else {
                Ok(coord)
            }
        }
    }

    /// Gets the number of unread input events.
    ///
    /// Wraps a call to [GetNumberOfConsoleInputEvents](https://docs.microsoft.com/en-us/windows/console/getnumberofconsoleinputevents).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let unread_events = WinConsole::input().get_number_of_input_events().unwrap();
    /// ```
    pub fn get_number_of_input_events(&self) -> Result<usize> {
        let handle = self.get_handle();

        unsafe {
            let mut num_events = 0;
            if GetNumberOfConsoleInputEvents(**handle, &mut num_events) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(num_events as usize)
            }
        }
    }

    /// Gets the number of mouse buttons used for the mouse available for this console.
    ///
    /// Wraps a call to [GetNumberOfConsoleMouseButtons](https://docs.microsoft.com/en-us/windows/console/getnumberofconsolemousebuttons).
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let x = WinConsole::input().get_number_of_mouse_buttons().unwrap();
    /// let y = WinConsole::output().get_number_of_mouse_buttons().unwrap();
    /// assert_eq!(x, y);
    /// ```
    pub fn get_number_of_mouse_buttons(&self) -> Result<u32> {
        let mut num_buttons = 0;

        unsafe {
            if GetNumberOfConsoleMouseButtons(&mut num_buttons) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(num_buttons)
            }
        }
    }

    /// Moves a block of data in a screen buffer.
    /// The effects of the move can be limited by specifying a clipping rectangle,
    /// so the contents of the console screen buffer outside the clipping rectangle are unchanged.
    ///
    /// Wraps a call to [ScrollConsoleScreenBufferW](https://docs.microsoft.com/en-us/windows/console/scrollconsolescreenbuffer).
    ///
     /// # Errors
    /// - No documented errors.
    pub fn scroll_screen_buffer(&self,
                                scroll_rect: SmallRect,
                                clip_rect: Option<SmallRect>,
                                destination: Coord,
                                fill: CharInfo
    ) -> Result<()>{
        let handle = self.get_handle();
        let chi = &mut fill.into();
        let srect = &mut scroll_rect.into();
        let crect = match clip_rect{
            Some(r) => &mut r.into(),
            None => null_mut()
        };

        unsafe{
            if ScrollConsoleScreenBufferW(
                **handle,
                srect,
                crect,
                destination.into(),
                chi) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Reads a single event from the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    /// use win32console::console::WinConsole;
    ///
    /// loop{
    ///        // A simple alphanumeric reader from the std input
    ///        if let KeyEvent(event) = WinConsole::input().read_single_input().unwrap(){
    ///             // Only enter when the key is pressed down
    ///            if event.key_down{
    ///                // Only alphanumeric are allowed so any other is ignore
    ///                if !(event.u_char.is_ascii_alphanumeric()) {
    ///                    match event.virtual_key_code{
    ///                        0x1B => { break; }         // Exit when escape is press
    ///                        _ => {}
    ///                    }
    ///                }
    ///                 else {
    ///                    let mut buf = [0];
    ///                    event.u_char.encode_utf8(&mut buf);
    ///                    // Write the character
    ///                    WinConsole::output().write_utf8(&buf);
    ///                 }
    ///            }
    ///        }
    ///    }
    /// ```
    pub fn read_single_input(&self) -> Result<InputRecord> {
        unsafe {
            let mut record: InputRecord = std::mem::zeroed();
            let mut buf = slice::from_mut(&mut record);
            self.read_input(&mut buf)?;
            Ok(record)
        }
    }

    /// Reads input events from the console.
    ///
    /// - `buffer_size`: the size of the buffer that will store the events.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    /// let input_records = WinConsole::input().read_input_n(10).unwrap();
    ///
    /// let mut buf = String::new();
    /// for record in input_records{
    ///     if let KeyEvent(key) = record{
    ///         if key.key_down && key.u_char.is_ascii_alphanumeric(){
    ///             buf.push(key.u_char);
    ///         }
    ///     }
    /// }
    ///
    /// WinConsole::output().write_utf8(buf.as_bytes());
    /// ```
    pub fn read_input_n(&self, buffer_size: usize) -> Result<Vec<InputRecord>> {
        if buffer_size == 0 {
            return Ok(vec![]);
        }

        let mut buffer = vec![unsafe { std::mem::zeroed::<InputRecord>() }; buffer_size];

        self.read_input(buffer.as_mut_slice())?;
        Ok(buffer)
    }

    /// Fills the specified buffer with [`InputRecord`] from the console.
    ///
    /// Wraps a call to [ReadConsoleInputW](https://docs.microsoft.com/en-us/windows/console/readconsoleinput).
    ///
    /// # Returns
    /// The number of input events read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::structs::input_record::InputRecord;
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    ///
    /// let mut input_records : [InputRecord; 10] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// WinConsole::input().read_input(&mut input_records).unwrap();
    ///
    /// let mut buf = String::new();
    /// for record in input_records.iter(){
    ///     if let KeyEvent(key) = record{
    ///         if key.key_down && key.u_char.is_ascii_alphanumeric(){
    ///             buf.push(key.u_char);
    ///         }
    ///     }
    /// }
    ///
    /// WinConsole::output().write_utf8(buf.as_bytes());
    /// ```
    pub fn read_input(&self, records: &mut [InputRecord]) -> Result<usize> {
        if records.len() == 0 {
            return Ok(0);
        }

        let handle = self.get_handle();
        let num_records = records.len();
        let mut num_events = 0;

        let mut buf = vec![unsafe { std::mem::zeroed::<INPUT_RECORD>() }; num_records];

        unsafe {
            if ReadConsoleInputW(
                **handle,
                buf.as_mut_ptr(),
                num_records as u32,
                &mut num_events,
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                // Documentation specify that at least 1 event will be read.
                debug_assert!(num_events > 0);

                // Copies each of the read events to the destination buffer
                for i in 0..num_records {
                    records[i] = buf[i].into()
                }

                Ok(num_events as usize)
            }
        }
    }

    /// Fills the specified buffer with the unread [`InputRecord`] from the console.
    ///
    /// # Returns
    /// The number of input events read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// Wraps a call to [PeekConsoleInputW](https://docs.microsoft.com/en-us/windows/console/peekconsoleinput).
    ///
    /// # Example
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::structs::input_record::InputRecord;
    /// use win32console::console::WinConsole;
    /// use win32console::structs::input_record::InputRecord::KeyEvent;
    ///
    /// let mut input_records : [InputRecord; 10] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// WinConsole::input().peek_input(&mut input_records).unwrap();
    ///
    /// let mut buf = String::new();
    /// for record in input_records.iter(){
    ///     if let KeyEvent(key) = record{
    ///         if key.key_down && key.u_char.is_ascii_alphanumeric(){
    ///             buf.push(key.u_char);
    ///         }
    ///     }
    /// }
    ///
    /// WinConsole::output().write_utf8(buf.as_bytes());
    /// ```
    pub fn peek_input(&self, records: &mut [InputRecord]) -> Result<usize> {
        if records.len() == 0 {
            return Ok(0);
        }

        let handle = self.get_handle();
        let num_records = records.len();
        let mut num_events = 0;

        unsafe {
            let mut buf = iter::repeat_with(|| std::mem::zeroed::<INPUT_RECORD>())
                .take(num_records)
                .collect::<Vec<INPUT_RECORD>>();

            if PeekConsoleInputW(
                **handle,
                buf.as_mut_ptr(),
                num_records as u32,
                &mut num_events,
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                // Documentation specify that at least 1 event will be read.
                debug_assert!(num_events > 0);

                // Copies each of the read events to the destination buffer
                for i in 0..num_records {
                    records[i] = buf[i].into()
                }

                Ok(num_events as usize)
            }
        }
    }

    /// Reads a `String` from the standard input, followed by a newline.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    ///
    /// WinConsole::output().write_utf8("What's your name? ".as_bytes());
    /// let value = WinConsole::input().read_string().unwrap();
    /// WinConsole::output().write_utf8(format!("Hello {}", value).as_bytes());
    /// ```
    pub fn read_string(&self) -> Result<String> {
        // Used buffer size from:
        // https://source.dot.net/#System.Console/System/Console.cs,dac049f8d10df4a0
        const MAX_BUFFER_SIZE: usize = 4096;

        let mut buffer: [u16; MAX_BUFFER_SIZE] = unsafe { MaybeUninit::zeroed().assume_init() };
        let chars_read = self.read_utf16(&mut buffer)?;

        match String::from_utf16(buffer[..chars_read].as_ref()) {
            Ok(string) => Ok(string),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }

    /// Fills the given `u8` buffer with characters from the standard input.
    ///
    /// # Returns
    /// The number of characters read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::console::WinConsole;
    /// let mut buffer : [u8 ; 10] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// WinConsole::input().read_utf8(&mut buffer);
    /// ```
    pub fn read_utf8(&self, buffer: &mut [u8]) -> Result<usize> {
        if buffer.len() == 0 {
            return Ok(0);
        }

        let mut utf16_buffer = vec![u16::default(); buffer.len()];

        // Writes the read data to the 'utf16_buffer'.
        self.read_utf16(&mut utf16_buffer)?;
        let written = WinConsole::utf16_to_utf8(&utf16_buffer, buffer)?;
        Ok(written)
    }

    /// Fills the given `u16` buffer with characters from the standard input.
    ///
    /// Wraps a call to [ReadConsoleW](https://docs.microsoft.com/en-us/windows/console/readconsole).
    ///
    /// # Returns
    /// The number of characters read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::console::WinConsole;
    /// let mut buffer : [u16 ; 10] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// WinConsole::input().read_utf16(&mut buffer);
    /// ```
    pub fn read_utf16(&self, buffer: &mut [u16]) -> Result<usize> {
        if buffer.len() == 0 {
            return Ok(0);
        }

        // https://github.com/rust-lang/rust/blob/master/src/libstd/sys/windows/stdio.rs
        // https://stackoverflow.com/questions/43836040/win-api-readconsole
        const CTRL_Z: u16 = 0x1A;
        const CTRL_Z_MASK: u32 = (1 << CTRL_Z) as u32;

        let mut input_control = CONSOLE_READCONSOLE_CONTROL {
            nLength: std::mem::size_of::<CONSOLE_READCONSOLE_CONTROL>() as u32,
            nInitialChars: 0,
            dwCtrlWakeupMask: CTRL_Z_MASK,
            dwControlKeyState: 0,
        };

        let handle = self.get_handle();
        let mut chars_read = 0;

        if !WinConsole::is_console(&handle) {
            let mut data = match String::from_utf16(buffer) {
                Ok(string) => string,
                Err(e) => return Err(Error::new(std::io::ErrorKind::InvalidInput, e)),
            };

            unsafe {
                if ReadFile(
                    **handle,
                    data.as_mut_ptr() as *mut c_void,
                    buffer.len() as u32,
                    &mut chars_read,
                    null_mut(),
                ) == 0
                {
                    return Err(Error::last_os_error());
                }
            }

            return Ok(chars_read as usize);
        }

        unsafe {
            if ReadConsoleW(
                **handle,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut chars_read,
                &mut input_control,
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                if chars_read > 0 && buffer[chars_read as usize - 1] == CTRL_Z {
                    chars_read -= 1;
                }

                Ok(chars_read as usize)
            }
        }
    }

    /// Fills the given `u8` buffer with characters from the standard input using the specified
    /// console read control.
    ///
    /// - `control`: provides information used for a read operation as the number of chars
    /// to skip or the end signal.
    ///
    /// Wraps a call to [ReadConsoleA](https://docs.microsoft.com/en-us/windows/console/readconsole).
    ///
    /// # Returns
    /// The number of characters read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::console::WinConsole;
    /// use win32console::structs::console_read_control::ConsoleReadControl;
    ///
    /// const CTRL_Z: u8 = 26;
    /// const CTRL_Z_MASK: u32 = (1 << CTRL_Z) as u32;
    ///
    /// let control = ConsoleReadControl::new_with_mask(CTRL_Z_MASK);
    /// let mut buffer : [u8 ; 32] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// let mut len = WinConsole::input().read_utf8_with_control(&mut buffer, control).unwrap();
    ///
    /// // If the last character is the control signal we ignore it.
    /// if len > 0 && buffer[len - 1] == CTRL_Z{
    ///     len -= 1;
    /// }
    ///
    /// let string = String::from_utf8_lossy(&buffer[..len])
    ///                     .trim() // String terminated in newline
    ///                     .to_string();
    ///
    /// // buffer is terminated in '\r\n', assertion will fail when write 32 characters
    /// assert_eq!(len - 2, string.len());
    /// WinConsole::output().write_utf8(string.as_bytes());
    /// ```
    pub fn read_utf8_with_control(
        &self,
        buffer: &mut [u8],
        control: ConsoleReadControl,
    ) -> Result<usize> {
        if buffer.len() == 0 {
            return Ok(0);
        }

        let mut utf16_buffer = vec![u16::default(); buffer.len()];
        let written = self.read_utf16_with_control(utf16_buffer.as_mut_slice(), control)?;
        WinConsole::utf16_to_utf8(&utf16_buffer, buffer)?;
        Ok(written)
    }

    /// Fills the given `u16` buffer with characters from the standard input using the specified
    /// console read control.
    ///
    /// - `control`: provides information used for a read operation as the number of chars
    /// to skip or the end signal.
    ///
    /// Wraps a call to [ReadConsoleW](https://docs.microsoft.com/en-us/windows/console/readconsole).
    ///
    /// # Returns
    /// The number of characters read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::console::WinConsole;
    /// use win32console::structs::console_read_control::ConsoleReadControl;
    ///
    /// const CTRL_Z: u16 = 26;
    /// const CTRL_Z_MASK: u32 = (1 << CTRL_Z) as u32;
    ///
    /// let control = ConsoleReadControl::new_with_mask(CTRL_Z_MASK);
    /// let mut buffer : [u16 ; 32] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// let mut len = WinConsole::input().read_utf16_with_control(&mut buffer, control).unwrap();
    ///
    /// // If the last character is the control signal we ignore it.
    /// if len > 0 && buffer[len - 1] == CTRL_Z{
    ///     len -= 1;
    /// }
    ///
    /// let string = String::from_utf16_lossy(&buffer[..len])
    ///                     .trim() // String terminated in newline
    ///                     .to_string();
    ///
    /// // buffer is terminated in '\r\n', assertion will fail when write 32 characters
    /// assert_eq!(len - 2, string.len());
    /// WinConsole::output().write_utf8(string.as_bytes());
    /// ```
    pub fn read_utf16_with_control(
        &self,
        buffer: &mut [u16],
        control: ConsoleReadControl,
    ) -> Result<usize> {
        if buffer.len() == 0 {
            return Ok(0);
        }

        let mut input_control = control.into();
        let handle = self.get_handle();
        let mut chars_read = 0;

        if !WinConsole::is_console(&handle) {
            let mut data = match String::from_utf16(buffer) {
                Ok(string) => string,
                Err(e) => return Err(Error::new(std::io::ErrorKind::InvalidInput, e)),
            };

            unsafe {
                if ReadFile(
                    **handle,
                    data.as_mut_ptr() as *mut c_void,
                    buffer.len() as u32,
                    &mut chars_read,
                    null_mut(),
                ) == 0
                {
                    return Err(Error::last_os_error());
                }
            }

            return Ok(chars_read as usize);
        }

        unsafe {
            if ReadConsoleW(
                **handle,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut chars_read,
                &mut input_control,
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                Ok(chars_read as usize)
            }
        }
    }

    /// Reads character and color attribute data from a rectangular block of character cells in a console screen buffer,
    /// and the function writes the data to a rectangular block at a specified location in the destination buffer.
    ///
    /// Wraps a call to [ReadConsoleOutputW](https://docs.microsoft.com/en-us/windows/console/readconsoleoutput).
    pub fn read_char_buffer(&self, buffer_size: Coord, buffer_coord: Coord, read_region: &mut SmallRect) -> Result<Vec<CharInfo>>{
        let handle = self.get_handle();
        let length = buffer_size.x * buffer_size.y;
        let mut buffer = vec![unsafe{ std::mem::zeroed::<CHAR_INFO>() }; length as usize];
        let raw_rect = &mut (*read_region).into();

        unsafe{
            if ReadConsoleOutputW(
                **handle,
                buffer.as_mut_ptr(),
                buffer_size.into(),
                buffer_coord.into(),
                raw_rect) == 0{
                Err(Error::last_os_error())
            }
            else{
                let ret = buffer.iter()
                    .map(|c| (*c).into())
                    .collect::<Vec<CharInfo>>();

                *read_region = SmallRect::from(*raw_rect);
                Ok(ret)
            }
        }
    }

    /// Flushes the console input buffer. All input records currently in the input buffer are discarded.
    ///
    /// Wraps a call to [FlushConsoleInputBuffer](https://docs.microsoft.com/en-us/windows/console/flushconsoleinputbuffer).
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// WinConsole::input().flush_input();
    /// ```
    pub fn flush_input(&self) -> Result<()>{
        let handle = self.get_handle();

        unsafe {
            if FlushConsoleInputBuffer(**handle) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Writes the specified `u8` buffer of chars in the current cursor position of the console.
    ///
    /// Wraps a call to [WriteConsoleA](https://docs.microsoft.com/en-us/windows/console/writeconsole).
    ///
    /// # Returns
    /// The number of characters written.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// ```
    pub fn write_utf8(&self, data: &[u8]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }

        let handle = self.get_handle();
        let mut chars_written = 0;

        // If is being redirected write to the handle
        if !WinConsole::is_console(&handle) {
            let buf = match String::from_utf8(data.to_vec()) {
                Ok(string) => string,
                Err(e) => return Err(Error::new(std::io::ErrorKind::InvalidInput, e)),
            };

            unsafe {
                if WriteFile(
                    **handle,
                    buf.as_ptr() as *const c_void,
                    data.len() as u32,
                    &mut chars_written,
                    null_mut(),
                ) == 0
                {
                    return Err(Error::last_os_error());
                }
            }
            return Ok(data.len());
        }

        unsafe {
            if WriteConsoleA(
                **handle,
                data.as_ptr() as *const c_void,
                data.len() as u32,
                &mut chars_written,
                null_mut(),
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                assert_eq!(chars_written, data.len() as u32);
                Ok(chars_written as usize)
            }
        }
    }

    /// Writes the specified buffer of chars in the current cursor position of the console.
    ///
    /// Wraps a call to [WriteConsoleW](https://docs.microsoft.com/en-us/windows/console/writeconsole).
    ///
    /// # Returns
    /// The number of characters written.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// let x = "Hello World!".encode_utf16().collect::<Vec<u16>>();
    /// WinConsole::output().write_utf16(x.as_slice());
    /// ```
    pub fn write_utf16(&self, data: &[u16]) -> Result<usize> {
        if data.len() == 0 {
            return Ok(0);
        }

        let handle = self.get_handle();
        let mut chars_written = 0;

        // If is being redirected write to the handle
        if !WinConsole::is_console(&handle) {
            let buf = match String::from_utf16(data) {
                Ok(string) => string,
                Err(e) => return Err(Error::new(std::io::ErrorKind::InvalidInput, e)),
            };

            unsafe {
                if WriteFile(
                    **handle,
                    buf.as_ptr() as *const c_void,
                    data.len() as u32,
                    &mut chars_written,
                    null_mut(),
                ) == 0
                {
                    return Err(Error::last_os_error());
                }
            }
            return Ok(data.len());
        }

        unsafe {
            if WriteConsoleW(
                **handle,
                data.as_ptr() as *const c_void,
                data.len() as u32,
                &mut chars_written,
                null_mut(),
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                assert_eq!(chars_written, data.len() as u32);
                Ok(chars_written as usize)
            }
        }
    }

    /// Writes the given buffer of `CharInfo` into the screen buffer.
    ///
    /// Wraps a call to [WriteConsoleOutputW](https://docs.microsoft.com/en-us/windows/console/writeconsoleoutput).
    ///
    /// See also: [`https://www.randygaul.net/2011/11/16/windows-console-game-writing-to-the-console/`]
    ///
    /// - `buffer_size`: the size of the `buffer` in rows and columns.
    /// - `buffer_start`: the origin in the `buffer` where start to take the characters to write, typically (0,0).
    /// - `write_area`: Represents the screen buffer area to write to.
    ///
    /// # Remarks
    /// - This functions don't affect the cursor position.
    /// - If the `write_area` is outside the screen buffer no data is written.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::structs::coord::Coord;
    /// use win32console::console::WinConsole;
    /// use win32console::structs::char_info::CharInfo;
    /// use win32console::structs::small_rect::SmallRect;
    /// const WIDTH : usize = 40;
    /// const HEIGHT : usize = 30;
    ///
    /// let mut buffer = Vec::with_capacity(WIDTH * HEIGHT);
    /// let buffer_size = Coord::new(WIDTH as i16, HEIGHT as i16);
    /// let window = SmallRect::new(0, 0, (WIDTH - 1) as i16, (HEIGHT - 1) as i16);
    ///
    /// WinConsole::output().set_window_info(true, &window).unwrap();
    /// WinConsole::output().set_screen_buffer_size(buffer_size.clone()).unwrap();
    ///
    /// for i in 0..buffer.capacity(){
    ///    let char_info = CharInfo::new(' ', (16 << i % 3) as u16);
    ///     buffer.push(char_info);
    /// }
    ///
    /// WinConsole::output().write_char_buffer(buffer.as_ref(), buffer_size, Coord::ZERO, window).unwrap();
    /// ```
    pub fn write_char_buffer(
        &self,
        buffer: &[CharInfo],
        buffer_size: Coord,
        buffer_start: Coord,
        write_area: SmallRect,
    ) -> Result<()> {
        if buffer.len() == 0 {
            return Ok(());
        }

        let handle = self.get_handle();
        let write_area_raw: PSMALL_RECT = &mut write_area.into();

        let buf = buffer
            .iter()
            .map(|c| (*c).into())
            .collect::<Vec<CHAR_INFO>>();

        unsafe {
            if WriteConsoleOutputW(
                **handle,
                buf.as_ptr() as PCHAR_INFO,
                buffer_size.into(),
                buffer_start.into(),
                write_area_raw,
            ) == 0
            {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Checks if the handle is a handle to a console
    #[inline]
    fn is_console(handle: &Handle) -> bool {
        let mut mode = 0;
        unsafe { GetConsoleMode(**handle, &mut mode) != 0 }
    }

    /// Converts the content of the given utf16 buffer to utf8 and writes it to the
    /// destination buffer.
    fn utf16_to_utf8(source: &[u16], destination: &mut [u8]) -> Result<usize> {
        // The actual number of utf8 characters written to the destination buffer
        let mut written = 0;

        let utf16_iterator = source.iter().cloned();
        for chr in std::char::decode_utf16(utf16_iterator) {
            match chr {
                Ok(value) => {
                    value.encode_utf8(&mut destination[written..]);
                    written += value.len_utf8();
                }
                Err(e) => {
                    return Err(Error::new(ErrorKind::InvalidData, e));
                }
            }
        }

        Ok(written)
    }
}

// ConsoleColor methods
impl WinConsole {
    const FG_COLOR_MARK: u16 = 0xF;
    const BG_COLOR_MASK: u16 = 0xF0;

    /// Gets the foreground color of the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::console_color::ConsoleColor;
    /// let old_fgcolor = WinConsole::output().get_foreground_color().unwrap();
    /// let old_bgcolor = WinConsole::output().get_background_color().unwrap();
    ///
    /// WinConsole::output().set_foreground_color(ConsoleColor::Red);
    /// WinConsole::output().set_background_color(ConsoleColor::Yellow);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    ///
    /// // Restore colors
    /// WinConsole::output().set_foreground_color(old_fgcolor);
    /// WinConsole::output().set_background_color(old_bgcolor);
    /// ```
    pub fn get_foreground_color(&self) -> std::io::Result<ConsoleColor> {
        let attributes = self.get_text_attribute()?;
        Ok(
            ConsoleColor::try_from(attributes & WinConsole::FG_COLOR_MARK)
                .ok()
                .expect(format!("Invalid color value: {}", attributes).as_ref()),
        )
    }

    /// Gets the background color of the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::console_color::ConsoleColor;
    /// let old_fgcolor = WinConsole::output().get_foreground_color().unwrap();
    /// let old_bgcolor = WinConsole::output().get_background_color().unwrap();
    ///
    /// WinConsole::output().set_foreground_color(ConsoleColor::Black);
    /// WinConsole::output().set_background_color(ConsoleColor::White);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    ///
    /// // Restore colors
    /// WinConsole::output().set_foreground_color(old_fgcolor);
    /// WinConsole::output().set_background_color(old_bgcolor);
    /// ```
    pub fn get_background_color(&self) -> std::io::Result<ConsoleColor> {
        let attributes = self.get_text_attribute()? << 4;
        Ok(
            ConsoleColor::try_from(attributes & WinConsole::BG_COLOR_MASK)
                .ok()
                .expect(format!("Invalid color value: {}", attributes).as_ref()),
        )
    }

    /// Sets the foreground color of the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::console_color::ConsoleColor;
    /// let old_fgcolor = WinConsole::output().get_foreground_color().unwrap();
    /// let old_bgcolor = WinConsole::output().get_background_color().unwrap();
    ///
    /// WinConsole::output().set_foreground_color(ConsoleColor::Yellow);
    /// WinConsole::output().set_background_color(ConsoleColor::DarkMagenta);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    ///
    /// // Restore colors
    /// WinConsole::output().set_foreground_color(old_fgcolor);
    /// WinConsole::output().set_background_color(old_bgcolor);
    /// ```
    pub fn set_foreground_color(&self, color: ConsoleColor) -> std::io::Result<()> {
        let old_attributes = self.get_text_attribute()?;
        let new_attributes = (old_attributes & !(old_attributes & WinConsole::FG_COLOR_MARK))
            | color.as_foreground_color();
        self.set_text_attribute(new_attributes)
    }

    /// Sets the background color of the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Example
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::console_color::ConsoleColor;
    /// let old_fgcolor = WinConsole::output().get_foreground_color().unwrap();
    /// let old_bgcolor = WinConsole::output().get_background_color().unwrap();
    ///
    /// WinConsole::output().set_foreground_color(ConsoleColor::DarkBlue);
    /// WinConsole::output().set_background_color(ConsoleColor::Green);
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    ///
    /// // Restore colors
    /// WinConsole::output().set_foreground_color(old_fgcolor);
    /// WinConsole::output().set_background_color(old_bgcolor);
    /// ```
    pub fn set_background_color(&self, color: ConsoleColor) -> std::io::Result<()> {
        let old_attributes = self.get_text_attribute()?;
        let new_attributes = (old_attributes & !(old_attributes & WinConsole::BG_COLOR_MASK))
            | color.as_background_color();
        self.set_text_attribute(new_attributes)
    }
}
