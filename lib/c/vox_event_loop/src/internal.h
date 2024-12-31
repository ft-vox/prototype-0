#pragma once

#include "vox/event_loop.h"

#include "t/std.os.thread.h"

#define QUEUE_NODE_SIZE 1024

typedef struct vox_event_loop_queue_node {
  struct vox_event_loop_queue_node *next;
  vox_event_loop_task_t *tasks[QUEUE_NODE_SIZE];
  size_t offset;
  size_t size;
} vox_event_loop_queue_node_t;

typedef struct vox_event_loop_queue {
  vox_event_loop_queue_node_t *head;
  vox_event_loop_queue_node_t *tail;
} vox_event_loop_queue_t;

struct vox_event_loop {
  vox_event_loop_queue_t queue;
  MutexHandle mutex;
  ConditionVariableHandle condition_variable;
};

typedef vox_event_loop_err_t (*start_and_then_t)(
    vox_event_loop_async_task_t *task_to_start, vox_event_loop_t *loop,
    vox_event_loop_task_t *task_then);

struct vox_event_loop_async_task {
  start_and_then_t start_and_then;
  unsigned char opaque[];
};
