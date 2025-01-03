#pragma once

#ifdef __cplusplus
extern "C" {
#else
#include <stdbool.h>
#endif

#ifndef ERR_T_DEFINED
#define ERR_T_DEFINED
typedef bool err_t;
#endif

#include <stdbool.h>

// #if _WIN32
// #ifdef T_STD_OS_THREAD_EXPORTS
// #define T_STD_OS_THREAD_API __declspec(dllexport)
// #else
// #define T_STD_OS_THREAD_API __declspec(dllimport)
// #endif
// #else
#define T_STD_OS_THREAD_API
// #endif

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

T_STD_OS_THREAD_API ThreadHandle
t_std_os_thread_threadNew(void *context, err_t (*routine)(void *context));
T_STD_OS_THREAD_API void t_std_os_thread_threadExit(void);

T_STD_OS_THREAD_API MutexHandle t_std_os_thread_mutexNew(void);

T_STD_OS_THREAD_API ConditionVariableHandle
t_std_os_thread_conditionVariableNew(void);

#ifdef __cplusplus
}
#endif
