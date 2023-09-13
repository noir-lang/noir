if (NOT(FUZZING))
    add_executable(
        grumpkin_srs_gen
        grumpkin_srs_gen.cpp
    )

    target_link_libraries(
        grumpkin_srs_gen
        PRIVATE
        srs
        ecc
        crypto_sha256
    )
endif()