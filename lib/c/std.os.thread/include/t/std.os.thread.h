#pragma once

#ifndef ERR_T_DEFINED
#define ERR_T_DEFINED
typedef bool err_t;
#endif

#if _WIN32
#define DLLEXPORT __declspec(dllexport)
#else
#define DLLEXPORT
#endif

typedef struct ThreadHandle *ThreadHandle;
typedef struct MutexHandle *MutexHandle;
typedef struct ConditionVariableHandle *ConditionVariableHandle;

typedef const struct ThreadHandleV {
  err_t (*join)(ThreadHandle self);
  err_t (*detach)(ThreadHandle self);
} *ThreadHandleV;
struct ThreadHandle {
  ThreadHandleV v;
  unsigned char opaque[];
};

typedef struct MutexLockHandle *MutexLockHandle;
struct MutexLockHandle {
  err_t (*unlock)(MutexLockHandle self);
  unsigned char opaque[];
};

typedef const struct MutexHandleV {
  err_t (*try_lock)(MutexHandle self, MutexLockHandle *out);
  err_t (*lock)(MutexHandle self, MutexLockHandle *out);
  void (*destroy)(MutexHandle self);
} *MutexHandleV;
struct MutexHandle {
  MutexHandleV v;
  unsigned char opaque[];
};

typedef const struct ConditionVariableHandleV {
  err_t (*wait)(ConditionVariableHandle self, MutexHandle mutex);
  err_t (*signal)(ConditionVariableHandle self);
  err_t (*broadcast)(ConditionVariableHandle self);
  void (*destroy)(ConditionVariableHandle self);
} *ConditionVariableHandleV;
struct ConditionVariableHandle {
  ConditionVariableHandleV v;
  unsigned char opaque[];
};
