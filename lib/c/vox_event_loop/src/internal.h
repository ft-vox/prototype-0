#pragma once

#include "vox_event_loop.h"

#define QUEUE_NODE_SIZE 1024

typedef struct vox_event_loop_queue_node {
  struct vox_event_loop_queue_node *next;
  vox_event_loop_task_t *tasks[QUEUE_NODE_SIZE];
} vox_event_loop_queue_node_t;

typedef struct vox_event_loop_queue {
  vox_event_loop_queue_node_t *head;
} vox_event_loop_queue_t;

struct vox_event_loop {
  vox_event_loop_queue_t queue;
};
