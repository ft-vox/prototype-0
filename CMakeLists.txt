cmake_minimum_required(VERSION 3.10)
project(TStdOsThread)

set(TSTDOSTHREAD_VERSION 1.0)

include_directories(include)

add_library(TStdOsThread SHARED src/plugin.c src/thread.c src/mutex.c src/conditionVariable.c)

target_include_directories(TStdOsThread
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
  PRIVATE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/external_headers>
)

set_target_properties(TStdOsThread PROPERTIES OUTPUT_NAME "TStdOsThread")

set_target_properties(TStdOsThread PROPERTIES VERSION ${TSTDOSTHREAD_VERSION})

install(TARGETS TStdOsThread
  LIBRARY DESTINATION lib
  ARCHIVE DESTINATION lib
)

install(FILES include/t.h DESTINATION include)

option(BUILD_TESTS "Build tests" OFF)

if(BUILD_TESTS)
  add_executable(Test
    test/main.c
  )

  target_link_libraries(Test TStdOsThread)
  target_include_directories(Test PRIVATE external_headers)

  enable_testing()

  add_test(NAME Test COMMAND Test)
endif()
