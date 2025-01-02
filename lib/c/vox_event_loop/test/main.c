#include "vox/event_loop.h"

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>

#include "cross_platform_time.h"

static bool always_true(void *unused) {
  (void)unused;
  return true;
}

int main(void) {
  vox_event_loop_t *loop = vox_event_loop_new();
  assert(loop != NULL);

  cross_platform_instant_t *start = cross_platform_instant_new();
  assert(start != NULL);

  vox_event_loop_run_block(loop, always_true, NULL);
  assert(cross_platform_instant_elapsed(start) < 100);
}
