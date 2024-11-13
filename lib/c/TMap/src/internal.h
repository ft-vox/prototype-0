#pragma once

#include <stdbool.h>

#ifndef ERR_T_DEFINED
#define ERR_T_DEFINED
typedef bool err_t;
#endif

// write once, read only map
typedef struct TMap *TMap;
TMap TMap_new(void);
err_t TMap_insert(TMap map, const char *key, void *value,
                  void (*deleteValue)(void *value));
void *TMap_search(TMap map, const char *key);
bool TMap_has(TMap map, const char *key);
void TMap_delete(TMap self);
