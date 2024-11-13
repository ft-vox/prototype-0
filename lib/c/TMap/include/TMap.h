#pragma once

#include <stdbool.h>

#ifndef ERR_T_DEFINED
#define ERR_T_DEFINED
typedef bool err_t;
#endif

// write once, read only map
typedef struct TMap *TMap;
typedef TMap (*TMap_new)(void);
typedef err_t (*TMap_insert)(TMap map, const char *key, void *value,
                             void (*deleteValue)(void *value));
typedef void *(*TMap_search)(TMap map, const char *key);
typedef bool (*TMap_has)(TMap map, const char *key);
typedef void (*TMap_delete)(TMap self);
