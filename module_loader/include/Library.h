#pragma once

typedef struct Library *Library;

Library LibraryOpen(const char *path);
void *LibraryGet(Library self, const char *name);
void LibraryClose(Library self);
