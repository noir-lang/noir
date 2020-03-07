include(cmake/kretz/OptimizeForArchitecture.cmake)
include(cmake/kretz/AddCompilerFlag.cmake)

OptimizeForArchitecture()
list(FIND _available_vector_units_list bmi2 HAS_BMI2)

if((NOT DISABLE_ASM) AND (HAS_BMI2 EQUAL -1))
    message(STATUS "Cannot detect BMI2 instruction set.")
    set(DISABLE_ASM ON)
endif()