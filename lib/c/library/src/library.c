#include "Library.h"

#include <stdlib.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <dlfcn.h>
#endif

struct Library {
#ifdef _WIN32
  HMODULE lib_handle;
#else
  void *lib_handle;
#endif
};

Library LibraryOpen(const char *libname) {
#ifdef _WIN32
  HMODULE handle = LoadLibrary(libname);
  if (!handle) {
    abort();
  }
#else
  void *handle = dlopen(libname, RTLD_NOW);
  if (!handle) {
    abort();
  }
#endif
  const Library result = malloc(sizeof(struct Library));
  if (!result) {
#ifdef _WIN32
    FreeLibrary(handle);
#else
    dlclose(handle);
#endif
    abort();
  }
  result->lib_handle = handle;
  return result;
}

void *LibraryGet(Library self, const char *symbol) {
#ifdef _WIN32
  void *sym = (void *)GetProcAddress(self->lib_handle, symbol);
  if (!sym) {
    abort();
  }
#else
  void *sym = dlsym(self->lib_handle, symbol);
  if (!sym) {
    abort();
  }
#endif
  return sym;
}

void LibraryClose(Library self) {
#ifdef _WIN32
  const int result = FreeLibrary(self->lib_handle);
  if (!result) {
    abort();
  }
#else
  const int result = dlclose(self->lib_handle);
  if (result != 0) {
    abort();
  }
#endif
  free(self);
}
