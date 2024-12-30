#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef bool vox_event_loop_err_t;

typedef struct vox_event_loop vox_event_loop_t;

typedef struct vox_event_loop_task vox_event_loop_task_t;

typedef struct vox_event_loop_async_task vox_event_loop_async_task_t;

typedef struct {
  vox_event_loop_task_t *next;
  vox_event_loop_async_task_t *task;
} vox_event_loop_await_t;

typedef struct {
  vox_event_loop_err_t (*resume)(vox_event_loop_task_t *self,
                                 vox_event_loop_t *event_loop,
                                 vox_event_loop_await_t *out_next);
  void (*drop)(vox_event_loop_task_t *self, vox_event_loop_t *event_loop);
} vox_event_loop_task_base_t;

struct vox_event_loop_task {
  vox_event_loop_task_base_t base;
  unsigned char opaque[];
};

vox_event_loop_t *vox_event_loop_new(void);
vox_event_loop_err_t vox_event_loop_add_task(vox_event_loop_t *self,
                                             vox_event_loop_task_t *task);
bool vox_event_loop_remove_pending_task(vox_event_loop_t *self,
                                        vox_event_loop_task_t *task);
void vox_event_loop_run_block(vox_event_loop_t *self,
                              bool (*until)(void *context), void *context);

typedef struct vox_event_loop_wait_handle vox_event_loop_wait_handle_t;
vox_event_loop_err_t *
vox_event_loop_async_task_wait_new(vox_event_loop_async_task_t **out_task,
                                   vox_event_loop_wait_handle_t **out_handle);
void vox_event_loop_async_task_wait(vox_event_loop_wait_handle_t *handle);

typedef struct vox_event_loop_file_handle vox_event_loop_file_handle_t;
vox_event_loop_async_task_t *
vox_event_loop_async_task_file_open(bool create, const char *path,
                                    vox_event_loop_file_handle_t **out);
vox_event_loop_async_task_t *
vox_event_loop_async_task_file_close(vox_event_loop_file_handle_t *handle);
vox_event_loop_async_task_t *vox_event_loop_async_task_file_seek_absolute(
    vox_event_loop_file_handle_t *handle, int64_t position);
vox_event_loop_async_task_t *
vox_event_loop_async_task_file_write(vox_event_loop_file_handle_t *handle,
                                     const char *buffer, size_t buffer_length);
vox_event_loop_async_task_t *
vox_event_loop_async_task_file_read(vox_event_loop_file_handle_t *handle,
                                    size_t length, char *buffer,
                                    size_t *out_buffer_length);
