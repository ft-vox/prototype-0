#define T_STD_OS_THREAD_EXPORTS
#include "t/std.os.thread.h"

#include <stdbool.h>
#include <stdlib.h>

#include "internal.h"

static err_t join(ThreadHandle self);
static err_t detach(ThreadHandle self);

static void free_actual(struct ThreadHandleActual *self) {
  MutexLockHandle lock_handle;
  self->rcMutex->v->lock(self->rcMutex, &lock_handle);
  unsigned int rc = --self->rc;
  lock_handle->unlock(lock_handle);
  if (!rc) {
    self->rcMutex->v->destroy(self->rcMutex);
    free(self);
  }
}

#ifdef _WIN32
static DWORD routine_wrapper(void *context) {
  struct ThreadHandleActual *const actual =
      (struct ThreadHandleActual *)context;
  err_t (*const actual_routine)(void *) = actual->contextWrapper.actual_routine;
  void *const actual_context = actual->contextWrapper.actual_context;
  free_actual(actual);
  return (DWORD)actual_routine(actual_context);
}
#else
static void *routine_wrapper(void *context) {
  struct ThreadHandleActual *const actual =
      (struct ThreadHandleActual *)context;
  err_t (*const actual_routine)(void *) = actual->contextWrapper.actual_routine;
  void *const actual_context = actual->contextWrapper.actual_context;
  free_actual(actual);
  return (void *)(uintptr_t)actual_routine(actual_context);
}
#endif

static const struct ThreadHandleV v = {join, detach};

T_STD_OS_THREAD_API ThreadHandle
t_std_os_thread_threadNew(void *context, err_t (*routine)(void *context)) {
  struct ThreadHandleActual *const result = malloc(sizeof(ThreadHandleActual));
  if (!result) {
    return NULL;
  }
  result->v = &v;
  result->contextWrapper.actual_context = context;
  result->contextWrapper.actual_routine = routine;
  result->rcMutex = t_std_os_thread_mutexNew();
  result->rc = 2;
  if (!result->rcMutex) {
    free(result);
    return NULL;
  }
#ifdef _WIN32
  result->handle = CreateThread(NULL, 0, routine_wrapper, result, 0, NULL);
  if (result->handle == NULL) {
    result->rcMutex->v->destroy(result->rcMutex);
    free(result);
    return NULL;
  }
#else
  if (pthread_create(&result->handle, NULL, routine_wrapper, result) != 0) {
    result->rcMutex->v->destroy(result->rcMutex);
    free(result);
    return NULL;
  }
#endif
  return (ThreadHandle)result;
}

T_STD_OS_THREAD_API void t_std_os_thread_threadExit(void) {
#ifdef _WIN32
  ExitThread(0);
#else
  pthread_exit(NULL);
#endif
}

static err_t join(ThreadHandle self) {
  ThreadHandleActual *actual = (ThreadHandleActual *)self;
#ifdef _WIN32
  DWORD waitResult = WaitForSingleObject(actual->handle, INFINITE);
  if (waitResult != WAIT_OBJECT_0) {
    return true;
  }
  if (!CloseHandle(actual->handle)) {
    return true;
  }
#else
  if (pthread_join(actual->handle, NULL) != 0) {
    return true;
  }
#endif
  free_actual(actual);
  return false;
}

static err_t detach(ThreadHandle self) {
  ThreadHandleActual *actual = (ThreadHandleActual *)self;
#ifdef _WIN32
  if (!CloseHandle(actual->handle)) {
    return true;
  }
#else
  if (pthread_detach(actual->handle) != 0) {
    return true;
  }
#endif
  free_actual(actual);
  return false;
}
