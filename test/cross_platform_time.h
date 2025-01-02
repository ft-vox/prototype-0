#pragma once

#include <stdlib.h>

#if _WIN32
#include <windows.h>
#else
#include <time.h>
#include <unistd.h>
#endif

static inline void cross_platform_sleep(int milliseconds) {
#if _WIN32
  Sleep(milliseconds);
#else
  usleep(milliseconds * 1000);
#endif
}

typedef struct cross_platform_instant {
#if _WIN32
  LARGE_INTEGER start_time;
  LARGE_INTEGER frequency;
#else
  struct timespec start_time;
#endif
} cross_platform_instant_t;

cross_platform_instant_t *cross_platform_instant_new(void) {
  cross_platform_instant_t *instant =
      (cross_platform_instant_t *)malloc(sizeof(cross_platform_instant_t));
  if (!instant) {
    return NULL;
  }

#if _WIN32
  QueryPerformanceFrequency(&instant->frequency);
  QueryPerformanceCounter(&instant->start_time);
#else
  clock_gettime(CLOCK_MONOTONIC, &instant->start_time);
#endif

  return instant;
}

void cross_platform_instant_delete(cross_platform_instant_t *instant) {
  free(instant);
}

unsigned int cross_platform_instant_elapsed(cross_platform_instant_t *instant) {
#if _WIN32
  LARGE_INTEGER now;
  QueryPerformanceCounter(&now);
  LONGLONG elapsed_ticks = now.QuadPart - instant->start_time.QuadPart;
  return (unsigned int)((elapsed_ticks * 1000) / instant->frequency.QuadPart);
#else
  struct timespec now;
  clock_gettime(CLOCK_MONOTONIC, &now);
  return (unsigned int)((now.tv_sec - instant->start_time.tv_sec) * 1000 +
                        (now.tv_nsec - instant->start_time.tv_nsec) / 1000000);
#endif
}
