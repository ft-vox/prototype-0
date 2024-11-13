use std::{ffi::CStr, os::raw::c_char};

use library_wrapper::Library;
use tmap_wrapper::TMap;

struct Mod {
    library_handle: Library,
    name: String,
    major_version: u16,
    minor_version: u16,
}

impl Mod {
    pub unsafe fn open(mod_name: &str) -> Mod {
        let library_handle = Library::open(mod_name).expect("mod not found.");
        let c_name = library_handle.get::<c_char>("name").unwrap();
        let c_major_version = library_handle.get::<u16>("major_version").unwrap();
        let c_minor_version = library_handle.get::<u16>("minor_version").unwrap();

        Mod {
            library_handle,
            name: CStr::from_ptr(c_name).to_str().unwrap().to_owned(),
            major_version: *c_major_version,
            minor_version: *c_minor_version,
        }
    }
}

pub struct Mods {
    map: TMap, // must be dropped before library_handles drops, so it must be earlier field
    library_handles: Vec<Mod>, // for more details, see https://stackoverflow.com/a/41056727
}

impl Mods {
    pub fn new() -> Mods {
        Mods {
            map: TMap::new(),
            library_handles: Vec::new(),
        }
    }
}
