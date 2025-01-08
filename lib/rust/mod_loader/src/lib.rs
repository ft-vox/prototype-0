use std::{collections::HashMap, ffi::CStr};

use library_wrapper::Library;
use mod_bindings::{ModApplyFunction, ModValidateFunction};
use tmap_wrapper::{TMap, TMap_get};

mod mod_bindings;

struct Mod {
    library_handle: Library,
    id: String,
    major_version: u16,
    minor_version: u16,
}

struct ModBuilder {
    result: Mod,
    apply: ModApplyFunction,
    validate: ModValidateFunction,
    compatible_engine_major_version: u16,
    compatible_engine_minor_version: u16,
}

impl ModBuilder {
    pub unsafe fn open(mod_name: &str) -> ModBuilder {
        let library_handle = Library::open(mod_name).expect("mod not found.");
        let c_mod = *library_handle.get::<mod_bindings::Mod>("mod").unwrap();

        ModBuilder {
            result: Mod {
                library_handle,
                id: CStr::from_ptr(c_mod.metadata.id)
                    .to_str()
                    .unwrap()
                    .to_owned(),
                major_version: c_mod.metadata.mod_major_version,
                minor_version: c_mod.metadata.mod_minor_version,
            },
            apply: c_mod.apply,
            validate: c_mod.validate,
            compatible_engine_major_version: c_mod.metadata.compatible_engine_major_version,
            compatible_engine_minor_version: c_mod.metadata.compatible_engine_minor_version,
        }
    }
}

pub struct ModInfo {
    name: String,
    id: String,
}

pub enum ModsConstructionError {
    VersionIncompatible(Vec<ModInfo>),
    DuplicateIds(Vec<ModInfo>),
    FailedToLoad(ModInfo),
    ConditionNotMet(Vec<ModInfo>),
    UnresolvedDependencies {
        unresolved_dependencies: Vec<String>,
        modules_failed_to_load: Vec<String>,
    },
}

pub struct Mods {
    pub map: TMap, // must be dropped before library_handles drops, so it must be earlier field
    mods: Vec<Mod>, // for more details, see https://stackoverflow.com/a/41056727
}

impl Mods {
    pub fn new(
        names: &[String],
        engine_major_version: u16,
        engine_minor_version: u16,
    ) -> Result<Mods, ModsConstructionError> {
        let mut version_incompatible = Vec::new();
        let mod_builders: Vec<_> = names
            .iter()
            .flat_map(|name| {
                let mod_builder = unsafe { ModBuilder::open(name) };
                if mod_builder.compatible_engine_major_version != engine_major_version
                    || mod_builder.compatible_engine_minor_version > engine_minor_version
                {
                    version_incompatible.push(ModInfo {
                        name: name.clone(),
                        id: mod_builder.result.id,
                    });
                    None
                } else {
                    Some(mod_builder)
                }
            })
            .collect();
        if mod_builders.len() != names.len() {
            return Err(ModsConstructionError::VersionIncompatible(
                version_incompatible,
            ));
        }

        let mut ids = HashMap::new();
        for mod_builder in mod_builders.iter() {
            let count = ids.entry(mod_builder.result.id.as_str()).or_insert(0);
            *count += 1usize;
        }
        if ids.len() != names.len() {
            return Err(ModsConstructionError::DuplicateIds(
                mod_builders
                    .iter()
                    .enumerate()
                    .flat_map(|(index, mod_builder)| {
                        if *ids.get(mod_builder.result.id.as_str()).unwrap() > 1 {
                            Some(ModInfo {
                                name: names[index].clone(),
                                id: mod_builder.result.id.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            ));
        }

        let mut result = TMap::new();
        for (index, mod_builder) in mod_builders.iter().enumerate() {
            if unsafe {
                mod_builder.apply.unwrap()(
                    std::mem::transmute(result.raw()),
                    Some(std::mem::transmute(TMap_get)),
                )
            } {
                return Err(ModsConstructionError::FailedToLoad(ModInfo {
                    name: names[index].clone(),
                    id: mod_builder.result.id.clone(),
                }));
            }
        }

        let failed_to_load: Vec<_> = mod_builders
            .iter()
            .enumerate()
            .flat_map(|(index, mod_builder)| {
                if unsafe {
                    mod_builder.validate.unwrap()(
                        std::mem::transmute(result.raw()),
                        Some(std::mem::transmute(TMap_get)),
                    )
                } {
                    Some(ModInfo {
                        name: names[index].clone(),
                        id: mod_builder.result.id.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();
        if failed_to_load.len() != 0 {
            return Err(ModsConstructionError::ConditionNotMet(failed_to_load));
        }

        return Ok(Mods {
            map: result,
            mods: mod_builders
                .into_iter()
                .map(|mod_builder| mod_builder.result)
                .collect(),
        });
    }
}
