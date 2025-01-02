#define T_STD_OS_THREAD_EXPORTS
#include "t/std.os.thread.h"

#include <stdbool.h>
#include <stdlib.h>

#include "internal.h"

static err_t unlock(MutexLockHandle self);
static err_t try_lock(MutexHandle self, MutexLockHandle *out);
static err_t lock(MutexHandle self, MutexLockHandle *out);
static void destroy(MutexHandle self);

static const struct MutexHandleV v = {try_lock, lock, destroy};

T_STD_OS_THREAD_API MutexHandle t_std_os_thread_mutexNew(void) {
  struct MutexHandleActual *const result = malloc(sizeof(MutexHandleActual));
  if (!result) {
    return NULL;
  }
  result->v = &v;
#ifdef _WIN32
  InitializeCriticalSection(&result->handle);
#else
  if (pthread_mutex_init(&result->handle, NULL) != 0) {
    free(result);
    return NULL;
  }
#endif
  return (MutexHandle)result;
}

static err_t unlock(MutexLockHandle self) {
  MutexLockHandleActual *actual = (MutexLockHandleActual *)self;
#ifdef _WIN32
  LeaveCriticalSection(actual->handle);
#else
  if (pthread_mutex_unlock(actual->handle) != 0) {
    return true;
  }
#endif
  free(self);
  return false;
}

static err_t try_lock(MutexHandle self, MutexLockHandle *out) {
  MutexHandleActual *actual = (MutexHandleActual *)self;
  MutexLockHandleActual *result = malloc(sizeof(MutexLockHandleActual));
  if (!result) {
    return true;
  }
  result->unlock = unlock;
#ifdef _WIN32
  if (!TryEnterCriticalSection(&actual->handle)) {
    free(result);
    return true;
  }
  result->handle = &actual->handle;
#else
  if (pthread_mutex_trylock(&actual->handle) != 0) {
    free(result);
    return true;
  }
  result->handle = &actual->handle;
#endif
  *out = (MutexLockHandle)result;
  return false;
}

static err_t lock(MutexHandle self, MutexLockHandle *out) {
  MutexHandleActual *actual = (MutexHandleActual *)self;
  MutexLockHandleActual *result = malloc(sizeof(MutexLockHandleActual));
  if (!result) {
    return true;
  }
  result->unlock = unlock;
#ifdef _WIN32
  EnterCriticalSection(&actual->handle);
  result->handle = &actual->handle;
#else
  if (pthread_mutex_lock(&actual->handle) != 0) {
    free(result);
    return true;
  }
  result->handle = &actual->handle;
#endif
  *out = (MutexLockHandle)result;
  return false;
}

static void destroy(MutexHandle self) {
  MutexHandleActual *actual = (MutexHandleActual *)self;
#ifdef _WIN32
  DeleteCriticalSection(&actual->handle);
#else
  pthread_mutex_destroy(&actual->handle);
#endif
  free(actual);
}
