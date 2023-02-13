set(CMAKE_C_COMPILER "gcc-10")
set(CMAKE_CXX_COMPILER "g++-10")
# TODO(Cody): git rid of this when Adrian's work goes in
add_compile_options(-Wno-uninitialized)
add_compile_options(-Wno-maybe-uninitialized)