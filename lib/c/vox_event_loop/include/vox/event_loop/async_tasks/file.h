#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "../core.h"

typedef struct vox_event_loop_file_handle vox_event_loop_file_handle_t;

vox_event_loop_async_task_t *
vox_event_loop_async_task_file_open(bool create, const char *path,
                                    vox_event_loop_file_handle_t **out);
void vox_event_loop_async_task_file_close(vox_event_loop_file_handle_t *handle);
vox_event_loop_async_task_t *vox_event_loop_async_task_file_seek_absolute(
    vox_event_loop_file_handle_t *handle, int64_t position, bool *out_succeed);
vox_event_loop_async_task_t *
vox_event_loop_async_task_file_write(vox_event_loop_file_handle_t *handle,
                                     const char *buffer, size_t buffer_length,
                                     bool *out_succeed);
vox_event_loop_async_task_t *vox_event_loop_async_task_file_read(
    vox_event_loop_file_handle_t *handle, size_t length, char *buffer,
    size_t *out_buffer_length, bool *out_succeed);
