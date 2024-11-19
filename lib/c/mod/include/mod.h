#pragma once

#include "TMap.h"
#include "mod_types.h"

const MapDependency *
get_any_unresolved_map_dependency(const MapDependency *dependency, TMap map,
                                  TMap_has has);

/**
 * Retrieves all unresolved map dependencies.
 *
 * Allocates an array of unresolved dependencies and stores it in `*out`.
 * The number of dependencies is stored in `*out_length`.
 * Caller is responsible for freeing the memory allocated for `*out`.
 *
 * @param dependency The root dependency to resolve.
 * @param map The dependency map.
 * @param has The callback function to check dependency availability.
 * @param out Output parameter for the array of unresolved dependencies.
 * @param out_length Output parameter for the number of unresolved dependencies.
 * @return false on success, or true on memory allocation failure.
 */
err_t get_all_unresolved_map_dependencies(const MapDependency *dependency,
                                          TMap map, TMap_has has,
                                          const MapDependency ***out,
                                          size_t *out_length);
