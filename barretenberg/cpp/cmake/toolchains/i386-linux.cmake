set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR i386)

add_compile_options("-m32")
add_link_options("-m32")
set(MULTITHREADING OFF)
add_definitions(-DDISABLE_ASM=1)
