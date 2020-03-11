include(cmake/kretz/OptimizeForArchitecture.cmake)
include(cmake/kretz/AddCompilerFlag.cmake)

OptimizeForArchitecture()
list(FIND _available_vector_units_list bmi2 HAS_BMI2)

if((NOT DISABLE_ASM) AND (HAS_BMI2 EQUAL -1))
    message(STATUS "Cannot detect BMI2 instruction set.")
    set(DISABLE_ASM ON)
endif()

if(WASM)
    # Disable SLP vectorization on WASM as it's brokenly slow. To give an idea, with this off it still takes
    # 2m:18s to compile scalar_multiplication.cpp, and with it on I estimate it's 50-100 times longer. I never
    # had the patience to wait it out...
    add_compile_options(-fno-exceptions -fno-slp-vectorize)
endif()