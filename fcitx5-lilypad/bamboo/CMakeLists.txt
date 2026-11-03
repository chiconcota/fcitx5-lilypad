find_program(CARGO_EXECUTABLE cargo HINTS "$ENV{HOME}/.cargo/bin" /usr/bin /usr/local/bin)
if (NOT CARGO_EXECUTABLE)
    message(FATAL_ERROR "cargo executable not found. Please install Rust and Cargo.")
endif()

file(GLOB_RECURSE BAMBOO_CORE_RS_SRCS src/*.rs)

add_custom_command(OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/libbamboo_core.a" "${CMAKE_CURRENT_BINARY_DIR}/bamboo-core.h"
    DEPENDS ${BAMBOO_CORE_RS_SRCS} "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml" "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.lock" "${CMAKE_CURRENT_SOURCE_DIR}/include/bamboo-core.h"
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    COMMAND ${CARGO_EXECUTABLE} build --release --target-dir "${CMAKE_CURRENT_BINARY_DIR}/target"
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_CURRENT_BINARY_DIR}/target/release/libbamboo_core.a" "${CMAKE_CURRENT_BINARY_DIR}/libbamboo_core.a"
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_CURRENT_SOURCE_DIR}/include/bamboo-core.h" "${CMAKE_CURRENT_BINARY_DIR}/bamboo-core.h"
)
add_custom_target(bamboo-core DEPENDS "${CMAKE_CURRENT_BINARY_DIR}/libbamboo_core.a" "${CMAKE_CURRENT_BINARY_DIR}/bamboo-core.h")

add_library(Bamboo::Core UNKNOWN IMPORTED GLOBAL)
add_dependencies(Bamboo::Core bamboo-core)
set_target_properties(Bamboo::Core PROPERTIES
     IMPORTED_LOCATION "${CMAKE_CURRENT_BINARY_DIR}/libbamboo_core.a"
     INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_BINARY_DIR};${CMAKE_CURRENT_SOURCE_DIR}/include")
