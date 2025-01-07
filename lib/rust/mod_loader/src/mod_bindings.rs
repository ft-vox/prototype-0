/* automatically generated by rust-bindgen 0.71.1 */

pub type err_t = bool;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TMap {
    _unused: [u8; 0],
}
pub type TMap_ptr = *mut TMap;
pub type TMap_search = ::std::option::Option<
    unsafe extern "C" fn(
        map: TMap_ptr,
        key: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_void,
>;
pub type TMap_has = ::std::option::Option<
    unsafe extern "C" fn(map: TMap_ptr, key: *const ::std::os::raw::c_char) -> bool,
>;
pub const MapDependencyType_MAP_DEPENDENCY_TYPE_LEAF: MapDependencyType = 0;
pub const MapDependencyType_MAP_DEPENDENCY_TYPE_ALL_OF: MapDependencyType = 1;
pub const MapDependencyType_MAP_DEPENDENCY_TYPE_ANY_OF: MapDependencyType = 2;
pub const MapDependencyType_MAP_DEPENDENCY_TYPE_ONE_OF: MapDependencyType = 3;
pub type MapDependencyType = ::std::os::raw::c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MapDependencyLeafValue {
    pub key: *const ::std::os::raw::c_char,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of MapDependencyLeafValue"][::std::mem::size_of::<MapDependencyLeafValue>() - 8usize];
    ["Alignment of MapDependencyLeafValue"]
        [::std::mem::align_of::<MapDependencyLeafValue>() - 8usize];
    ["Offset of field: MapDependencyLeafValue::key"]
        [::std::mem::offset_of!(MapDependencyLeafValue, key) - 0usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MapDependencyArrayValue {
    pub array: *const MapDependency,
    pub array_length: usize,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of MapDependencyArrayValue"][::std::mem::size_of::<MapDependencyArrayValue>() - 16usize];
    ["Alignment of MapDependencyArrayValue"]
        [::std::mem::align_of::<MapDependencyArrayValue>() - 8usize];
    ["Offset of field: MapDependencyArrayValue::array"]
        [::std::mem::offset_of!(MapDependencyArrayValue, array) - 0usize];
    ["Offset of field: MapDependencyArrayValue::array_length"]
        [::std::mem::offset_of!(MapDependencyArrayValue, array_length) - 8usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MapDependencyLeaf {
    pub type_: MapDependencyType,
    pub value: MapDependencyLeafValue,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of MapDependencyLeaf"][::std::mem::size_of::<MapDependencyLeaf>() - 16usize];
    ["Alignment of MapDependencyLeaf"][::std::mem::align_of::<MapDependencyLeaf>() - 8usize];
    ["Offset of field: MapDependencyLeaf::type_"]
        [::std::mem::offset_of!(MapDependencyLeaf, type_) - 0usize];
    ["Offset of field: MapDependencyLeaf::value"]
        [::std::mem::offset_of!(MapDependencyLeaf, value) - 8usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MapDependencyArray {
    pub type_: MapDependencyType,
    pub value: MapDependencyArrayValue,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of MapDependencyArray"][::std::mem::size_of::<MapDependencyArray>() - 24usize];
    ["Alignment of MapDependencyArray"][::std::mem::align_of::<MapDependencyArray>() - 8usize];
    ["Offset of field: MapDependencyArray::type_"]
        [::std::mem::offset_of!(MapDependencyArray, type_) - 0usize];
    ["Offset of field: MapDependencyArray::value"]
        [::std::mem::offset_of!(MapDependencyArray, value) - 8usize];
};
#[repr(C)]
#[derive(Copy, Clone)]
pub union MapDependency {
    pub type_: MapDependencyType,
    pub leaf: MapDependencyLeaf,
    pub array: MapDependencyArray,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of MapDependency"][::std::mem::size_of::<MapDependency>() - 24usize];
    ["Alignment of MapDependency"][::std::mem::align_of::<MapDependency>() - 8usize];
    ["Offset of field: MapDependency::type_"]
        [::std::mem::offset_of!(MapDependency, type_) - 0usize];
    ["Offset of field: MapDependency::leaf"][::std::mem::offset_of!(MapDependency, leaf) - 0usize];
    ["Offset of field: MapDependency::array"]
        [::std::mem::offset_of!(MapDependency, array) - 0usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ModMetadata {
    pub id: *const ::std::os::raw::c_char,
    pub mod_major_version: u16,
    pub mod_minor_version: u16,
    pub compatible_engine_major_version: u16,
    pub compatible_engine_minor_version: u16,
    pub dependency: *mut MapDependency,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of ModMetadata"][::std::mem::size_of::<ModMetadata>() - 24usize];
    ["Alignment of ModMetadata"][::std::mem::align_of::<ModMetadata>() - 8usize];
    ["Offset of field: ModMetadata::id"][::std::mem::offset_of!(ModMetadata, id) - 0usize];
    ["Offset of field: ModMetadata::mod_major_version"]
        [::std::mem::offset_of!(ModMetadata, mod_major_version) - 8usize];
    ["Offset of field: ModMetadata::mod_minor_version"]
        [::std::mem::offset_of!(ModMetadata, mod_minor_version) - 10usize];
    ["Offset of field: ModMetadata::compatible_engine_major_version"]
        [::std::mem::offset_of!(ModMetadata, compatible_engine_major_version) - 12usize];
    ["Offset of field: ModMetadata::compatible_engine_minor_version"]
        [::std::mem::offset_of!(ModMetadata, compatible_engine_minor_version) - 14usize];
    ["Offset of field: ModMetadata::dependency"]
        [::std::mem::offset_of!(ModMetadata, dependency) - 16usize];
};
pub type ModApplyFunction =
    ::std::option::Option<unsafe extern "C" fn(map: TMap_ptr, search: TMap_search) -> err_t>;
pub type ModValidateFunction =
    ::std::option::Option<unsafe extern "C" fn(map: TMap_ptr, search: TMap_search) -> err_t>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Mod {
    pub metadata: ModMetadata,
    pub apply: ModApplyFunction,
    pub validate: ModValidateFunction,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of Mod"][::std::mem::size_of::<Mod>() - 40usize];
    ["Alignment of Mod"][::std::mem::align_of::<Mod>() - 8usize];
    ["Offset of field: Mod::metadata"][::std::mem::offset_of!(Mod, metadata) - 0usize];
    ["Offset of field: Mod::apply"][::std::mem::offset_of!(Mod, apply) - 24usize];
    ["Offset of field: Mod::validate"][::std::mem::offset_of!(Mod, validate) - 32usize];
};
unsafe extern "C" {
    pub fn get_any_unresolved_map_dependency(
        dependency: *const MapDependency,
        map: TMap_ptr,
        has: TMap_has,
    ) -> *const MapDependency;
}
unsafe extern "C" {
    #[doc = " Retrieves all unresolved map dependencies.\n\n Allocates an array of unresolved dependencies and stores it in `*out`.\n The number of dependencies is stored in `*out_length`.\n Caller is responsible for freeing the memory allocated for `*out`.\n\n @param dependency The root dependency to resolve.\n @param map The dependency map.\n @param has The callback function to check dependency availability.\n @param out Output parameter for the array of unresolved dependencies.\n @param out_length Output parameter for the number of unresolved dependencies.\n @return false on success, or true on memory allocation failure."]
    pub fn get_all_unresolved_map_dependencies(
        dependency: *const MapDependency,
        map: TMap_ptr,
        has: TMap_has,
        out: *mut *mut *const MapDependency,
        out_length: *mut usize,
    ) -> err_t;
}
