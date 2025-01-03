#include "vox/event_loop.h"

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>

#include "cross_platform_time.h"

static bool always_true(void *unused) {
  (void)unused;
  return true;
}

static vox_event_loop_task_t *test_task(void);

static bool end = false;

int main(void) {
  vox_event_loop_t *loop = vox_event_loop_new();
  assert(loop);

  cross_platform_instant_t *start = cross_platform_instant_new();
  assert(start);

  vox_event_loop_run_block(loop, always_true, NULL);
  assert(cross_platform_instant_elapsed(start) < 10);

  vox_event_loop_task_t *const task = test_task();
  assert(task);
  assert(!vox_event_loop_add_task(loop, task));

  while (!end) {
    assert(!vox_event_loop_block_while_no_task(loop, 500, NULL));
    puts("running");
    assert(!vox_event_loop_run_block(loop, always_true, NULL));
    puts("no tasks to run");
  }
  assert(cross_platform_instant_elapsed(start) < 500);
  puts("done");
}

typedef struct test_task {
  vox_event_loop_task_base_t base;
  vox_event_loop_file_handle_t *fh;
  bool succeed;
} test_task_t;

static void test_task_0(vox_event_loop_task_t *self,
                        vox_event_loop_t *event_loop);
static vox_event_loop_err_t test_task_1(vox_event_loop_task_t *self,
                                        vox_event_loop_t *event_loop,
                                        vox_event_loop_await_t *out_next);
static vox_event_loop_err_t test_task_2(vox_event_loop_task_t *self,
                                        vox_event_loop_t *event_loop,
                                        vox_event_loop_await_t *out_next);
static vox_event_loop_err_t test_task_3(vox_event_loop_task_t *self,
                                        vox_event_loop_t *event_loop,
                                        vox_event_loop_await_t *out_next);

static vox_event_loop_task_t *test_task(void) {
  test_task_t *const result = (test_task_t *)malloc(sizeof(test_task_t));
  if (!result) {
    return NULL;
  }
  result->fh = NULL;
  result->base.drop = test_task_0;
  result->base.resume = test_task_1;
  return (vox_event_loop_task_t *)((void *)result);
}

static void test_task_0(vox_event_loop_task_t *self,
                        vox_event_loop_t *event_loop) {
  test_task_t *const actual = (test_task_t *)self;
  (void)event_loop;
  vox_event_loop_async_task_file_close(actual->fh);
  free(self);
}

static vox_event_loop_err_t test_task_1(vox_event_loop_task_t *self,
                                        vox_event_loop_t *event_loop,
                                        vox_event_loop_await_t *out_next) {
  puts("task_1 start");
  (void)event_loop;
  test_task_t *const actual = (test_task_t *)self;
  vox_event_loop_async_task_t *task =
      vox_event_loop_async_task_file_open(true, "./test.txt", &actual->fh);
  assert(task);
  actual->base.resume = test_task_2;
  *out_next = (vox_event_loop_await_t){.next = self, .task = task};
  puts("task_1 end");
  return false;
}

static vox_event_loop_err_t test_task_2(vox_event_loop_task_t *self,
                                        vox_event_loop_t *event_loop,
                                        vox_event_loop_await_t *out_next) {
  puts("task_2 start");
  (void)event_loop;
  test_task_t *const actual = (test_task_t *)self;
  assert(actual->fh);
  vox_event_loop_async_task_t *task = vox_event_loop_async_task_file_write(
      actual->fh, "Hello world!\n", 13, &actual->succeed);
  assert(task);
  actual->base.resume = test_task_3;
  *out_next = (vox_event_loop_await_t){.next = self, .task = task};
  puts("task_2 end");
  return false;
}

static vox_event_loop_err_t test_task_3(vox_event_loop_task_t *self,
                                        vox_event_loop_t *event_loop,
                                        vox_event_loop_await_t *out_next) {
  puts("task_3 start");
  (void)event_loop;
  test_task_t *const actual = (test_task_t *)self;
  assert(actual->succeed);
  vox_event_loop_async_task_file_close(actual->fh);
  free(actual);
  end = true;
  *out_next = (vox_event_loop_await_t){.next = NULL, .task = NULL};
  puts("task_3 end");
  return false;
}
