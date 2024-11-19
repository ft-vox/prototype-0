#include "mod.h"

#include <stdlib.h>

#include "TMap.h"
#include "mod_types.h"

static const MapDependency *
get_any_unresolved_map_dependency_leaf(const MapDependency *original,
                                       const MapDependencyLeafValue *dependency,
                                       TMap map, TMap_has has);
static const MapDependency *get_any_unresolved_map_dependency_all_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has);
static const MapDependency *get_any_unresolved_map_dependency_any_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has);
static const MapDependency *get_any_unresolved_map_dependency_one_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has);

const MapDependency *
get_any_unresolved_map_dependency(const MapDependency *dependency, TMap map,
                                  TMap_has has) {
  switch (dependency->type) {
  case MAP_DEPENDENCY_TYPE_LEAF:
    return get_any_unresolved_map_dependency_leaf(
        dependency, &dependency->leaf.value, map, has);
  case MAP_DEPENDENCY_TYPE_ALL_OF:
    return get_any_unresolved_map_dependency_all_of(
        dependency, &dependency->array.value, map, has);
  case MAP_DEPENDENCY_TYPE_ANY_OF:
    return get_any_unresolved_map_dependency_any_of(
        dependency, &dependency->array.value, map, has);
  case MAP_DEPENDENCY_TYPE_ONE_OF:
    return get_any_unresolved_map_dependency_one_of(
        dependency, &dependency->array.value, map, has);
  }
}

static const MapDependency *
get_any_unresolved_map_dependency_leaf(const MapDependency *original,
                                       const MapDependencyLeafValue *dependency,
                                       TMap map, TMap_has has) {
  return has(map, dependency->key) ? NULL : original;
}

static const MapDependency *get_any_unresolved_map_dependency_all_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has) {
  (void)original;
  for (size_t i = 0; i < dependency->array_length; i++) {
    const MapDependency *const tmp =
        get_any_unresolved_map_dependency(&dependency->array[i], map, has);
    if (tmp) {
      return &dependency->array[i];
    }
  }
  return NULL;
}

static const MapDependency *get_any_unresolved_map_dependency_any_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has) {
  for (size_t i = 0; i < dependency->array_length; i++) {
    const MapDependency *const tmp =
        get_any_unresolved_map_dependency(&dependency->array[i], map, has);
    if (!tmp) {
      return NULL;
    }
  }
  return original;
}

static const MapDependency *get_any_unresolved_map_dependency_one_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has) {
  bool appeared = false;
  for (size_t i = 0; i < dependency->array_length; i++) {
    const MapDependency *const tmp =
        get_any_unresolved_map_dependency(&dependency->array[i], map, has);
    if (tmp) {
      if (appeared) {
        return original;
      }
      appeared = true;
    }
  }
  return appeared ? NULL : original;
}

typedef struct ConstMapDependencyPointerArrayBuilder {
  const MapDependency **array;
  size_t capacity;
  size_t length;
} ConstMapDependencyPointerArrayBuilder;

static err_t ConstMapDependencyPointerArrayBuilder__append(
    ConstMapDependencyPointerArrayBuilder *self, const MapDependency *value) {
  if (self->length == self->capacity) {
    const MapDependency **const new_array = (const MapDependency **)realloc(
        self->array, sizeof(const MapDependency *) * self->capacity * 2);
    if (!new_array) {
      return true;
    }
    self->array = new_array;
    self->capacity *= 2;
  }
  self->array[self->length++] = value;
  return false;
}

static err_t append_all_unresolved_map_dependencies(
    const MapDependency *dependency, TMap map, TMap_has has,
    ConstMapDependencyPointerArrayBuilder *builder);
static err_t append_all_unresolved_map_dependencies_leaf(
    const MapDependency *original, const MapDependencyLeafValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder);
static err_t append_all_unresolved_map_dependencies_all_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder);
static err_t append_all_unresolved_map_dependencies_any_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder);
static err_t append_all_unresolved_map_dependencies_one_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder);

err_t get_all_unresolved_map_dependencies(const MapDependency *dependency,
                                          TMap map, TMap_has has,
                                          const MapDependency ***out,
                                          size_t *out_length) {
  ConstMapDependencyPointerArrayBuilder result = {
      (const MapDependency **)malloc(sizeof(const MapDependency *) * 10),
      10,
      0,
  };
  if (!result.array) {
    return true;
  }
  if (append_all_unresolved_map_dependencies(dependency, map, has, &result)) {
    free(result.array);
    return true;
  }
  *out = result.array;
  *out_length = result.length;
  return false;
}

static err_t append_all_unresolved_map_dependencies(
    const MapDependency *dependency, TMap map, TMap_has has,
    ConstMapDependencyPointerArrayBuilder *builder) {
  switch (dependency->type) {
  case MAP_DEPENDENCY_TYPE_LEAF:
    return append_all_unresolved_map_dependencies_leaf(
        dependency, &dependency->leaf.value, map, has, builder);
  case MAP_DEPENDENCY_TYPE_ALL_OF:
    return append_all_unresolved_map_dependencies_all_of(
        dependency, &dependency->array.value, map, has, builder);
  case MAP_DEPENDENCY_TYPE_ANY_OF:
    return append_all_unresolved_map_dependencies_any_of(
        dependency, &dependency->array.value, map, has, builder);
  case MAP_DEPENDENCY_TYPE_ONE_OF:
    return append_all_unresolved_map_dependencies_one_of(
        dependency, &dependency->array.value, map, has, builder);
  }
}

static err_t append_all_unresolved_map_dependencies_leaf(
    const MapDependency *original, const MapDependencyLeafValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder) {
  return has(map, dependency->key)
             ? false
             : ConstMapDependencyPointerArrayBuilder__append(builder, original);
}

static err_t append_all_unresolved_map_dependencies_all_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder) {
  (void)original;
  for (size_t i = 0; i < dependency->array_length; i++) {
    if (append_all_unresolved_map_dependencies(&dependency->array[i], map, has,
                                               builder)) {
      return true;
    }
  }
  return false;
}

static err_t append_all_unresolved_map_dependencies_any_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder) {
  return get_any_unresolved_map_dependency_any_of(original, dependency, map,
                                                  has) &&
         ConstMapDependencyPointerArrayBuilder__append(builder, original);
}

static err_t append_all_unresolved_map_dependencies_one_of(
    const MapDependency *original, const MapDependencyArrayValue *dependency,
    TMap map, TMap_has has, ConstMapDependencyPointerArrayBuilder *builder) {
  return get_any_unresolved_map_dependency_one_of(original, dependency, map,
                                                  has) &&
         ConstMapDependencyPointerArrayBuilder__append(builder, original);
}
