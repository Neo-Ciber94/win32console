use std::{
    iter,
    io::{Error, ErrorKind, Result},
    slice,
    mem::{transmute, MaybeUninit},
    str,
    convert::TryFrom
};

use winapi::{
    shared::minwindef::{MAX_PATH},
    ctypes::c_void,
    _core::ptr::{null_mut},
    um::{
        wincontypes::{PCHAR_INFO, PSMALL_RECT},
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
        wincon::{
            FillConsoleOutputAttribute, FillConsoleOutputCharacterW, GetConsoleScreenBufferInfo,
            GetConsoleScreenBufferInfoEx, GetConsoleTitleW, GetCurrentConsoleFontEx,
            GetLargestConsoleWindowSize, GetNumberOfConsoleMouseButtons, PeekConsoleInputW,
            SetConsoleCursorPosition, SetConsoleScreenBufferInfoEx, SetConsoleTextAttribute,
            SetConsoleTitleW, SetCurrentConsoleFontEx, CONSOLE_FONT_INFOEX,
            INPUT_RECORD,
        },
        wincon::CONSOLE_SCREEN_BUFFER_INFOEX,
        consoleapi::{GetConsoleMode, GetNumberOfConsoleInputEvents, ReadConsoleInputW, ReadConsoleW, SetConsoleMode, WriteConsoleW, GetConsoleCP, GetConsoleOutputCP, AllocConsole},
        fileapi::{CreateFileW, WriteFile, OPEN_EXISTING, ReadFile},
        handleapi::INVALID_HANDLE_VALUE,
        processenv::{GetStdHandle, SetStdHandle},
        winbase::{STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
        wincon::GetConsoleOriginalTitleW,
        wincon::FreeConsole,
        wincon::AttachConsole,
        wincon::SetConsoleOutputCP,
        wincon::SetConsoleCP,
        wincon::CHAR_INFO,
        wincon::SetConsoleWindowInfo,
        wincon::SMALL_RECT,
        wincon::WriteConsoleOutputW,
        wincon::SetConsoleScreenBufferSize,
        wincon::CONSOLE_READCONSOLE_CONTROL,
        wincon::CONSOLE_SCREEN_BUFFER_INFO,
        wincon::{GetCurrentConsoleFont, CONSOLE_FONT_INFO}
    }
};

use crate::{
    structs::console_screen_buffer_info_ex::ConsoleScreenBufferInfoEx,
    structs::console_screen_buffer_info::{ConsoleScreenBufferInfo, SmallRect},
    structs::console_font_info_ex::ConsoleFontInfoEx,
    structs::coord::Coord,
    structs::handle::Handle,
    structs::input_record::InputRecord,
    structs::char_info::CharInfo,
    structs::console_color::ConsoleColor,
    structs::console_read_control::ConsoleReadControl,
    structs::console_font_info::ConsoleFontInfo
};
use winapi::um::consoleapi::WriteConsoleA;

/// Represents an access to the windows console of the current process and provides methods for
/// interact with it.
///
/// This struct wraps a function [Fn() -> Handle] that must return a new [Handle] on each call,
/// this ensure a valid handle even after a system call of [SetStdHandle].
///
/// # Examples
/// Basic usages:
/// ```
/// use win32console::console::WinConsole;
///
/// WinConsole::output().write_utf8("What's your name?".as_bytes()).unwrap();
/// let name = WinConsole::input().read().unwrap();
/// WinConsole::output().write_utf8(format!("Oh, Hello {}!", name.trim()).as_ref()).unwrap();
/// ```
pub struct WinConsole {
    /// A function that returns a handle
    handle_provider: Box<dyn Fn() -> Handle>,
}

/// Type of a console handle, you can use this enum to get a handle by calling: [WinConsole::get_std_handle(...)].
///
/// # Examples
///
/// Basic usage:
/// ```
/// use win32console::console::{WinConsole, HandleType};
///
/// let handle = WinConsole::get_std_handle(HandleType::Input).unwrap();
/// assert!(handle.is_valid());
/// ```
pub enum HandleType {
    /// Represents the [STD_INPUT_HANDLE].
    Input,
    /// Represents the [STD_OUTPUT_HANDLE].
    Output,
    /// Represents the [STD_ERROR_HANDLE].
    Error,
}

/// Represents where the font information will be retrieve.
pub enum FontInfoSource {
    // Font information will be retrieve from the windows maximum size.
    MaximumSize = 1,
    // Font information will be retrieve from the windows current size.
    CurrentSize = 0
}

/// Wraps constants values of the console modes.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/getconsolemode]
pub struct ConsoleMode;

/// Wraps constants values of the console text attributes.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/console-screen-buffers]
pub struct ConsoleTextAttribute;

impl ConsoleMode {
    /// CTRL+C is processed by the system and is not placed in the input buffer.
    /// If the input buffer is being read by [ReadFile] or [ReadConsole],
    /// other control keys are processed by the system and are not returned in the [ReadFile] or [ReadConsole] buffer
    pub const ENABLE_PROCESSED_INPUT: u32 = 0x0001;

    /// The ReadFile or ReadConsole function returns only when a carriage return character is read.
    /// If this mode is disabled, the functions return when one or more characters are available.
    pub const ENABLE_LINE_INPUT: u32 = 0x0002;

    /// Characters read by the [ReadFile] or [ReadConsole] function are written to the active screen buffer as they are read.
    /// This mode can be used only if the [ENABLE_LINE_INPUT] mode is also enabled.
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

    /// Required to enable or disable extended flags. See [ENABLE_INSERT_MODE] and [ENABLE_QUICK_EDIT_MODE].
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolemode#parameters]
    pub const ENABLE_EXTENDED_FLAGS: u32 = 0x0080;

    /// Setting this flag directs the Virtual Terminal processing engine to convert user input received by the console window
    /// into Console Virtual Terminal Sequences that can be retrieved by a supporting application
    /// through [ReadFile] or [ReadConsole] functions.
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
    pub const COMMON_LVB_TRAILING_BYTE : u16 = 0x0200;
    /// Top horizontal
    pub const COMMON_LVB_GRID_HORIZONTAL : u16 = 0x0400;
    /// Left vertical.
    pub const COMMON_LVB_GRID_LVERTICAL : u16 = 0x0800;
    /// Right vertical.
    pub const COMMON_LVB_GRID_RVERTICAL : u16 = 0x1000;
    /// Reverse foreground and background attribute.
    pub const COMMON_LVB_REVERSE_VIDEO : u16 = 0x4000;
    /// Underscore.
    pub const COMMON_LVB_UNDERSCORE : u16 = 0x8000;
}

impl Into<u32> for HandleType {
    fn into(self) -> u32 {
        match self {
            HandleType::Input => STD_INPUT_HANDLE,
            HandleType::Output => STD_OUTPUT_HANDLE,
            HandleType::Error => STD_ERROR_HANDLE,
        }
    }
}

// Get console handle associative methods
impl WinConsole {
    /// Gets the specified handle by type.
    ///
    /// Wraps a call to [GetStdHandle]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getstdhandle]
    pub fn get_std_handle(handle_type: HandleType) -> Result<Handle> {
        unsafe {
            let raw_handle = GetStdHandle(handle_type.into());
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
    /// Wraps a call to [SetStdHandle]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setstdhandle]
    pub fn set_std_handle(handle_type: HandleType, handle: Handle) -> Result<()> {
        unsafe {
            if SetStdHandle(handle_type.into(), *handle) == 0 {
                return Err(Error::last_os_error());
            }

            Ok(())
        }
    }

    /// Creates a Handle to the standard input file [CONIN$], if the input
    /// is being redirected the value returned by [get_std_handle] cannot be used
    /// in functions that requires the console handle, but the returned [Handle]
    /// of this method can be used even if the input is being redirected.
    ///
    /// More info about console handles: [https://docs.microsoft.com/en-us/windows/console/console-handles?redirectedfrom=MSDN]
    ///
    /// Wraps a call to [CreateFileW].
    /// link: [https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew]
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

    /// Creates a Handle to the standard output file [CONOUT$], if the input
    /// is being redirected the value returned by [get_std_handle] cannot be used
    /// in functions that requires the console handle, but the returned [Handle]
    /// of this method can be used even if the output is being redirected.
    ///
    /// More info about console handles: [https://docs.microsoft.com/en-us/windows/console/console-handles?redirectedfrom=MSDN]
    ///
    /// Wraps a call to [CreateFileW].
    /// link: [https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew]
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
    /// Gets a console with the [STD_INPUT_HANDLE].
    ///
    /// # Examples
    ///
    /// Basic usages:
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::input();
    /// // Write using the input
    /// ```
    pub fn input() -> WinConsole {
        #[inline]
        fn get_input_handle() -> Handle {
            WinConsole::get_std_handle(HandleType::Input).expect("Invalid handle")
        }

        WinConsole {
            handle_provider: Box::new(get_input_handle),
        }
    }

    /// Gets a console with the [STD_OUTPUT_HANDLE].
    ///
    /// # Examples
    ///
    /// Basic usages:
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::output();
    /// // Read using the output
    /// ```
    pub fn output() -> WinConsole {
        #[inline]
        fn get_output_handle() -> Handle {
            WinConsole::get_std_handle(HandleType::Output).expect("Invalid handle")
        }

        WinConsole {
            handle_provider: Box::new(get_output_handle),
        }
    }

    /// Gets a console with current input handle.
    /// The handle will be always the current input handle even is the input is being redirected.
    ///
    /// # Examples
    ///
    /// Basic usages:
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::current_input();
    /// // Write using the input
    /// ```
    pub fn current_input() -> WinConsole {
        #[inline]
        fn get_input_handle() -> Handle {
            WinConsole::get_current_input_handle().expect("Invalid handle")
        }

        WinConsole {
            handle_provider: Box::new(get_input_handle),
        }
    }

    /// Gets a console with the current output handle.
    /// The handle will be always the current input handle even is the output is being redirected.
    ///
    /// # Examples
    ///
    /// Basic usages:
    /// ```
    /// use win32console::console::WinConsole;
    /// let console = WinConsole::current_output();
    /// // Read using the output
    /// ```
    pub fn current_output() -> WinConsole {
        #[inline]
        fn get_output_handle() -> Handle {
            WinConsole::get_current_output_handle().expect("Invalid handle")
        }

        WinConsole {
            handle_provider: Box::new(get_output_handle),
        }
    }
}

// Public methods
impl WinConsole {
    // Associative methods

    /// Allocates a new console for the calling process.
    ///
    /// # Errors
    /// - If the calling process have a console attached, `free_console` should be called first.
    pub fn alloc_console() -> Result<()>{
        unsafe{
            if AllocConsole() == 0{
                Err(Error::last_os_error())
            }
            else{
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
    /// # Errors
    /// - If the calling process is already attached to a console.
    /// - If the specified process does not have a console.
    /// - If the specified process does not exist.
    pub fn attach_console(process_id: u32) -> Result<()>{
        unsafe{
            if AttachConsole(process_id) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Detaches the calling process from its console.
    ///
    /// # Errors
    /// - If the calling process is not already attached to a console.
    pub fn free_console() -> Result<()>{
        unsafe{
            if FreeConsole() == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Sets the title of the current console.
///
/// Wraps a call to [SetConsoleTitle]
/// link: [https://docs.microsoft.com/en-us/windows/console/setconsoletitle]
///
/// # Errors
/// - No documented errors.
///
/// # Examples
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
    /// Wraps a call to [GetConsoleTitle]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsoletitle]
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Examples
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
        let mut buffer: [u16; MAX_PATH as usize] = unsafe{ MaybeUninit::zeroed().assume_init() };

        unsafe {
            let length = GetConsoleTitleW(buffer.as_mut_ptr(), MAX_PATH as u32) as usize;

            if length == 0 {
                Err(Error::last_os_error())
            } else {
                match String::from_utf16(&buffer){
                    Ok(string) => Ok(string),
                    Err(e) => {
                        Err(Error::new(ErrorKind::InvalidData, e))
                    }
                }
            }
        }
    }

    /// Retrieves the original title for the current console window.
    ///
    /// # Errors
    /// - If f the buffer is not large enough to store the title.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::WinConsole;
    /// let title = WinConsole::get_original_title().unwrap();
    /// WinConsole::output().write_utf8(title.as_bytes());
    /// ```
    pub fn get_original_title() -> Result<String>{
        let mut buffer: [u16; MAX_PATH as usize] = unsafe{ MaybeUninit::zeroed().assume_init() };

        unsafe{
            if GetConsoleOriginalTitleW(buffer.as_mut_ptr(), buffer.len() as u32) == 0{
                Err(Error::last_os_error())
            }
            else{
                match String::from_utf16(&buffer){
                    Ok(string) => Ok(string),
                    Err(e) => {
                        Err(Error::new(ErrorKind::InvalidData, e))
                    }
                }
            }
        }
    }

    /// Gets the input code page used by the console associated with the calling process.
    /// A console uses its input code page to translate keyboard input into the corresponding character value.
    ///
    /// See code pages: [https://docs.microsoft.com/en-us/windows/win32/intl/code-page-identifiers?redirectedfrom=MSDN]
    ///
    /// Wraps a call to [GetConsoleCP]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsolecp]
    pub fn get_input_code_page() -> Result<u32>{
        unsafe{
            let code_page = GetConsoleCP();
            if code_page == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(code_page)
            }
        }
    }

    /// Gets the output code page used by the console associated with the calling process.
    /// A console uses its output code page to translate the character values written by the various output
    /// functions into the images displayed in the console window.
    ///
    /// See code pages: [https://docs.microsoft.com/en-us/windows/win32/intl/code-page-identifiers?redirectedfrom=MSDN]
    ///
    /// Wraps a call to [GetConsoleOutputCP]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsoleoutputcp]
    pub fn get_output_code_page() -> Result<u32>{
        unsafe{
            let code_page = GetConsoleOutputCP();
            if code_page == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(code_page)
            }
        }
    }

    /// Sets the input code page used by the console associated with the calling process.
    /// A console uses its input code page to translate keyboard input into the corresponding character value.
    ///
    /// Wraps a call to [SetConsoleCP]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolecp]
    pub fn set_input_code(code_page: u32) -> Result<()>{
        unsafe{
            if SetConsoleCP(code_page) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Sets the output code page used by the console associated with the calling process.
    /// A console uses its output code page to translate the character values written by the various output functions
    /// into the images displayed in the console window.
    ///
    /// Wraps a call to [SetConsoleOutputCP]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsoleoutputcp]
    pub fn set_output_code(code_page: u32) -> Result<()>{
        unsafe{
            if SetConsoleOutputCP(code_page) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    // Instance methods

    /// Gets the handle used for this console, which will be provided by the `handle_provider`.
    pub fn get_handle(&self) -> Handle {
        (&self.handle_provider)()
    }

    /// Sets extended information about the console font.
    /// This function change the font into of all the current values in the console.
    ///
    /// Wraps a call to [SetCurrentConsoleFontEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setcurrentconsolefontex]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::{WinConsole, FontInfoSource};
    ///
    /// let old_info = WinConsole::output().get_font_info_ex(FontInfoSource::CurrentSize).unwrap();
    /// let mut new_info = old_info;
    /// new_info.font_weight = 800; //Bold font
    /// WinConsole::output().set_font_info_ex(new_info, FontInfoSource::CurrentSize).unwrap();
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
    pub fn set_font_info_ex(&self, info: ConsoleFontInfoEx, source: FontInfoSource) -> Result<()> {
        let handle = self.get_handle();
        let mut info = info.into();

        unsafe {
            if SetCurrentConsoleFontEx(*handle, source as i32, &mut info) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Gets information about the console font.
    ///
    /// Wraps a call to [GetCurrentConsoleFont]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getcurrentconsolefont]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::{WinConsole, FontInfoSource};    ///
    /// let info = WinConsole::output().get_font_info(FontInfoSource::CurrentSize).unwrap();
    /// ```
    pub fn get_font_info(&self, source: FontInfoSource) -> Result<ConsoleFontInfo>{
        let handle = self.get_handle();

        unsafe{
            let mut info : CONSOLE_FONT_INFO = std::mem::zeroed();
            if GetCurrentConsoleFont(*handle, source as i32, &mut info) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(info.into())
            }
        }
    }

    /// Gets extended information about the console font.
    ///
    /// Wraps a call to [GetCurrentConsoleFontEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getcurrentconsolefontex]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::{WinConsole, FontInfoSource};
    ///
    /// let old_info = WinConsole::output().get_font_info_ex(FontInfoSource::CurrentSize).unwrap();
    /// let mut new_info = old_info;
    /// new_info.font_weight = 800; //Bold font
    /// WinConsole::output().set_font_info_ex(new_info, FontInfoSource::CurrentSize).unwrap();
    /// WinConsole::output().write_utf8("Hello World".as_bytes()).unwrap();
    /// ```
    pub fn get_font_info_ex(&self, source: FontInfoSource) -> Result<ConsoleFontInfoEx> {
        let handle = self.get_handle();

        unsafe {
            let mut info: CONSOLE_FONT_INFOEX = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<ConsoleFontInfoEx>() as u32;

            let ptr: *mut CONSOLE_FONT_INFOEX = &mut info;

            if GetCurrentConsoleFontEx(*handle, source as i32, ptr) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(ConsoleFontInfoEx::from(&info))
            }
        }
    }

    /// Gets the current mode of the console
    ///
    /// Wraps a call to [GetConsoleMode]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsolemode]
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::{WinConsole, ConsoleMode};
    ///
    /// let old_mode = WinConsole::input().get_mode().unwrap();
    /// let new_mode = ConsoleMode::ENABLE_PROCESSED_INPUT | ConsoleMode::ENABLE_LINE_INPUT;
    /// // We change the input mode so the characters are not displayed
    /// WinConsole::input().set_mode(new_mode);
    ///
    /// let value = WinConsole::input().read().unwrap(); // Don't will be displayed due new mode
    /// WinConsole::output().write_utf8(value.as_bytes());
    /// WinConsole::input().set_mode(old_mode); // Reset the mode
    /// ```
    pub fn get_mode(&self) -> Result<u32> {
        let handle = self.get_handle();
        let mut mode = 0;

        unsafe {
            if GetConsoleMode(*handle, &mut mode) != 0 {
                Ok(mode)
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Sets the current mode of the console
    ///
    /// Wraps a call to [GetConsoleMode]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolemode]
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::{WinConsole, ConsoleMode};
    ///
    /// let old_mode = WinConsole::input().get_mode().unwrap();
    /// let new_mode = ConsoleMode::ENABLE_PROCESSED_INPUT | ConsoleMode::ENABLE_LINE_INPUT;
    /// // We change the input mode so the characters are not displayed
    /// WinConsole::input().set_mode(new_mode);
    ///
    /// let value = WinConsole::input().read().unwrap(); // Don't will be displayed due new mode
    /// WinConsole::output().write_utf8(value.as_bytes());
    /// WinConsole::input().set_mode(old_mode); // Reset the mode
    /// ```
    pub fn set_mode(&self, mode: u32) -> Result<()> {
        let handle = self.get_handle();

        unsafe {
            if SetConsoleMode(*handle, mode) != 0 {
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

    /// Gets the current screen buffer info.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// Wraps a call to [GetConsoleScreenBufferInfo]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfo]
    ///
    /// ```
    /// use win32console::console::{WinConsole, ConsoleTextAttribute};
    /// let info = WinConsole::output().get_screen_buffer_info().unwrap();
    /// ```
    pub fn get_screen_buffer_info(&self) -> Result<ConsoleScreenBufferInfo> {
        let handle = self.get_handle();

        unsafe {
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
            if GetConsoleScreenBufferInfo(*handle, &mut info) != 0 {
                Ok(ConsoleScreenBufferInfo::from(info))
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Gets extended information of the console screen buffer.
    ///
    /// Wraps a call to [GetConsoleScreenBufferInfoEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfoex]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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

            if GetConsoleScreenBufferInfoEx(*handle, &mut buffer_info) != 0 {
                Ok(ConsoleScreenBufferInfoEx::from(buffer_info))
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Sets the extended console screen buffer information.
    ///
    /// Wraps a call to [SetConsoleScreenBufferInfoEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolescreenbufferinfoex]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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
            if SetConsoleScreenBufferInfoEx(*handle, &mut buffer_info) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Set the size of the console screen buffer.
    ///
    /// Wraps a call to [SetConsoleScreenBufferSize].
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolescreenbuffersize]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::WinConsole;
    /// use win32console::structs::coord::Coord;
    /// const WIDTH : i16 = 30;
    /// const HEIGHT : i16 = 40;
    ///
    /// WinConsole::output().set_screen_buffer_size(Coord::new(WIDTH, HEIGHT));
    /// ```
    pub fn set_screen_buffer_size(&self, size: Coord) -> Result<()>{
        let handle = self.get_handle();

        unsafe{
            if SetConsoleScreenBufferSize(*handle, size.into()) == 0{
                Err(Error::last_os_error())
            }
            else{
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
    /// Wraps a call to [SetConsoleWindowInfo]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolewindowinfo]
    ///
    /// # Remarks
    /// - The function may return an error when using the console of an IDE.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    /// - If the `window` parameter is too big.
    ///
    /// # Examples
    /// ```
    /// use win32console::structs::console_screen_buffer_info::SmallRect;
    /// use win32console::console::WinConsole;
    /// let window = SmallRect::new(0, 0, 40, 50);
    /// WinConsole::output().set_window_info(true, &window);
    /// ```
    pub fn set_window_info(&self, absolute: bool, window: &SmallRect) -> Result<()>{
        let handle = self.get_handle();
        let mut small_rect: SMALL_RECT = (*window).into();
        let small_rect_ptr = &mut small_rect as PSMALL_RECT;

        unsafe{
            if SetConsoleWindowInfo(*handle, absolute.into(), small_rect_ptr) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Sets the position of the cursor. don't confuse with mouse cursor.
    ///
    /// Wraps a call to [SetConsoleCursorPosition]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolecursorposition]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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
            if SetConsoleCursorPosition(*handle, transmute(coord)) != 0 {
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
    /// # Examples
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

    /// Fills the content of the console with the specified [char].
    ///
    /// Wraps a call to [FillConsoleOutputCharacterW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/fillconsoleoutputcharacter]
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
                *handle,
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
    /// Wraps a call to [FillConsoleOutputAttribute]
    /// link: [https://docs.microsoft.com/en-us/windows/console/fillconsoleoutputattribute]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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
                *handle,
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
    /// Wraps a call to [SetConsoleTextAttribute]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsoletextattribute]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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
            if SetConsoleTextAttribute(*handle, attribute) != 0 {
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
    /// # Examples
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
    /// Wraps a call to [GetLargestConsoleWindowSize]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getlargestconsolewindowsize]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::WinConsole;
    /// let max_size = WinConsole::output().get_largest_window_size().unwrap();
    /// ```
    pub fn get_largest_window_size(&self) -> Result<Coord> {
        let handle = self.get_handle();

        unsafe {
            let coord: Coord = GetLargestConsoleWindowSize(*handle).into();

            if coord == Coord::ZERO {
                Err(Error::last_os_error())
            } else {
                Ok(coord)
            }
        }
    }

    /// Gets the number of unread input events.
    ///
    /// Wraps a call to [GetNumberOfConsoleInputEvents]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getnumberofconsoleinputevents]
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::WinConsole;
    /// let unread_events = WinConsole::input().get_number_of_input_events().unwrap();
    /// ```
    pub fn get_number_of_input_events(&self) -> Result<usize> {
        let handle = self.get_handle();

        unsafe {
            let mut num_events = 0;
            if GetNumberOfConsoleInputEvents(*handle, &mut num_events) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(num_events as usize)
            }
        }
    }

    /// Gets the number of mouse buttons used for the mouse available for this console.
    ///
    /// Wraps a call to [GetNumberOfConsoleMouseButtons]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getnumberofconsolemousebuttons]
    ///
    /// # Errors
    /// - No documented errors.
    ///
    /// # Examples:
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

    /// Reads a single event from the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Examples
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
    /// # Examples
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

        let mut buffer: Vec<InputRecord> =
            iter::repeat_with(unsafe { || std::mem::zeroed::<InputRecord>() })
                .take(buffer_size)
                .collect();

        self.read_input(buffer.as_mut_slice())?;
        Ok(buffer)
    }

    /// Fills the specified buffer with [InputRecord] from the console.
    ///
    /// Wraps a call to [ReadConsoleInputW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/readconsoleinput]
    ///
    /// # Returns
    /// The number of input events read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Examples
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
        let handle = self.get_handle();
        let num_records = records.len();
        let mut num_events = 0;

        unsafe {
            let mut buf = iter::repeat_with(|| std::mem::zeroed::<INPUT_RECORD>())
                .take(num_records)
                .collect::<Vec<INPUT_RECORD>>();

            if ReadConsoleInputW(
                *handle,
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

    /// Fills the specified buffer with the unread [InputRecord] from the console.
    ///
    /// # Returns
    /// The number of input events read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// Wraps a call to [PeekConsoleInputW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/peekconsoleinput]
    ///
    /// # Examples
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
        let handle = self.get_handle();
        let num_records = records.len();
        let mut num_events = 0;

        unsafe {
            let mut buf = iter::repeat_with(|| std::mem::zeroed::<INPUT_RECORD>())
                .take(num_records)
                .collect::<Vec<INPUT_RECORD>>();

            if PeekConsoleInputW(
                *handle,
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
    /// # Examples
    /// ```
    /// use win32console::console::WinConsole;
    ///
    /// WinConsole::output().write_utf8("What's your name? ".as_bytes());
    /// let value = WinConsole::input().read().unwrap();
    /// WinConsole::output().write_utf8(format!("Hello {}", value).as_bytes());
    /// ```
    pub fn read(&self) -> Result<String> {
        // Used buffer size from:
        // https://source.dot.net/#System.Console/System/Console.cs,dac049f8d10df4a0
        const MAX_BUFFER_SIZE: usize = 4096;

        let mut buffer: [u16; MAX_BUFFER_SIZE] = unsafe { MaybeUninit::zeroed().assume_init() };
        let chars_read = self.read_utf16(&mut buffer)?;

        match String::from_utf16(buffer[..chars_read].as_ref()) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }

    /// Fills the given `[u8]` buffer with characters from the standard input.
    ///
    /// # Returns
    /// The number of characters read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Examples
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::console::WinConsole;
    /// let mut buffer : [u8 ; 10] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// WinConsole::input().read_utf8(&mut buffer);
    /// ```
    pub fn read_utf8(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut utf16_buffer = iter::repeat_with(u16::default)
            .take(buffer.len())
            .collect::<Vec<u16>>();

        // Writes the read data to the 'utf16_buffer'.
        self.read_utf16(&mut utf16_buffer)?;

        let mut written = 0;
        for chr in std::char::decode_utf16(utf16_buffer) {
            match chr {
                Ok(chr) => {
                    chr.encode_utf8(&mut buffer[written..]);
                    written += 1;
                }
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "utf16 string not supported",
                    ));
                }
            }
        }

        Ok(written)
    }

    /// Fills the given `[u16]` buffer with characters from the standard input.
    ///
    /// Wraps a call to [ReadConsoleW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/readconsole]
    ///
    /// # Returns
    /// The number of characters read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Examples
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::console::WinConsole;
    /// let mut buffer : [u16 ; 10] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// WinConsole::input().read_utf16(&mut buffer);
    /// ```
    pub fn read_utf16(&self, buffer: &mut [u16]) -> Result<usize> {
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

        if !WinConsole::is_console(&handle){
            let mut data = match String::from_utf16(buffer) {
                Ok(string) => string,
                Err(e) => return Err(Error::new(std::io::ErrorKind::InvalidInput, e)),
            };

            unsafe{
                if ReadFile(
                    *handle,
                    data.as_mut_ptr() as *mut c_void,
                    buffer.len() as u32,
                    &mut chars_read,
                    null_mut()) == 0{
                    return Err(Error::last_os_error());
                }
            }

            return Ok(chars_read as usize);
        }

        unsafe {
            if ReadConsoleW(
                *handle,
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

    /// Fills the given `[u16]` buffer with characters from the standard input using the specified
    /// console read control.
    ///
    /// - `control`: provides information used for a read operation as the number of chars
    /// to skip or the end signal.
    ///
    /// Wraps a call to [ReadConsoleW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/readconsole]
    ///
    /// # Returns
    /// The number of characters read.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an output handle: `WinConsole::output()`,
    /// the function should be called using `WinConsole::input()` or a valid input handle.
    ///
    /// # Examples
    /// ```
    /// use std::mem::MaybeUninit;
    /// use win32console::console::WinConsole;
    /// let mut buffer : [u16 ; 10] = unsafe { MaybeUninit::zeroed().assume_init() };
    /// WinConsole::input().read_utf16(&mut buffer);
    /// ```
    pub fn read_utf16_with_control(&self, buffer: &mut [u16], control: ConsoleReadControl) -> Result<usize>{
        let mut input_control = control.into();
        let handle = self.get_handle();
        let mut chars_read = 0;

        if !WinConsole::is_console(&handle){
            let mut data = match String::from_utf16(buffer) {
                Ok(string) => string,
                Err(e) => return Err(Error::new(std::io::ErrorKind::InvalidInput, e)),
            };

            unsafe{
                if ReadFile(
                    *handle,
                    data.as_mut_ptr() as *mut c_void,
                    buffer.len() as u32,
                    &mut chars_read,
                    null_mut()) == 0{
                    return Err(Error::last_os_error());
                }
            }

            return Ok(chars_read as usize);
        }

        unsafe {
            if ReadConsoleW(
                *handle,
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

    /// Writes the specified `[u8]` buffer of chars in the current cursor position of the console.
    ///
    /// Wraps a call to [WriteConsoleA]
    /// link: [https://docs.microsoft.com/en-us/windows/console/writeconsole]
    ///
    /// # Returns
    /// The number of characters written.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::WinConsole;
    /// WinConsole::output().write_utf8("Hello World!".as_bytes());
    /// ```
    pub fn write_utf8(&self, data: &[u8]) -> Result<usize> {
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
                    *handle,
                    buf.as_ptr() as *const c_void,
                    data.len() as u32,
                    &mut chars_written,
                    null_mut()) == 0{
                    return Err(Error::last_os_error());
                }
            }
            return Ok(data.len());
        }

        unsafe {
            if WriteConsoleA(
                *handle,
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
    /// Wraps a call to [WriteConsoleW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/writeconsole]
    ///
    /// # Returns
    /// The number of characters written.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
    /// ```
    /// use win32console::console::WinConsole;
    /// let x = "Hello World!".encode_utf16().collect::<Vec<u16>>();
    /// WinConsole::output().write_utf16(x.as_slice());
    /// ```
    pub fn write_utf16(&self, data: &[u16]) -> Result<usize> {
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
                    *handle,
                    buf.as_ptr() as *const c_void,
                    data.len() as u32,
                    &mut chars_written,
                    null_mut()) == 0{
                    return Err(Error::last_os_error());
                }
            }
            return Ok(data.len());
        }

        unsafe {
            if WriteConsoleW(
                *handle,
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
    /// Wraps a call to [WriteConsoleOutputW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/writeconsoleoutput]
    /// see also: [https://www.randygaul.net/2011/11/16/windows-console-game-writing-to-the-console/]
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
    /// # Examples
    /// ```
    /// use win32console::structs::coord::Coord;
    /// use win32console::structs::console_screen_buffer_info::SmallRect;
    /// use win32console::console::WinConsole;
    /// use win32console::structs::char_info::CharInfo;
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
    pub fn write_char_buffer(&self, buffer: &[CharInfo], buffer_size: Coord, buffer_start: Coord, write_area: SmallRect) -> Result<()>{
        let handle = self.get_handle();
        let write_area_raw: PSMALL_RECT = &mut write_area.into();

        let buf = buffer.iter()
            .map(|c| (*c).into())
            .collect::<Vec<CHAR_INFO>>();

        unsafe{
            if WriteConsoleOutputW(
                *handle,
                buf.as_ptr() as PCHAR_INFO,
                buffer_size.into(),
                buffer_start.into(),
                write_area_raw) == 0{
                Err(Error::last_os_error())
            }
            else{
                Ok(())
            }
        }
    }

    /// Checks if the handle is a handle to a console
    #[inline]
    pub fn is_console(handle: &Handle) -> bool {
        let mut mode = 0;
        unsafe { GetConsoleMode(**handle, &mut mode) != 0 }
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
    /// # Examples
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
        Ok(ConsoleColor::try_from(attributes & WinConsole::FG_COLOR_MARK)
            .ok()
            .expect(format!("Invalid color value: {}", attributes).as_ref()))
    }

    /// Gets the background color of the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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
        Ok(ConsoleColor::try_from(attributes & WinConsole::BG_COLOR_MASK)
            .ok()
            .expect(format!("Invalid color value: {}", attributes).as_ref()))
    }

    /// Sets the foreground color of the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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
        let new_attributes =
            (old_attributes & !(old_attributes & WinConsole::FG_COLOR_MARK)) | color.as_foreground_color();
        self.set_text_attribute(new_attributes)
    }

    /// Sets the background color of the console.
    ///
    /// # Errors
    /// - If the handle is an invalid handle or an input handle: `WinConsole::input()`,
    /// the function should be called using `WinConsole::output()` or a valid output handle.
    ///
    /// # Examples
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
        let new_attributes =
            (old_attributes & !(old_attributes & WinConsole::BG_COLOR_MASK)) | color.as_background_color();
        self.set_text_attribute(new_attributes)
    }
}