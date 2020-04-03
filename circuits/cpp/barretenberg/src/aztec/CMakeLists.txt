if(NOT WASM)
    add_compile_options(-march=native)
endif()
add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion -Wno-deprecated)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    add_compile_options(-Wno-unguarded-availability-new -Wno-c99-extensions -fconstexpr-steps=100000000)
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    add_compile_options(-Wno-deprecated-copy -fconstexpr-ops-limit=100000000)
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR})

# I feel this should be limited to ecc, however it's currently used in headers that go across libraries,
# and there currently isn't an easy way to inherit the DDISABLE_SHENANIGANS parameter.
if(DISABLE_ASM)
    message(STATUS "Using fallback non-assembly methods for field multiplications.")
    add_definitions(-DDISABLE_SHENANIGANS=1)
else()
    message(STATUS "Using optimized assembly for field arithmetic.")
endif()

add_subdirectory(env)
add_subdirectory(numeric)
add_subdirectory(srs)
add_subdirectory(ecc)
add_subdirectory(crypto)
add_subdirectory(polynomials)
add_subdirectory(plonk)
add_subdirectory(stdlib)
add_subdirectory(noir)
add_subdirectory(rollup)

if(BENCHMARKS)
    add_subdirectory(plonk_bench)
endif()

if(WASM)
    add_executable(
        barretenberg.wasm
        $<TARGET_OBJECTS:schnorr_objects>
        $<TARGET_OBJECTS:pedersen_objects>
        $<TARGET_OBJECTS:ecc_objects>
        $<TARGET_OBJECTS:proof_system_objects>
        $<TARGET_OBJECTS:rollup_client_proofs_objects>
    )

    target_link_options(
        barretenberg.wasm
        PRIVATE
        -nostartfiles -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=8388608
    )

    target_link_libraries(
        barretenberg.wasm
        PRIVATE
        rollup_client_proofs
    )
endif()