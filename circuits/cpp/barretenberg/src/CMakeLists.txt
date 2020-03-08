add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion -Wno-deprecated)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    add_compile_options(-Wno-unguarded-availability-new -Wno-c99-extensions -fconstexpr-steps=100000000)
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    add_compile_options(-Wno-deprecated-copy -fconstexpr-ops-limit=100000000)
endif()

if(WASM)
    set(WASI_SDK_PREFIX "${CMAKE_CURRENT_SOURCE_DIR}/wasi-sdk-8.0")
    add_compile_options(-fno-exceptions)
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR})

add_subdirectory(numeric)
add_subdirectory(srs)
add_subdirectory(ecc)
add_subdirectory(crypto)
add_subdirectory(polynomials)
add_subdirectory(plonk)
#add_subdirectory(barretenberg)
#add_subdirectory(noir)
#add_subdirectory(rollup/prover)
#add_subdirectory(rollup/tx_factory)
