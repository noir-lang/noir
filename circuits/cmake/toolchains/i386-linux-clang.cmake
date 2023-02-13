# Sometimes we need to set compilers manually, for example for fuzzing
if(NOT CMAKE_C_COMPILER)
    set(CMAKE_C_COMPILER "clang")
endif()

if(NOT CMAKE_CXX_COMPILER)
    set(CMAKE_CXX_COMPILER "clang++")
endif()

add_compile_options("-m32")
add_link_options("-m32")
set(MULTITHREADING OFF)
add_definitions(-DDISABLE_SHENANIGANS=1)