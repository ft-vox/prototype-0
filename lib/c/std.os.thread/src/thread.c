#include "t/std.os.thread.h"

#include <stdbool.h>
#include <stdlib.h>

#include "internal.h"

static err_t join(ThreadHandle self);
static err_t detach(ThreadHandle self);

#ifdef _WIN32
static DWORD routine_wrapper(void *context) {
  ContextWrapper *const context_wrapper = (ContextWrapper *)context;
  return (DWORD)context_wrapper->actual_routine(
      context_wrapper->actual_context);
}
#else
static void *routine_wrapper(void *context) {
  ContextWrapper *const context_wrapper = (ContextWrapper *)context;
  return (void *)(uintptr_t)context_wrapper->actual_routine(
      context_wrapper->actual_context);
}
#endif

static const struct ThreadHandleV v = {join, detach};

DLLEXPORT ThreadHandle
t_std_os_thread_threadNew(void *context, err_t (*routine)(void *context)) {
  struct ThreadHandleActual *const result = malloc(sizeof(ThreadHandleActual));
  if (!result) {
    return NULL;
  }
  result->v = &v;
  result->contextWrapper.actual_context = context;
  result->contextWrapper.actual_routine = routine;
#ifdef _WIN32
  result->handle =
      CreateThread(NULL, 0, routine_wrapper, &result->contextWrapper, 0, NULL);
  if (result->handle == NULL) {
    free(result);
    return NULL;
  }
#else
  if (pthread_create(&result->handle, NULL, routine_wrapper,
                     &result->contextWrapper) != 0) {
    free(result);
    return NULL;
  }
#endif
  return (ThreadHandle)result;
}

DLLEXPORT void t_std_os_thread_threadExit(void) {
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
  free(actual);
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
  free(actual);
  return false;
}
