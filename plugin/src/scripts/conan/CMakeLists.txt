cmake_minimum_required (VERSION 3.0.2)

set(PROJECT_NAME_STRING "${conan.name}")

project(${PROJECT_NAME_STRING} CXX)

message(STATUS "BUILDING PROJECT : ${PROJECT_NAME}")
message(STATUS "CMAKE_GENERATOR : ${CMAKE_GENERATOR}")

include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()

find_package(Threads REQUIRED)

#
# Setup Paths
#

set(EXECUTABLE_OUTPUT_PATH ${CMAKE_BINARY_DIR}/bin)
set(FILELIST ${CMAKE_BINARY_DIR}/filelist.txt)

# Load SRC Variable from file
file(READ ${FILELIST} SRC)
if(NOT SRC)
	message(FATAL_ERROR "EMPTY FILE: ${FILELIST}")
endif()
string(REGEX REPLACE  "#.*$"  ""  SRC  ${SRC})
string(REPLACE        "\n"    ";" SRC  ${SRC})

# Source includes. We want the compiler to be more picky with these.
include_directories(
	${CMAKE_BINARY_DIR}/../../src
	src
)

#
# Compilation
#

IF (WIN32)
ELSE()
  # Theoretically fixes an issue on OSX where system headers still generate warnings.
  set(CMAKE_INCLUDE_SYSTEM_FLAG_CXX "-isystem ")
ENDIF()

IF (${conan.artifact_type} STREQUAL "lib")
	set(ARTIFACT_NAME ${conan.artifact_name})
	add_library(${conan.artifact_name} ${SRC})
ELSE()
	set(ARTIFACT_NAME ${PROJECT_NAME})
	add_executable(${PROJECT_NAME} ${SRC})
ENDIF()

set_target_properties(${ARTIFACT_NAME} PROPERTIES
    CXX_STANDARD 14
    CXX_STANDARD_REQUIRED YES
)

#
# Linking
#

target_link_libraries(${ARTIFACT_NAME} ${CONAN_LIBS})

if(THREADS_HAVE_PTHREAD_ARG)
	set_property(TARGET ${ARTIFACT_NAME} PROPERTY COMPILE_OPTIONS "-pthread")
	set_property(TARGET ${ARTIFACT_NAME} PROPERTY INTERFACE_COMPILE_OPTIONS "-pthread")
endif()
if(CMAKE_THREAD_LIBS_INIT)
	target_link_libraries(${ARTIFACT_NAME} "${CMAKE_THREAD_LIBS_INIT}")
endif()
