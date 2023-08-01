if (NOT(FUZZING))
    add_executable(
        bb
        main.cpp
    )

    target_link_libraries(
        bb
        PRIVATE
        barretenberg
        env
    )
endif()