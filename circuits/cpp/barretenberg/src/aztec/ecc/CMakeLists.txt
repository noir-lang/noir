barretenberg_module(ecc numeric keccak srs)

if(DISABLE_ADX)
    message(STATUS "Disabling ADX assembly variant.")
    target_compile_definitions(ecc PRIVATE -DDISABLE_ADX)
endif()

if(NOT DEFINED PIPPENGER_BLOCK_SIZE)
    set(PIPPENGER_BLOCK_SIZE "20" CACHE STRING "maximum multi-exponentiation size")
endif()
message(STATUS "Setting pippenger block size to " ${PIPPENGER_BLOCK_SIZE})
target_compile_definitions(ecc PRIVATE -DPIPPENGER_BLOCK_SIZE=${PIPPENGER_BLOCK_SIZE})

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