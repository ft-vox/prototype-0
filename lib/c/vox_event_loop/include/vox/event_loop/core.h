#pragma once

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#include <cstddef>
#else
#include <stdbool.h>
#include <stddef.h>
#endif

typedef bool vox_event_loop_err_t;

typedef struct vox_event_loop vox_event_loop_t;

typedef struct vox_event_loop_task vox_event_loop_task_t;

typedef struct vox_event_loop_async_task vox_event_loop_async_task_t;

typedef struct {
  vox_event_loop_async_task_t *task;
  vox_event_loop_task_t *next; // if task is NULL, discard (not dispose) next
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
vox_event_loop_err_t vox_event_loop_run_block(vox_event_loop_t *self,
                                              bool (*until)(void *context),
                                              void *context);
vox_event_loop_err_t
vox_event_loop_block_while_no_task(vox_event_loop_t *self,
                                   unsigned int timeout_millis,
                                   bool *out_timeout_occurred);

#ifdef __cplusplus
}
#endif
