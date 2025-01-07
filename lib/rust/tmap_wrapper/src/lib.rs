use std::{error::Error, ffi::CString, fmt::Display, os::raw::c_void};

use tmap_bindings::{TMap_delete, TMap_has, TMap_insert, TMap_new, TMap_ptr, TMap_search};

mod tmap_bindings;

pub struct TMap {
    raw: TMap_ptr,
}

#[derive(Debug)]
pub enum TMapInsertError {
    AlreadyExist,
}

impl Display for TMapInsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TMapInsertError {}

impl TMap {
    pub fn new() -> TMap {
        let result = unsafe { TMap_new() };
        if result.is_null() {
            panic!("Memory allocation failure");
        }
        TMap { raw: result }
    }

    pub fn raw(&self) -> TMap_ptr {
        self.raw
    }

    fn cast_fn<T>(
        opt_fn: Option<unsafe extern "C" fn(*mut T)>,
    ) -> Option<unsafe extern "C" fn(*mut c_void)> {
        opt_fn.map(|f| unsafe {
            std::mem::transmute::<unsafe extern "C" fn(*mut T), unsafe extern "C" fn(*mut c_void)>(
                f,
            )
        })
    }

    pub fn insert<T>(
        &mut self,
        key: &str,
        value: *mut T,
        delete_value: Option<unsafe extern "C" fn(value: *mut T)>,
    ) -> Result<(), TMapInsertError> {
        let c_key = CString::new(key).unwrap();
        if unsafe {
            TMap_insert(
                self.raw,
                c_key.as_ptr(),
                value as *mut c_void,
                Self::cast_fn(delete_value),
            )
        } {
            if unsafe { TMap_has(self.raw, c_key.as_ptr()) } {
                Err(TMapInsertError::AlreadyExist)
            } else {
                panic!("Memory allocation failure")
            }
        } else {
            Ok(())
        }
    }

    /**
    # Safety

    It's safe only if the type T corresponding to the key is correct.
     */
    pub unsafe fn search<T>(&self, key: &str) -> Option<*mut T> {
        let c_key = CString::new(key).unwrap();
        let result = TMap_search(self.raw, c_key.as_ptr());
        if result.is_null() && !TMap_has(self.raw, c_key.as_ptr()) {
            None
        } else {
            Some(result as *mut T)
        }
    }

    pub fn has(&self, key: &str) -> bool {
        let c_key = CString::new(key).unwrap();
        unsafe { TMap_has(self.raw, c_key.as_ptr()) }
    }
}

impl Drop for TMap {
    fn drop(&mut self) {
        unsafe { TMap_delete(self.raw) };
    }
}

pub const TMap_get: unsafe extern "C" fn(
    map: TMap_ptr,
    key: *const ::std::os::raw::c_char,
) -> *mut ::std::os::raw::c_void = tmap_bindings::TMap_search;
