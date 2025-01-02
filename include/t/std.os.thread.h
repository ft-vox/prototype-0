#pragma once

#include "t.h"

#include <stdbool.h>

#if _WIN32
#define DLLEXPORT __declspec(dllexport)
#else
#define DLLEXPORT
#endif

DLLEXPORT err_t plugin(T context, TMap_search search);

typedef struct ThreadHandle *ThreadHandle;
typedef struct MutexHandle *MutexHandle;
typedef struct ConditionVariableHandle *ConditionVariableHandle;

typedef const struct ThreadHandleV {
  err_t (*join)(ThreadHandle self);
  err_t (*detach)(ThreadHandle self);
} *ThreadHandleV;
struct ThreadHandle {
  ThreadHandleV v;
  unsigned char opaque[];
};

typedef struct MutexLockHandle *MutexLockHandle;
struct MutexLockHandle {
  err_t (*unlock)(MutexLockHandle self);
  unsigned char opaque[];
};

typedef const struct MutexHandleV {
  err_t (*try_lock)(MutexHandle self, MutexLockHandle *out);
  err_t (*lock)(MutexHandle self, MutexLockHandle *out);
  void (*destroy)(MutexHandle self);
} *MutexHandleV;
struct MutexHandle {
  MutexHandleV v;
  unsigned char opaque[];
};

typedef const struct ConditionVariableHandleV {
  err_t (*wait)(ConditionVariableHandle self, MutexHandle mutex);
  err_t (*wait_with_timeout)(ConditionVariableHandle self, MutexHandle mutex,
                             unsigned int timeout_millis,
                             bool *out_timeout_occurred);
  err_t (*signal)(ConditionVariableHandle self);
  err_t (*broadcast)(ConditionVariableHandle self);
  void (*destroy)(ConditionVariableHandle self);
} *ConditionVariableHandleV;
struct ConditionVariableHandle {
  ConditionVariableHandleV v;
  unsigned char opaque[];
};

#define KEY_STD_OS_THREAD_THREAD_NEW "std.os.thread.thread_new"
#define KEY_STD_OS_THREAD_THREAD_EXIT "std.os.thread.thread_exit"
#define KEY_STD_OS_THREAD_MUTEX_NEW "std.os.thread.mutex_new"
#define KEY_STD_OS_THREAD_CONDITION_VARIABLE_NEW                               \
  "std.os.thread.condition_variable_new"
