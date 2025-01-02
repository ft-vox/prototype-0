#include "vox/event_loop.h"

#include <limits.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "t/std.os.thread.h"

#include "internal.h"
#include "vox/event_loop/core.h"

struct vox_event_loop_file_handle {
  FILE *fp;
};

typedef struct file_open_task {
  start_and_then_t start_and_then;
  bool create;
  const char *path;
  vox_event_loop_file_handle_t **out;
  vox_event_loop_t *loop;
  vox_event_loop_task_t *task_then;
} file_open_task_t;

static vox_event_loop_err_t file_open_routine(void *context) {
  file_open_task_t *const task = context;
  vox_event_loop_file_handle_t *result = (vox_event_loop_file_handle_t *)malloc(
      sizeof(vox_event_loop_file_handle_t));
  if (result) {
    if (!task->create) {
      result->fp = fopen(task->path, "rb");
      if (!result->fp) {
        free(result);
        result = NULL;
      }
      if (result) {
        fclose(result->fp);
      }
    }
    if (result) {
      result->fp = fopen(task->path, "wb");
      if (!result->fp) {
        free(result);
        result = NULL;
      }
    }
  }
  *task->out = result;
  vox_event_loop_t *const loop = task->loop;
  vox_event_loop_task_t *const task_then = task->task_then;
  free(task);
  return vox_event_loop_add_task(loop, task_then);
}

static vox_event_loop_err_t
file_open_task(vox_event_loop_async_task_t *task_to_start,
               vox_event_loop_t *loop, vox_event_loop_task_t *task_then) {
  ((file_open_task_t *)task_to_start)->loop = loop;
  ((file_open_task_t *)task_to_start)->task_then = task_then;
  ThreadHandle thread =
      t_std_os_thread_threadNew(task_to_start, file_open_routine);
  if (!thread) {
    return true;
  }
  return thread->v->detach(thread);
}

vox_event_loop_async_task_t *
vox_event_loop_async_task_file_open(bool create, const char *path,
                                    vox_event_loop_file_handle_t **out) {
  file_open_task_t *const result =
      (file_open_task_t *)malloc(sizeof(file_open_task_t));
  result->start_and_then = file_open_task;
  result->create = create;
  result->path = path;
  result->out = out;
  return (vox_event_loop_async_task_t *)(void *)result;
}

typedef struct file_close_task {
  start_and_then_t start_and_then;
  vox_event_loop_file_handle_t *handle;
  vox_event_loop_t *loop;
  vox_event_loop_task_t *task_then;
} file_close_task_t;

static vox_event_loop_err_t file_close_routine(void *context) {
  file_close_task_t *const task = context;
  fclose(task->handle->fp);
  free(task->handle);
  vox_event_loop_t *const loop = task->loop;
  vox_event_loop_task_t *const task_then = task->task_then;
  free(task);
  return vox_event_loop_add_task(loop, task_then);
}

static vox_event_loop_err_t
file_close_task(vox_event_loop_async_task_t *task_to_start,
                vox_event_loop_t *loop, vox_event_loop_task_t *task_then) {
  ((file_close_task_t *)task_to_start)->loop = loop;
  ((file_close_task_t *)task_to_start)->task_then = task_then;
  ThreadHandle thread =
      t_std_os_thread_threadNew(task_to_start, file_close_routine);
  if (!thread) {
    return true;
  }
  return thread->v->detach(thread);
}

vox_event_loop_async_task_t *
vox_event_loop_async_task_file_close(vox_event_loop_file_handle_t *handle) {
  file_close_task_t *const result =
      (file_close_task_t *)malloc(sizeof(file_close_task_t));
  result->start_and_then = file_close_task;
  result->handle = handle;
  return (vox_event_loop_async_task_t *)(void *)result;
}

typedef struct file_write_task {
  start_and_then_t start_and_then;
  vox_event_loop_file_handle_t *handle;
  const char *buffer;
  size_t buffer_length;
  bool *out_succeed;
  vox_event_loop_t *loop;
  vox_event_loop_task_t *task_then;
} file_write_task_t;

static vox_event_loop_err_t file_write_routine(void *context) {
  file_write_task_t *const task = context;
  if (task->handle && task->handle->fp) {
    *task->out_succeed = fwrite(task->buffer, 1, task->buffer_length,
                                task->handle->fp) == task->buffer_length &&
                         !ferror(task->handle->fp);
  }
  vox_event_loop_t *const loop = task->loop;
  vox_event_loop_task_t *const task_then = task->task_then;
  free(task);
  return vox_event_loop_add_task(loop, task_then);
}

static vox_event_loop_err_t
file_write_task(vox_event_loop_async_task_t *task_to_start,
                vox_event_loop_t *loop, vox_event_loop_task_t *task_then) {
  ((file_write_task_t *)task_to_start)->loop = loop;
  ((file_write_task_t *)task_to_start)->task_then = task_then;
  ThreadHandle thread =
      t_std_os_thread_threadNew(task_to_start, file_write_routine);
  if (!thread) {
    return true;
  }
  return thread->v->detach(thread);
}

vox_event_loop_async_task_t *
vox_event_loop_async_task_file_write(vox_event_loop_file_handle_t *handle,
                                     const char *buffer, size_t buffer_length,
                                     bool *out_succeed) {
  file_write_task_t *const result =
      (file_write_task_t *)malloc(sizeof(file_write_task_t));
  result->start_and_then = file_write_task;
  result->handle = handle;
  result->buffer = buffer;
  result->buffer_length = buffer_length;
  result->out_succeed = out_succeed;
  return (vox_event_loop_async_task_t *)(void *)result;
}

typedef struct file_read_task {
  start_and_then_t start_and_then;
  vox_event_loop_file_handle_t *handle;
  size_t length;
  char *buffer;
  size_t *out_buffer_length;
  bool *out_succeed;
  vox_event_loop_t *loop;
  vox_event_loop_task_t *task_then;
} file_read_task_t;

static vox_event_loop_err_t file_read_routine(void *context) {
  file_read_task_t *const task = context;
  *task->out_buffer_length =
      fread(task->buffer, 1, task->length, task->handle->fp);
  *task->out_succeed = !ferror(task->handle->fp);
  vox_event_loop_t *const loop = task->loop;
  vox_event_loop_task_t *const task_then = task->task_then;
  free(task);
  return vox_event_loop_add_task(loop, task_then);
}

static vox_event_loop_err_t
file_read_task(vox_event_loop_async_task_t *task_to_start,
               vox_event_loop_t *loop, vox_event_loop_task_t *task_then) {
  ((file_read_task_t *)task_to_start)->loop = loop;
  ((file_read_task_t *)task_to_start)->task_then = task_then;
  ThreadHandle thread =
      t_std_os_thread_threadNew(task_to_start, file_read_routine);
  if (!thread) {
    return true;
  }
  return thread->v->detach(thread);
}

vox_event_loop_async_task_t *vox_event_loop_async_task_file_read(
    vox_event_loop_file_handle_t *handle, size_t length, char *buffer,
    size_t *out_buffer_length, bool *out_succeed) {
  file_read_task_t *const result =
      (file_read_task_t *)malloc(sizeof(file_read_task_t));
  result->start_and_then = file_read_task;
  result->handle = handle;
  result->length = length;
  result->buffer = buffer;
  result->out_buffer_length = out_buffer_length;
  result->out_succeed = out_succeed;
  return (vox_event_loop_async_task_t *)(void *)result;
}

typedef struct file_seek_task {
  start_and_then_t start_and_then;
  vox_event_loop_file_handle_t *handle;
  int64_t position;
  bool *out_succeed;
  vox_event_loop_t *loop;
  vox_event_loop_task_t *task_then;
} file_seek_task_t;

static vox_event_loop_err_t file_seek_routine(void *context) {
  file_seek_task_t *const task = context;
  if (task->handle && task->handle->fp) {
    if (task->position > LONG_MAX) {
      *task->out_succeed = false;
    } else {
      fseek(task->handle->fp, task->position, SEEK_SET);
      *task->out_succeed = !ferror(task->handle->fp);
    }
  }
  vox_event_loop_t *const loop = task->loop;
  vox_event_loop_task_t *const task_then = task->task_then;
  free(task);
  return vox_event_loop_add_task(loop, task_then);
}

static vox_event_loop_err_t
file_seek_task(vox_event_loop_async_task_t *task_to_start,
               vox_event_loop_t *loop, vox_event_loop_task_t *task_then) {
  ((file_seek_task_t *)task_to_start)->loop = loop;
  ((file_seek_task_t *)task_to_start)->task_then = task_then;
  ThreadHandle thread =
      t_std_os_thread_threadNew(task_to_start, file_seek_routine);
  if (!thread) {
    return true;
  }
  return thread->v->detach(thread);
}

vox_event_loop_async_task_t *vox_event_loop_async_task_file_seek_absolute(
    vox_event_loop_file_handle_t *handle, int64_t position, bool *out_succeed) {
  file_seek_task_t *const result =
      (file_seek_task_t *)malloc(sizeof(file_seek_task_t));
  result->start_and_then = file_seek_task;
  result->handle = handle;
  result->position = position;
  result->out_succeed = out_succeed;
  return (vox_event_loop_async_task_t *)(void *)result;
}
