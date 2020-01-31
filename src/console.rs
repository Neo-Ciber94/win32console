use std::io::{Error, Result, ErrorKind};
use std::iter;
use std::mem::{transmute, MaybeUninit};
use std::str;

use winapi::_core::ptr::{null, null_mut};
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{FALSE, MAX_PATH};
use winapi::um::consoleapi::{GetConsoleMode, GetNumberOfConsoleInputEvents, ReadConsoleInputW, SetConsoleMode, WriteConsoleW, ReadConsoleW};
use winapi::um::fileapi::{CreateFileW, WriteFile, OPEN_EXISTING};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::processenv::{GetStdHandle, SetStdHandle};
use winapi::um::winbase::{STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
use winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO;
use winapi::um::wincon::CONSOLE_READCONSOLE_CONTROL;
use winapi::um::wincon::{
    FillConsoleOutputAttribute, FillConsoleOutputCharacterW, GetConsoleScreenBufferInfo,
    GetConsoleScreenBufferInfoEx, GetConsoleTitleW, GetCurrentConsoleFontEx,
    GetLargestConsoleWindowSize, GetNumberOfConsoleMouseButtons, PeekConsoleInputW,
    SetConsoleCursorPosition, SetConsoleScreenBufferInfoEx, SetConsoleTextAttribute,
    SetConsoleTitleW, SetCurrentConsoleFontEx, WriteConsoleOutputCharacterW, CONSOLE_FONT_INFOEX,
    INPUT_RECORD,
};
use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE};

use crate::structs::console_font_info::ConsoleFontInfoEx;
use crate::structs::console_screen_buffer_info::ConsoleScreenBufferInfo;
use crate::structs::console_screen_buffer_info_ex::ConsoleScreenBufferInfoEx;
use crate::structs::coord::Coord;
use crate::structs::handle::{Handle};
use crate::structs::input::InputRecord;
use std::sync::Arc;

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

/// Wraps constants values of the console modes.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/getconsolemode]
pub struct ConsoleMode;

/// Wraps constants values of the console text attributes.
///
/// link: [https://docs.microsoft.com/en-us/windows/console/console-screen-buffers]
pub struct ConsoleTextAttribute;

impl ConsoleMode {
    pub const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
    pub const ENABLE_ECHO_INPUT: u32 = 0x0004;
    pub const ENABLE_INSERT_MODE: u32 = 0x0020;
    pub const ENABLE_LINE_INPUT: u32 = 0x0002;
    pub const ENABLE_MOUSE_INPUT: u32 = 0x0010;
    pub const ENABLE_QUICK_EDIT_MODE: u32 = 0x0040;
    pub const ENABLE_WINDOW_INPUT: u32 = 0x0008;
    pub const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;
}

impl ConsoleTextAttribute {
    pub const FOREGROUND_BLUE: u16 = 1;
    pub const FOREGROUND_GREEN: u16 = 2;
    pub const FOREGROUND_RED: u16 = 4;
    pub const FOREGROUND_INTENSITY: u16 = 8;
    pub const BACKGROUND_BLUE: u16 = 16;
    pub const BACKGROUND_GREEN: u16 = 32;
    pub const BACKGROUND_RED: u16 = 64;
    pub const BACKGROUND_INTENSITY: u16 = 128;
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

            Ok(Handle::from_raw(raw_handle))
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
    pub fn get_or_create_input_handle() -> Result<Handle> {
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

        Ok(Handle::new_closeable(raw_handle))
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
    pub fn get_or_create_output_handle() -> Result<Handle> {
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

        Ok(Handle::new_closeable(raw_handle))
    }
}

// Factory methods
impl WinConsole {
    /// Gets a console with the [STD_INPUT_HANDLE].
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
    pub fn current_input() -> WinConsole {
        #[inline]
        fn get_input_handle() -> Handle {
            WinConsole::get_or_create_input_handle().expect("Invalid handle")
        }

        WinConsole {
            handle_provider: Box::new(get_input_handle),
        }
    }

    /// Gets a console with the current output handle.
    pub fn current_output() -> WinConsole {
        #[inline]
        fn get_output_handle() -> Handle {
            WinConsole::get_or_create_output_handle().expect("Invalid handle")
        }

        WinConsole {
            handle_provider: Box::new(get_output_handle),
        }
    }

    /// Gets a console with the specified handle provider.
    /// The function should return a valid handle after each call.
    pub fn from<F>(handle_provider: F) -> WinConsole where F: 'static + Sized + Fn() -> Handle,
    {
        WinConsole {
            handle_provider: Box::new(handle_provider),
        }
    }
}

/// Public methods
impl WinConsole {
    /// Gets the handle used for this console.
    pub fn get_handle(&self) -> Handle {
        (&self.handle_provider)()
    }

    /// Sets the title of the current console.
    ///
    /// Wraps a call to [SetConsoleTitle]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsoletitle]
    pub fn set_title(&self, title: &str) -> Result<()> {
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
    pub fn get_title(&self) -> Result<String> {
        let mut buffer = iter::repeat_with(u16::default)
            .take(MAX_PATH as usize)
            .collect::<Vec<u16>>();

        let temp = buffer.as_mut_slice();

        unsafe {
            let length = GetConsoleTitleW(temp.as_mut_ptr(), MAX_PATH as u32) as usize;

            if length == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(String::from_utf16_lossy(&temp[..length]))
            }
        }
    }

    /// Sets the[ConsoleFontInfoEx] of the current console.
    /// This function change the font into of all the current values in the console.
    ///
    /// Wraps a call to [SetCurrentConsoleFontEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setcurrentconsolefontex]
    pub fn set_console_font_info(&self, info: ConsoleFontInfoEx) -> Result<()> {
        let handle = self.get_handle();
        let mut info = info.into();

        unsafe {
            if SetCurrentConsoleFontEx(*handle, FALSE, &mut info) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Gets the current [ConsoleFontInfoEx] of the console.
    ///
    /// Wraps a call to [GetCurrentConsoleFontEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getcurrentconsolefontex]
    pub fn get_console_font_info(&self) -> Result<ConsoleFontInfoEx> {
        let handle = self.get_handle();
        unsafe {
            let mut info: CONSOLE_FONT_INFOEX = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<ConsoleFontInfoEx>() as u32;

            let ptr: *mut CONSOLE_FONT_INFOEX = &mut info;

            if GetCurrentConsoleFontEx(*handle, FALSE, ptr) == 0 {
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
    pub fn has_mode(&self, mode: u32) -> Result<bool> {
        match self.get_mode() {
            Ok(state) => Ok((state & mode) != 0),
            Err(error) => Err(error),
        }
    }

    /// Gets the current [ConsoleScreenBufferInfo].
    ///
    /// Wraps a call to [GetConsoleScreenBufferInfo]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfo]
    pub fn get_console_screen_buffer_info(&self) -> Result<ConsoleScreenBufferInfo> {
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

    /// Gets the current [ConsoleScreenBufferInfoEx].
    ///
    /// Wraps a call to [GetConsoleScreenBufferInfoEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfoex]
    pub fn get_console_screen_buffer_info_ex(&self) -> Result<ConsoleScreenBufferInfoEx> {
        let handle = self.get_handle();

        unsafe {
            let mut buffer_info = std::mem::zeroed();
            if GetConsoleScreenBufferInfoEx(*handle, &mut buffer_info) != 0 {
                Ok(ConsoleScreenBufferInfoEx::from(buffer_info))
            } else {
                Err(Error::last_os_error())
            }
        }
    }

    /// Sets teh current [ConsoleScreenBufferInfoEx]
    ///
    /// Wraps a call to [SetConsoleScreenBufferInfoEx]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolescreenbufferinfoex]
    pub fn set_console_screen_buffer_info_ex(&self, info: ConsoleScreenBufferInfoEx) -> Result<()> {
        let handle = self.get_handle();

        unsafe {
            let mut buf = info.into();
            if SetConsoleScreenBufferInfoEx(*handle, &mut buf) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    /// Sets the position of the cursor.
    ///
    /// Wraps a call to [SetConsoleCursorPosition]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsolecursorposition]
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

    /// Gets the current position of the cursor.
    pub fn get_cursor_position(&self) -> Result<Coord> {
        self.get_console_screen_buffer_info()
            .map(|value| value.cursor_position)
    }

    /// Clears the content of the console and set the cursor to (0, 0)
    pub fn clear(&self) -> Result<()> {
        unsafe {
            let mut info = self.get_console_screen_buffer_info()?;
            let size = info.size;
            let length: u32 = size.x as u32 * size.y as u32;
            self.fill_with_char(Coord::default(), length, ' ')?;

            self.fill_with_attribute(Coord::default(), length, info.attributes);
            self.set_cursor_position(Coord::default());

            Ok(())
        }
    }

    /// Fills the content of the console with the specified [char].
    ///
    /// Wraps a call to [FillConsoleOutputCharacterW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/fillconsoleoutputcharacter]
    pub fn fill_with_char(&self, start_location: Coord, cells_to_write: u32, value: char,) -> Result<u32> {
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
    /// Wraps a call to [SetConsoleTextAttribute]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsoletextattribute]
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
    pub fn get_text_attribute(&self) -> Result<u16> {
        Ok(self.get_console_screen_buffer_info()?.attributes)
    }

    /// Gets the largest size the console window can get.
    ///
    /// Wraps a call to [GetLargestConsoleWindowSize]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getlargestconsolewindowsize]
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
    pub fn get_number_of_input_events(&self) -> Result<u32> {
        let handle = self.get_handle();

        unsafe {
            let mut num_events = 0;
            if GetNumberOfConsoleInputEvents(*handle, &mut num_events) == 0 {
                Err(Error::last_os_error())
            } else {
                Ok(num_events)
            }
        }
    }

    /// Gets the number of mouse buttons used for the mouse available for this console.
    ///
    /// Wraps a call to [GetNumberOfConsoleMouseButtons]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getnumberofconsolemousebuttons]
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
    pub fn read_single_input(&self) -> Result<InputRecord> {
        unsafe {
            let mut record: [InputRecord; 1] = [std::mem::zeroed()];
            self.read_console_input(&mut record).map(|_| record[0])
        }
    }

    /// Reads the given number of events from the console.
    pub fn read_n_console_input(&self, count: u32) -> Result<Vec<InputRecord>> {
        if count == 0 {
            return Ok(vec![]);
        }

        let mut buffer : Vec<InputRecord> = iter::repeat_with(unsafe{ || std::mem::zeroed::<InputRecord>() })
            .take(count as usize)
            .collect();

        self.read_console_input(buffer.as_mut_slice()).map(|_|{
            buffer
        })
    }

    /// Fills the specified buffer with [InputRecord] from the console.
    ///
    /// Wraps a call to [ReadConsoleInputW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/readconsoleinput]
    pub fn read_console_input(&self, records: &mut [InputRecord]) -> Result<u32> {
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

                Ok(num_events)
            }
        }
    }

    pub fn read(&self) -> Result<String>{
        // Used buffer size from:
        // https://source.dot.net/#System.Console/System/Console.cs,dac049f8d10df4a0
        const MAX_BUFFER_SIZE : usize = 4096;

        let mut buffer : [u16; MAX_BUFFER_SIZE] = unsafe { MaybeUninit::zeroed().assume_init() };
        let chars_read = self.read_utf16(&mut buffer)?;

        match String::from_utf16(buffer[..chars_read].as_ref()) {
            Ok(s) => Ok(s),
            Err(e) => {
                Err(Error::new(ErrorKind::InvalidData, e))
            }
        }
    }

    pub fn read_utf8(&self, buffer: &mut [u8]) -> Result<usize>{
        let mut utf16_buffer = iter::repeat_with(u16::default)
            .take(buffer.len())
            .collect::<Vec<u16>>();

        // Writes the read data to the 'utf16_buffer'.
        self.read_utf16(&mut utf16_buffer)?;

        let mut written = 0;
        for chr in std::char::decode_utf16(utf16_buffer){
            match chr {
                Ok(chr) => {
                    chr.encode_utf8(&mut buffer[written..]);
                    written += 1;
                }
                Err(e) => {
                    return Err(Error::new(ErrorKind::InvalidData, "utf16 string not supported"));
                }
            }
        }

        Ok(written)
    }

    pub fn read_utf16(&self, buffer: &mut [u16]) -> Result<usize>{
        // https://github.com/rust-lang/rust/blob/master/src/libstd/sys/windows/stdio.rs
        const CTRL_Z : u16 = 0x1A;
        const CTRL_Z_MASK : u32 = (1 << CTRL_Z) as u32;

        let mut input_control = CONSOLE_READCONSOLE_CONTROL{
            nLength: std::mem::size_of::<CONSOLE_READCONSOLE_CONTROL>() as u32,
            nInitialChars: 0,
            dwCtrlWakeupMask: CTRL_Z_MASK,
            dwControlKeyState: 0
        };

        let handle = self.get_handle();
        let mut chars_read = 0;

        unsafe{
            if ReadConsoleW(
                *handle,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut chars_read,
                &mut input_control) == 0{
                Err(Error::last_os_error())
            }
            else{
                if chars_read > 0 && buffer[chars_read as usize - 1] == CTRL_Z {
                    chars_read -= 1;
                }

                Ok(chars_read as usize)
            }
        }
    }

    /// Fills the specified buffer with the unread [InputRecord] from the console.
    ///
    /// Wraps a call to [PeekConsoleInputW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/peekconsoleinput]
    pub fn peek_console_input(&self, records: &mut [InputRecord]) -> Result<u32> {
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

                Ok(num_events)
            }
        }
    }

    /// Writes the specified buffer of chars in the current cursor position of the console.
    ///
    /// Wraps a call to [WriteConsoleW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/writeconsole]
    pub fn write_utf8(&self, data: &[u8]) -> Result<u32> {
        let chars = match str::from_utf8(data) {
            Ok(values) => values.encode_utf16().collect::<Vec<u16>>(),
            Err(e) => {
                return Err(Error::new(std::io::ErrorKind::InvalidInput, e));
            }
        };

        self.write_utf16(chars.as_slice())
    }

    /// Writes the specified buffer of chars in the current cursor position of the console.
    ///
    /// Wraps a call to [WriteConsoleW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/writeconsole]
    pub fn write_utf16(&self, data: &[u16]) -> Result<u32> {
        let handle = self.get_handle();
        let mut chars_written = 0;

        // If is being redirected write to the handle
        if !WinConsole::is_console(*handle) {
            let buf = match String::from_utf16(data){
                Ok(string) => string,
                Err(e) => {
                    return Err(Error::new(std::io::ErrorKind::InvalidInput, e))
                }
            };

            unsafe {
                WriteFile(
                    *handle,
                    buf.as_ptr() as *const c_void,
                    data.len() as u32,
                    &mut chars_written,
                    null_mut(),
                );
            }
            return Ok(data.len() as u32);
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
                Ok(chars_written)
            }
        }
    }

    /// Checks if the handle is a handle to a console
    fn is_console(handle: HANDLE) -> bool {
        let mut mode = 0;
        unsafe { GetConsoleMode(handle, &mut mode) != 0 }
    }
}
