#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "../src/internal.h"

#include "cross_platform_time.h"

static err_t mock_thread_routine(void *context) {
  int *data = (int *)context;
  (*data)++;
  return false;
}

void test_thread_creation_and_join(void) {
  int context = 0;
  ThreadHandle thread = threadNew(&context, mock_thread_routine);
  assert(thread != NULL);

  assert(thread->v->join(thread) == false);
  assert(context == 1);
}

static MutexHandle test_mutex = NULL;

static err_t try_lock_routine(void *arg) {
  MutexLockHandle *lock_handle = (MutexLockHandle *)arg;

  assert(test_mutex->v->lock(test_mutex, lock_handle) == false);
  assert(*lock_handle != NULL);

  assert((*lock_handle)->unlock(*lock_handle) == false);
  return false;
}

void test_mutex_locking_and_multiple_locks(void) {
  test_mutex = mutexNew();
  assert(test_mutex != NULL);

  MutexLockHandle lock_handle1 = NULL;
  MutexLockHandle lock_handle2 = NULL;

  assert(test_mutex->v->lock(test_mutex, &lock_handle1) == false);
  assert(lock_handle1 != NULL);

  ThreadHandle thread = threadNew(&lock_handle2, try_lock_routine);
  assert(thread != NULL);

  cross_platform_sleep(100);

  assert(lock_handle1->unlock(lock_handle1) == false);

  assert(thread->v->join(thread) == false);

  test_mutex->v->destroy(test_mutex);
  test_mutex = NULL;
}

static ConditionVariableHandle test_cv = NULL;
static bool condition_met = false;

static err_t wait_routine(void *arg) {
  (void)arg;

  MutexLockHandle lock_handle = NULL;
  assert(test_mutex->v->lock(test_mutex, &lock_handle) == false);

  assert(test_cv->v->wait(test_cv, test_mutex) == false);

  assert(condition_met == true);

  assert(lock_handle->unlock(lock_handle) == false);
  return false;
}

void test_condition_variable_signal_and_broadcast(void) {
  test_mutex = mutexNew();
  assert(test_mutex != NULL);

  test_cv = conditionVariableNew();
  assert(test_cv != NULL);

  ThreadHandle thread1 = threadNew(NULL, wait_routine);
  assert(thread1 != NULL);

  ThreadHandle thread2 = threadNew(NULL, wait_routine);
  assert(thread2 != NULL);

  cross_platform_sleep(100);

  MutexLockHandle lock_handle = NULL;
  assert(test_mutex->v->lock(test_mutex, &lock_handle) == false);
  condition_met = true;
  assert(test_cv->v->signal(test_cv) == false);
  assert(lock_handle->unlock(lock_handle) == false);

  cross_platform_sleep(100);

  assert(test_mutex->v->lock(test_mutex, &lock_handle) == false);
  assert(test_cv->v->broadcast(test_cv) == false);
  assert(lock_handle->unlock(lock_handle) == false);

  assert(thread1->v->join(thread1) == false);
  assert(thread2->v->join(thread2) == false);

  test_mutex->v->destroy(test_mutex);
  test_mutex = NULL;
  test_cv->v->destroy(test_cv);
  test_cv = NULL;
}

static err_t signal_routine(void *arg) {
  (void)arg;

  cross_platform_sleep(100);

  MutexLockHandle lock_handle = NULL;
  assert(test_mutex->v->lock(test_mutex, &lock_handle) == false);

  assert(test_cv->v->signal(test_cv) == false);

  assert(lock_handle->unlock(lock_handle) == false);

  return false;
}

void test_condition_variable_wait_with_timeout(void) {
  test_mutex = mutexNew();
  assert(test_mutex != NULL);

  test_cv = conditionVariableNew();
  assert(test_cv != NULL);

  MutexLockHandle lock_handle = NULL;

  assert(test_mutex->v->lock(test_mutex, &lock_handle) == false);

  bool timeout_occurred;
  unsigned int timeout_millis = 200;
  cross_platform_instant_t *start_time = cross_platform_instant_new();
  int wait_result = test_cv->v->wait_with_timeout(
      test_cv, test_mutex, timeout_millis, &timeout_occurred);
  unsigned int elapsed_time = cross_platform_instant_elapsed(start_time);
  cross_platform_instant_delete(start_time);

  assert(wait_result == false);
  assert(timeout_occurred);
  assert(elapsed_time >= timeout_millis);

  assert(lock_handle->unlock(lock_handle) == false);

  ThreadHandle thread = threadNew(NULL, signal_routine);
  assert(thread != NULL);

  assert(test_mutex->v->lock(test_mutex, &lock_handle) == false);

  start_time = cross_platform_instant_new();
  wait_result = test_cv->v->wait_with_timeout(
      test_cv, test_mutex, timeout_millis, &timeout_occurred);
  elapsed_time = cross_platform_instant_elapsed(start_time);

  assert(wait_result == false);
  assert(!timeout_occurred);
  assert(elapsed_time < timeout_millis);

  assert(lock_handle->unlock(lock_handle) == false);

  assert(thread->v->join(thread) == false);

  test_cv->v->destroy(test_cv);
  test_cv = NULL;

  test_mutex->v->destroy(test_mutex);
  test_mutex = NULL;
}

static err_t exit_routine(void *arg) {
  (void)arg;
  threadExit();
  return true;
}

void test_thread_exit(void) {
  ThreadHandle thread = threadNew(NULL, exit_routine);
  assert(thread != NULL);

  assert(thread->v->join(thread) == false);

  puts("Thread exit test completed successfully");
}

int main(void) {
  puts("Running tests...");

  test_thread_creation_and_join();
  puts("test_thread_creation_and_join passed");

  test_mutex_locking_and_multiple_locks();
  puts("test_mutex_locking_and_multiple_locks passed");

  test_condition_variable_signal_and_broadcast();
  puts("test_condition_variable_signal_and_broadcast passed");

  test_condition_variable_wait_with_timeout();
  puts("test_condition_variable_wait_with_timeout passed");

  test_thread_exit();
  puts("test_thread_exit passed");

  puts("All tests passed!");
  return 0;
}
