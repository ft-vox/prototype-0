#include "t/std.os.thread.h"

#include <stdbool.h>
#include <stdlib.h>

#include "internal.h"

static err_t v_wait(ConditionVariableHandle self, MutexHandle mutex);
static err_t v_signal(ConditionVariableHandle self);
static err_t broadcast(ConditionVariableHandle self);
static void destroy(ConditionVariableHandle self);

static const struct ConditionVariableHandleV v = {v_wait, v_signal, broadcast,
                                                  destroy};

ConditionVariableHandle conditionVariableNew(void) {
  struct ConditionVariableHandleActual *const result =
      malloc(sizeof(ConditionVariableHandleActual));
  if (!result) {
    return NULL;
  }
  result->v = &v;
#ifdef _WIN32
  InitializeConditionVariable(&result->cond);
#else
  if (pthread_cond_init(&result->cond, NULL) != 0) {
    free(result);
    return NULL;
  }
#endif
  return (ConditionVariableHandle)result;
}

static err_t v_wait(ConditionVariableHandle self, MutexHandle mutex) {
  ConditionVariableHandleActual *actual = (ConditionVariableHandleActual *)self;
#ifdef _WIN32
  CRITICAL_SECTION *actual_mutex = &((MutexHandleActual *)mutex)->handle;
  EnterCriticalSection(actual_mutex);
  if (!SleepConditionVariableCS(&actual->cond, actual_mutex, INFINITE)) {
    LeaveCriticalSection(actual_mutex);
    return true;
  }
  LeaveCriticalSection(actual_mutex);
#else
  pthread_mutex_t *actual_mutex = &((MutexHandleActual *)mutex)->handle;
  if (pthread_mutex_lock(actual_mutex) != 0) {
    return true;
  }
  if (pthread_cond_wait(&actual->cond, actual_mutex) != 0) {
    pthread_mutex_unlock(actual_mutex);
    return true;
  }
  pthread_mutex_unlock(actual_mutex);
#endif
  return false;
}

static err_t v_signal(ConditionVariableHandle self) {
  ConditionVariableHandleActual *actual = (ConditionVariableHandleActual *)self;
#ifdef _WIN32
  WakeConditionVariable(&actual->cond);
#else
  if (pthread_cond_signal(&actual->cond) != 0) {
    return true;
  }
#endif
  return false;
}

static err_t broadcast(ConditionVariableHandle self) {
  ConditionVariableHandleActual *actual = (ConditionVariableHandleActual *)self;
#ifdef _WIN32
  WakeAllConditionVariable(&actual->cond);
#else
  if (pthread_cond_broadcast(&actual->cond) != 0) {
    return true;
  }
#endif
  return false;
}

static void destroy(ConditionVariableHandle self) {
  ConditionVariableHandleActual *actual = (ConditionVariableHandleActual *)self;
#ifdef _WIN32
  // No explicit destroy needed for CONDITION_VARIABLE in Windows
#else
  pthread_cond_destroy(&actual->cond);
#endif
  free(actual);
}
