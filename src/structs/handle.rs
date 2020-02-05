use std::ops::Deref;
use std::sync::Arc;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::winnt::HANDLE;

/// Wraps a windows [HANDLE].
#[derive(Debug, Clone)]
pub struct Handle {
    inner: RawHandle,
}

/// Wraps the actual [HANDLE] and drop it is owned.
#[derive(Debug, Clone)]
struct RawHandle {
    handle: HANDLE,
    ownership: HandleOwnership,
}

// Synchronize the [Inner].
unsafe impl Send for RawHandle {}
unsafe impl Sync for RawHandle {}

/// Represents the ownership of a `HANDLE`.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum HandleOwnership {
    /// The handle is owned by the instance so should be close.
    Owned,
    /// The handle is used for others instances, so cannot be close.
    Shared,
}

impl Drop for RawHandle {
    fn drop(&mut self) {
        if self.ownership == HandleOwnership::Owned {
            assert!(
                unsafe { CloseHandle(self.handle) != 0 },
                "Cannot close the handle"
            )
        }
    }
}

impl Handle {
    /// Creates a new shared `Handle` from the specified.
    ///
    /// # Examples
    ///
    /// Basic usages:
    /// ```
    /// use winapi::um::winbase::STD_INPUT_HANDLE;
    /// use winapi::um::processenv::GetStdHandle;
    /// use win32console::structs::handle::Handle;
    ///
    /// let handle = Handle::from_raw(unsafe { GetStdHandle(STD_INPUT_HANDLE) });
    /// assert!(handle.is_valid());
    /// ```
    pub fn from_raw(handle: HANDLE) -> Handle {
        Handle {
            inner: RawHandle {
                handle,
                ownership: HandleOwnership::Shared
            },
        }
    }

    /// Creates a new `Handle` from the specified which will be close
    /// when goes out of scope by calling [CloseHandle].
    ///
    /// # Examples
    ///
    /// Basic usages:
    /// ```
    /// use win32console::structs::handle::Handle;
    /// use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    /// use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    /// use winapi::um::winnt::{GENERIC_READ, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_WRITE};
    /// use std::ptr::null_mut;
    ///
    /// let file_name: Vec<u16> = "CONIN$\0".encode_utf16().collect();
    /// let handle = Handle::new_closeable(unsafe { CreateFileW(
    ///                file_name.as_ptr(),
    ///                GENERIC_READ | GENERIC_WRITE,
    ///                FILE_SHARE_READ | FILE_SHARE_WRITE,
    ///                null_mut(),
    ///                OPEN_EXISTING,
    ///                0,
    ///                null_mut(),
    ///            ) });
    /// assert_ne!(*handle, INVALID_HANDLE_VALUE);
    /// ```
    pub fn new_closeable(handle: HANDLE) -> Handle {
        Handle {
            inner: RawHandle {
                handle,
                ownership: HandleOwnership::Owned
            },
        }
    }

    /// Gets a reference to the underlying `HANDLE`.
    ///
    /// # Examples
    ///
    /// Basic usages:
    /// ```
    /// use win32console::structs::handle::Handle;
    /// use winapi::um::processenv::GetStdHandle;
    /// use winapi::um::winbase::STD_INPUT_HANDLE;
    /// use winapi::um::winnt::HANDLE;
    /// use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    ///
    /// let handle = Handle::from_raw(unsafe { GetStdHandle(STD_INPUT_HANDLE) });
    /// let raw_handle = handle.get_raw();
    /// assert_ne!(*raw_handle, INVALID_HANDLE_VALUE);
    /// ```
    pub fn get_raw(&self) -> &HANDLE {
        &self.inner.handle
    }

    /// Compare this handle to [INVALID_HANDLE_VALUE] to determines if the handle is valid.
    ///
    /// # Examples:
    ///
    /// Basic usages:
    /// ```
    /// use win32console::structs::handle::Handle;
    /// use winapi::um::processenv::GetStdHandle;
    /// use winapi::um::winbase::STD_INPUT_HANDLE;
    /// use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    ///
    /// let handle = Handle::from_raw(unsafe { GetStdHandle(STD_INPUT_HANDLE) });
    /// assert!(handle.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        if self.inner.handle == INVALID_HANDLE_VALUE {
            return false;
        }

        true
    }
}

impl Deref for Handle {
    type Target = HANDLE;

    /// Gets a reference to the underlying [HANDLE].
    fn deref(&self) -> &Self::Target {
        &self.inner.handle
    }
}
