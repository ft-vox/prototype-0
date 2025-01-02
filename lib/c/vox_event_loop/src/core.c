#include "vox/event_loop.h"

#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "t/std.os.thread.h"

#include "internal.h"

vox_event_loop_t *vox_event_loop_new(void) {
  vox_event_loop_t *result =
      (vox_event_loop_t *)malloc(sizeof(vox_event_loop_t));
  if (!result) {
    return NULL;
  }

  result->queue.head = NULL;
  result->queue.tail = NULL;

  result->mutex = t_std_os_thread_mutexNew();
  if (!result->mutex) {
    free(result->queue.head);
    free(result);
    return NULL;
  }

  result->condition_variable = t_std_os_thread_conditionVariableNew();
  if (!result->condition_variable) {
    result->mutex->v->destroy(result->mutex);
    free(result->queue.head);
    free(result);
    return NULL;
  }

  return (vox_event_loop_t *)result;
}

vox_event_loop_err_t vox_event_loop_add_task(vox_event_loop_t *self,
                                             vox_event_loop_task_t *task) {
  MutexLockHandle lock_handle = NULL;
  if (self->mutex->v->lock(self->mutex, &lock_handle) != false) {
    return true;
  }

  if (!self->queue.tail) {
    vox_event_loop_queue_node_t *const new_node =
        (vox_event_loop_queue_node_t *)malloc(
            sizeof(vox_event_loop_queue_node_t));
    if (!new_node) {
      lock_handle->unlock(lock_handle);
      return true;
    }
    self->queue.head = new_node;
    self->queue.tail = new_node;
    new_node->next = NULL;
    new_node->offset = 0;
    new_node->size = 0;
  } else if (self->queue.tail->size == QUEUE_NODE_SIZE) {
    vox_event_loop_queue_node_t *const new_node =
        (vox_event_loop_queue_node_t *)malloc(
            sizeof(vox_event_loop_queue_node_t));
    if (!new_node) {
      lock_handle->unlock(lock_handle);
      return true;
    }
    self->queue.tail->next = new_node;
    self->queue.tail = new_node;
    new_node->next = NULL;
    new_node->offset = 0;
    new_node->size = 0;
  }
  vox_event_loop_queue_node_t *node = self->queue.tail;

  node->tasks[node->size++] = task;
  self->condition_variable->v->signal(self->condition_variable);
  lock_handle->unlock(lock_handle);
  return false;
}

vox_event_loop_err_t vox_event_loop_run_block(vox_event_loop_t *self,
                                              bool (*until)(void *context),
                                              void *context) {
  vox_event_loop_t *loop = (vox_event_loop_t *)self;

  while (until(context)) {
    MutexLockHandle lock_handle = NULL;
    if (loop->mutex->v->lock(loop->mutex, &lock_handle)) {
      return true;
    }

    vox_event_loop_queue_node_t *node = loop->queue.head;
    vox_event_loop_task_t *task = NULL;
    if (node) {
      task = node->tasks[node->offset++];
      if (node->offset == node->size) {
        if (!node->next) {
          loop->queue.head = NULL;
          loop->queue.tail = NULL;
        } else {
          loop->queue.head = node->next;
        }
        free(node);
      }
    }

    lock_handle->unlock(lock_handle);

    if (!task) {
      return false;
    }

    vox_event_loop_await_t result;
    if (task->base.resume(task, loop, &result)) {
      return true;
    }

    if (result.task) {
      if (result.task->start_and_then(result.task, self, result.next)) {
        return true;
      }
    }
  }
  return false;
}

vox_event_loop_err_t
vox_event_loop_block_while_no_task(vox_event_loop_t *self,
                                   unsigned int timeout_millis,
                                   bool *out_timeout_occurred) {
  MutexLockHandle lock_handle = NULL;
  if (self->mutex->v->lock(self->mutex, &lock_handle)) {
    return true;
  }

  if (self->condition_variable->v->wait_with_timeout(
          self->condition_variable, self->mutex, timeout_millis,
          out_timeout_occurred)) {
    lock_handle->unlock(lock_handle);
    return true;
  }

  lock_handle->unlock(lock_handle);
  return false;
}
