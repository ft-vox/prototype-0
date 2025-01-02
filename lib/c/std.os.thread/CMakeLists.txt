cmake_minimum_required(VERSION 3.0)
project(TStdOsThread)

set(TSTDOSTHREAD_VERSION 1.0)

include_directories(include)

add_library(TStdOsThread STATIC src/thread.c src/mutex.c src/conditionVariable.c)

target_include_directories(TStdOsThread
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

set_target_properties(TStdOsThread PROPERTIES OUTPUT_NAME "t_std.os.thread")

set_target_properties(TStdOsThread PROPERTIES VERSION ${TSTDOSTHREAD_VERSION})

install(TARGETS TStdOsThread
  EXPORT TCoreTargets
  LIBRARY DESTINATION lib
  ARCHIVE DESTINATION lib
)

install(
  FILES
    include/t/std.os.thread.h
  DESTINATION
    include/t
)
