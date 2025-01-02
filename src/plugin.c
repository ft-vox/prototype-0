#define T_STD_OS_THREAD_EXPORTS
#include "t/std.os.thread.h"

#include <stddef.h>

#include "internal.h"
#include "t.h"

T_STD_OS_THREAD_API err_t plugin(T context, TMap_search search) {
  TMap_insert insert =
      (TMap_insert)search(context->map, KEY_BUILTIN_TMAP_INSERT);
  return insert(context->map, KEY_STD_OS_THREAD_THREAD_NEW, (void *)threadNew,
                NULL) ||
         insert(context->map, KEY_STD_OS_THREAD_THREAD_EXIT, (void *)threadExit,
                NULL) ||
         insert(context->map, KEY_STD_OS_THREAD_MUTEX_NEW, (void *)mutexNew,
                NULL) ||
         insert(context->map, KEY_STD_OS_THREAD_CONDITION_VARIABLE_NEW,
                (void *)conditionVariableNew, NULL);
}
