#pragma once

#if _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

static inline void cross_platform_sleep(int milliseconds) {
#if _WIN32
  Sleep(milliseconds);
#else
  usleep(milliseconds * 1000);
#endif
}
