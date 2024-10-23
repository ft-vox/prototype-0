extern crate libc;

use libc::{c_char, c_void};
use std::ffi::CString;

#[repr(C)]
struct Internal {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct Library {
    library: *mut Internal,
}

extern "C" {
    fn LibraryOpen(path: *const c_char) -> *mut Internal;
    fn LibraryGet(self_: *mut Internal, name: *const c_char) -> *mut c_void;
    fn LibraryClose(self_: *mut Internal);
}

impl Library {
    pub fn open(libname: &str) -> Option<Self> {
        let c_libname = CString::new(libname).unwrap();
        unsafe {
            let library = LibraryOpen(c_libname.as_ptr());
            if library.is_null() {
                None
            } else {
                Some(Library { library })
            }
        }
    }

    /**
    # Safety

    It's safe only if given type T is appropriate.
    */
    pub unsafe fn get<T>(&self, symbol: &str) -> Option<*mut T> {
        let c_symbol = CString::new(symbol).unwrap();
        unsafe {
            let sym = LibraryGet(self.library, c_symbol.as_ptr()) as *mut T;
            if sym.is_null() {
                None
            } else {
                Some(sym)
            }
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            LibraryClose(self.library);
        }
    }
}
