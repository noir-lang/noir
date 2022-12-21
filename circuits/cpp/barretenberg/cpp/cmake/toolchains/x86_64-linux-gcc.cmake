set(CMAKE_C_COMPILER "gcc")
set(CMAKE_CXX_COMPILER "g++")
# TODO(Cody): git rid of this when Adrian's work goes in
add_compile_options(-Wno-uninitialized)
add_compile_options(-Wno-maybe-uninitialized)