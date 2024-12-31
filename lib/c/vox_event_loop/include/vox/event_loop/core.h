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
vox_event_loop_err_t vox_event_loop_run_block(vox_event_loop_t *self,
                                              bool (*until)(void *context),
                                              void *context);
