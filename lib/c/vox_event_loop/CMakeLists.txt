cmake_minimum_required(VERSION 3.0)
project(VoxEventLoop)

set(VOXEVENTLOOP_VERSION 1.0)

include_directories(include)

add_library(VoxEventLoop STATIC src/core.c src/file.c)

target_include_directories(VoxEventLoop
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
  PRIVATE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../std.os.thread/include>
)

set_target_properties(VoxEventLoop PROPERTIES OUTPUT_NAME "VoxEventLoop")

set_target_properties(VoxEventLoop PROPERTIES VERSION ${VOXEVENTLOOP_VERSION})

install(TARGETS VoxEventLoop
  EXPORT VoxEventLoop
  LIBRARY DESTINATION lib
  ARCHIVE DESTINATION lib
)

install(
  FILES
    include/vox/event_loop.h
  DESTINATION
    include/t/vox
)

install(
  FILES
    include/vox/event_loop/core.h
  DESTINATION
    include/t/vox/event_loop
)

install(
  FILES
    include/vox/event_loop/async_tasks/file.h
  DESTINATION
    include/t/vox/event_loop/async_tasks
)

option(BUILD_TESTS "Build tests" OFF)

if(BUILD_TESTS)
  add_executable(Test
    test/main.c
  )

  target_link_libraries(Test VoxEventLoop t_std.os.thread)
  target_link_directories(Test PRIVATE dependencies/lib)
  target_include_directories(Test PRIVATE external_headers dependencies/include)

  enable_testing()

  add_test(NAME Test COMMAND Test)
endif()
