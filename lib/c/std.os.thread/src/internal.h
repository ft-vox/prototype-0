#pragma once

#include "t/std.os.thread.h"

#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#endif

typedef struct ContextWrapper {
  void *actual_context;
  err_t (*actual_routine)(void *context);
} ContextWrapper;

typedef struct ThreadHandleActual {
  ThreadHandleV v;
#ifdef _WIN32
  HANDLE handle;
#else
  pthread_t handle;
#endif
  ContextWrapper contextWrapper;
  MutexHandle rcMutex;
  unsigned int rc;
} ThreadHandleActual;

typedef struct MutexHandleActual {
  MutexHandleV v;
#ifdef _WIN32
  CRITICAL_SECTION handle;
#else
  pthread_mutex_t handle;
#endif
} MutexHandleActual;

typedef struct MutexLockHandleActual {
  err_t (*unlock)(MutexLockHandle self);
#ifdef _WIN32
  CRITICAL_SECTION *handle;
#else
  pthread_mutex_t *handle;
#endif
} MutexLockHandleActual;

typedef struct ConditionVariableHandleActual {
  ConditionVariableHandleV v;
#ifdef _WIN32
  CONDITION_VARIABLE cond;
#else
  pthread_cond_t cond;
#endif
} ConditionVariableHandleActual;
