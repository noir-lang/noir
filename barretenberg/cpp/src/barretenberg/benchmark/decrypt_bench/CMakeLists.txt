if (NOT FUZZING) 
    add_executable(
        decrypt_bench
        main.cpp
    )

    target_link_libraries(
        decrypt_bench
        PRIVATE
        ecc
        common
    )
endif()