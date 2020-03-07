# barretenberg ecc
# copyright 2019 Spilsbury Holdings

set(
    SOURCE_FILES
    uint512/uint512_impl.hpp
    uint512/uint512.hpp
    uint256/uint256_impl.hpp
    uint256/uint256.hpp
)

if(TESTING)
    add_executable(
        bignum_tests
        ./uint256/uint256.test.cpp
        ./uint512/uint512.test.cpp
    )

    if(WASM)
        target_link_options(
            bignum_tests
            PRIVATE
            -Wl,-z,stack-size=8388608
        )
    endif()

    target_link_libraries(
        bignum_tests
        PUBLIC
        ecc
        PRIVATE
        gtest
        gmock
        gtest_main
        ${private_os_libs}
    )
endif()