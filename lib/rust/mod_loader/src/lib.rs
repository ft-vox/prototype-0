use std::{
    collections::{BTreeSet, HashSet},
    ffi::CStr,
};

use library_wrapper::Library;
use mod_bindings::MapDependency;
use tmap_wrapper::TMap;

mod mod_bindings;

struct Mod {
    library_handle: Library,
    name: String,
    major_version: u16,
    minor_version: u16,
}

struct ModBuilder {
    result: Mod,
    compatible_engine_major_version: u16,
    compatible_engine_minor_version: u16,
    dependency: *const MapDependency,
}

impl ModBuilder {
    pub unsafe fn open(mod_name: &str) -> ModBuilder {
        let library_handle = Library::open(mod_name).expect("mod not found.");
        let c_mod = *library_handle.get::<mod_bindings::Mod>("mod").unwrap();

        ModBuilder {
            result: Mod {
                library_handle,
                name: CStr::from_ptr(c_mod.metadata.id)
                    .to_str()
                    .unwrap()
                    .to_owned(),
                major_version: c_mod.metadata.mod_major_version,
                minor_version: c_mod.metadata.mod_minor_version,
            },
            compatible_engine_major_version: c_mod.metadata.compatible_engine_major_version,
            compatible_engine_minor_version: c_mod.metadata.compatible_engine_minor_version,
            dependency: c_mod.metadata.dependency,
        }
    }
}

struct ModsBuilder {
    mod_id_set: HashSet<String>,
    map: Option<TMap>,
}

impl ModsBuilder {
    /**
    # Safety

    It must be dropped before library handle drops.
     */
    unsafe fn new() -> ModsBuilder {
        ModsBuilder {
            mod_id_set: HashSet::new(),
            map: Some(TMap::new()),
        }
    }
}

pub enum ModsConstructionError {
    DuplicateIds(Vec<String>),
    UnresolvedDependencies {
        unresolved_dependencies: Vec<String>,
        modules_failed_to_load: Vec<String>,
    },
}

pub struct Mods {
    map: TMap, // must be dropped before library_handles drops, so it must be earlier field
    library_handles: Vec<Mod>, // for more details, see https://stackoverflow.com/a/41056727
}

impl Mods {
    pub fn new(
        names: &[String],
        engine_major_version: u16,
        engine_minor_version: u16,
    ) -> Result<Mods, ModsConstructionError> {
        let library_handles = Vec::new();
        let unresolved_dependencies = BTreeSet::new();
        let unresolved_mods;
        {
            let mut builder = unsafe { ModsBuilder::new() };
            // ... add mods to builder
            if unresolved_dependencies.len() == 0 {
                return Ok(Mods {
                    map: builder.map.take().unwrap(),
                    library_handles,
                });
            }
        }
        Err(ModsConstructionError::UnresolvedDependencies(
            unresolved_dependencies.into_iter().collect(),
        ))
    }
}
