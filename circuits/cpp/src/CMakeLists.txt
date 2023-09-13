set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion -Wno-deprecated -Wno-tautological-compare -Wfatal-errors)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    add_compile_options(-fconstexpr-steps=100000000)
    if(MEMORY_CHECKS)
        message(STATUS "Compiling with memory checks.")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address")
    endif()
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    add_compile_options(-fconstexpr-ops-limit=100000000)
endif()

include_directories(${CMAKE_SOURCE_DIR}/barretenberg/cpp/src)
include_directories(${CMAKE_CURRENT_SOURCE_DIR} ${CMAKE_SOURCE_DIR}/barretenberg/cpp/src/msgpack-c/include)

# I feel this should be limited to ecc, however it's currently used in headers that go across libraries,
# and there currently isn't an easy way to inherit the DDISABLE_SHENANIGANS parameter.
if(DISABLE_ASM)
    message(STATUS "Using fallback non-assembly methods for field multiplications.")
    add_definitions(-DDISABLE_SHENANIGANS=1)
else()
    message(STATUS "Using optimized assembly for field arithmetic.")
endif()

add_subdirectory(aztec3)