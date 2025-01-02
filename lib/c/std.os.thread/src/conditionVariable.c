#include "t/std.os.thread.h"

#include <errno.h>
#include <stdbool.h>
#include <stdlib.h>

#include "internal.h"

static err_t v_wait(ConditionVariableHandle self, MutexHandle mutex);
static err_t v_wait_with_timeout(ConditionVariableHandle self,
                                 MutexHandle mutex, unsigned int timeout_millis,
                                 bool *out_timeout_occurred);
static err_t v_signal(ConditionVariableHandle self);
static err_t broadcast(ConditionVariableHandle self);
static void destroy(ConditionVariableHandle self);

static const struct ConditionVariableHandleV v = {v_wait, v_wait_with_timeout,
                                                  v_signal, broadcast, destroy};

DLLEXPORT ConditionVariableHandle t_std_os_thread_conditionVariableNew(void) {
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
  if (!SleepConditionVariableCS(&actual->cond, actual_mutex, INFINITE)) {
    return true;
  }
#else
  pthread_mutex_t *actual_mutex = &((MutexHandleActual *)mutex)->handle;
  if (pthread_cond_wait(&actual->cond, actual_mutex) != 0) {
    return true;
  }
#endif
  return false;
}

static err_t v_wait_with_timeout(ConditionVariableHandle self,
                                 MutexHandle mutex, unsigned int timeout_millis,
                                 bool *out_timeout_occurred) {
  ConditionVariableHandleActual *actual = (ConditionVariableHandleActual *)self;
#ifdef _WIN32
  CRITICAL_SECTION *actual_mutex = &((MutexHandleActual *)mutex)->handle;
  DWORD result =
      SleepConditionVariableCS(&actual->cond, actual_mutex, timeout_millis);
  if (result == 0) {
    DWORD error = GetLastError();
    if (error == ERROR_TIMEOUT) {
      if (out_timeout_occurred) {
        *out_timeout_occurred = true;
      }
      return false;
    }
    return true;
  }
#else
  pthread_mutex_t *actual_mutex = &((MutexHandleActual *)mutex)->handle;
  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);

  ts.tv_sec += timeout_millis / 1000;
  ts.tv_nsec += (timeout_millis % 1000) * 1000000;
  if (ts.tv_nsec >= 1000000000) {
    ts.tv_sec += 1;
    ts.tv_nsec -= 1000000000;
  }

  int result = pthread_cond_timedwait(&actual->cond, actual_mutex, &ts);
  if (result == ETIMEDOUT) {
    if (out_timeout_occurred) {
      *out_timeout_occurred = true;
    }
    return false;
  }
  if (result != 0) {
    return true;
  }
#endif
  if (out_timeout_occurred) {
    *out_timeout_occurred = false;
  }
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
