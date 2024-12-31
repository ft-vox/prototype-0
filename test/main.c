#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#include "../src/internal.h"

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

  // TODO: where is thread->v->destroy?
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

  for (int i = 0; i < 2147483647; i++) {
    continue;
  }

  assert(lock_handle1->unlock(lock_handle1) == false);

  assert(thread->v->join(thread) == false);

  test_mutex->v->destroy(test_mutex);
}

void test_condition_variable_signal_and_broadcast(void) {
  MutexHandle mutex = mutexNew();
  assert(mutex != NULL);

  ConditionVariableHandle cv = conditionVariableNew();
  assert(cv != NULL);

  MutexLockHandle lock_handle = NULL;
  assert(mutex->v->lock(mutex, &lock_handle) == false);

  assert(cv->v->signal(cv) == false);

  assert(cv->v->broadcast(cv) == false);

  assert(lock_handle->unlock(lock_handle) == false);
  mutex->v->destroy(mutex);
  cv->v->destroy(cv);
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

  printf("Thread exit test completed successfully\n");
}

int main(void) {
  printf("Running tests...\n");

  test_thread_creation_and_join();
  printf("test_thread_creation_and_join passed\n");

  test_mutex_locking_and_multiple_locks();
  printf("test_mutex_locking_and_multiple_locks passed\n");

  test_condition_variable_signal_and_broadcast();
  printf("test_condition_variable_signal_and_broadcast passed\n");

  test_thread_exit();
  printf("test_thread_exit passed\n");

  printf("All tests passed!\n");
  return 0;
}
