#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "TMap.h"

typedef enum MapDependencyType {
  MAP_DEPENDENCY_TYPE_LEAF,
  MAP_DEPENDENCY_TYPE_ALL_OF,
  MAP_DEPENDENCY_TYPE_ANY_OF,
  MAP_DEPENDENCY_TYPE_ONE_OF,
} MapDependencyType;

typedef union MapDependency MapDependency;

typedef struct MapDependencyLeafValue {
  const char *key;
} MapDependencyLeafValue;

typedef struct MapDependencyArrayValue {
  const MapDependency *array;
  size_t array_length;
} MapDependencyArrayValue;

typedef struct MapDependencyLeaf {
  MapDependencyType type;
  MapDependencyLeafValue value;
} MapDependencyLeaf;

typedef struct MapDependencyArray {
  MapDependencyType type;
  MapDependencyArrayValue value;
} MapDependencyArray;

union MapDependency {
  MapDependencyType type;
  MapDependencyLeaf leaf;
  MapDependencyArray array;
};

typedef struct ModMetadata {
  const char *id;
  uint16_t mod_major_version;
  uint16_t mod_minor_version;
  uint16_t compatible_engine_major_version;
  uint16_t compatible_engine_minor_version;
} ModMetadata;

typedef err_t (*ModApplyFunction)(TMap map, TMap_search search);
typedef err_t (*ModValidateFunction)(TMap map, TMap_search search);

typedef struct Mod {
  ModMetadata metadata;
  ModApplyFunction apply;
  ModValidateFunction validate;
} Mod;
