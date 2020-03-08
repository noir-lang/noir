# barretenberg ecc
# copyright 2019 Spilsbury Holdings

if(DISABLE_ASM)
    message(STATUS "Using fallback non-assembly methods for field multiplications.")
    add_definitions(-DDISABLE_SHENANIGANS=1)
else()
    message(STATUS "Using optimized assembly for field arithmetic.")
endif()

if(DISABLE_ADX)
    message(STATUS "Disabling ADX assembly variant.")
    add_definitions(-DDISABLE_ADX=1)
endif()

if(NOT DEFINED PIPPENGER_BLOCK_SIZE)
    set(PIPPENGER_BLOCK_SIZE "20" CACHE STRING "maximum multi-exponentiation size")
endif()
message(STATUS "Setting pippenger block size to " ${PIPPENGER_BLOCK_SIZE})
add_definitions(-DPIPPENGER_BLOCK_SIZE=${PIPPENGER_BLOCK_SIZE})

barretenberg_module(ecc numeric keccak srs)

target_precompile_headers(
    ecc
    PUBLIC
    "$<$<COMPILE_LANGUAGE:CXX>:<random$<ANGLE-R>>"
    $<$<COMPILE_LANGUAGE:CXX>:"numeric/uint256/uint256.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"numeric/uint256/uint256_impl.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/asm_macros.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field_impl_generic.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field_impl_x64.hpp">
)