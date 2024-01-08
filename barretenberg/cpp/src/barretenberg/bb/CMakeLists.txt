if (NOT(FUZZING))
    add_executable(
        bb
        main.cpp
        get_bn254_crs.cpp
        get_grumpkin_crs.cpp
    )

    target_link_libraries(
        bb
        PRIVATE
        barretenberg
        env
    )
endif()