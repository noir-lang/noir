set(CMAKE_C_COMPILER "/usr/local/clang_9.0.0/bin/clang-9")
set(CMAKE_CXX_COMPILER "/usr/local/clang_9.0.0/bin/clang++")

add_compile_options(-stdlib=libc++)
add_link_options(-lc++ -lc++abi)