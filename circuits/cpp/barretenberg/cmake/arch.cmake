if(WASM)
    # Disable SLP vectorization on WASM as it's brokenly slow. To give an idea, with this off it still takes
    # 2m:18s to compile scalar_multiplication.cpp, and with it on I estimate it's 50-100 times longer. I never
    # had the patience to wait it out...
    add_compile_options(-fno-exceptions -fno-slp-vectorize)
endif()

if(NOT WASM)
    add_compile_options(-march=sandybridge)
endif()