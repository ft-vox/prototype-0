cmake_minimum_required(VERSION 3.0)
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
  EXPORT TCoreTargets
  LIBRARY DESTINATION lib
  ARCHIVE DESTINATION lib
)

install(FILES include/t.h DESTINATION include)

install(EXPORT TCoreTargets
  FILE TCoreTargets.cmake
  DESTINATION lib/cmake/TStdOsThread
)

include(CMakePackageConfigHelpers)

write_basic_package_version_file(
  "TCoreConfigVersion.cmake"
  VERSION ${TSTDOSTHREAD_VERSION}
  COMPATIBILITY AnyNewerVersion
)

install(FILES
  "${CMAKE_CURRENT_SOURCE_DIR}/TCoreConfig.cmake"
  "${CMAKE_CURRENT_BINARY_DIR}/TCoreConfigVersion.cmake"
  DESTINATION lib/cmake/TStdOsThread
)

export(EXPORT TCoreTargets
  FILE "${CMAKE_CURRENT_BINARY_DIR}/TCoreTargets.cmake"
  NAMESPACE TStdOsThread::
)

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
