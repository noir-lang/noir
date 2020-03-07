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

set(
    SOURCE_FILES
    fields/asm_macros.hpp
    fields/field_impl_generic.hpp
    fields/field_impl_x64.hpp
    fields/field_impl.hpp
    fields/field.hpp
    fields/field2_impl.hpp
    fields/field2.hpp
    fields/field6.hpp
    fields/field12.hpp
    groups/wnaf.hpp
    groups/affine_element_impl.hpp
    groups/affine_element.hpp
    groups/element_impl.hpp
    groups/element.hpp
    groups/group_impl_asm.tcc
    groups/group_impl_int128.tcc
    groups/group.hpp
    curves/bn254/fr.hpp
    curves/bn254/fq.hpp
    curves/bn254/fq2.hpp
    curves/bn254/fq6.hpp
    curves/bn254/fq12.hpp
    curves/bn254/g1.hpp
    curves/bn254/g2.hpp
    curves/bn254/scalar_multiplication/mmu.cpp
    curves/bn254/scalar_multiplication/mmu.hpp
    curves/bn254/scalar_multiplication/process_buckets.hpp
    curves/bn254/scalar_multiplication/scalar_multiplication_unsafe.cpp
    curves/bn254/scalar_multiplication/scalar_multiplication.cpp
    curves/bn254/scalar_multiplication/scalar_multiplication.hpp
    curves/bn254/pairing_impl.hpp
    curves/bn254/pairing.hpp
    curves/grumpkin/grumpkin.hpp
)

add_library(
    ecc STATIC
    ${SOURCE_FILES}
)

target_precompile_headers(
    ecc
    PUBLIC
    "$<$<COMPILE_LANGUAGE:CXX>:<random$<ANGLE-R>>"
    $<$<COMPILE_LANGUAGE:CXX>:"bignum/uint256/uint256.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"bignum/uint256/uint256_impl.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/asm_macros.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field_impl_generic.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field_impl_x64.hpp">
)

if(WASM)
    add_executable(
        ecc.wasm
        ${SOURCE_FILES}
    )

    target_link_options(
        ecc.wasm
        PRIVATE
        -nostartfiles -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory
    )
endif()

if(TESTING)
    add_executable(
        ecc_tests
        ./curves/bn254/fr.test.cpp
        ./curves/bn254/fq.test.cpp
        ./curves/bn254/fq2.test.cpp
        ./curves/bn254/fq6.test.cpp
        ./curves/bn254/fq12.test.cpp
        ./curves/bn254/g1.test.cpp
        ./curves/bn254/g2.test.cpp
        ./curves/grumpkin/grumpkin.test.cpp
    )

    if(WASM)
        target_link_options(
            ecc_tests
            PRIVATE
            -Wl,-z,stack-size=8388608
        )
    endif()

    target_link_libraries(
        ecc_tests
        PUBLIC
        ecc
        keccak
        PRIVATE
        gtest
        gmock
        gtest_main
        ${private_os_libs}
    )
endif()