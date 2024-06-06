if(CHECK_CIRCUIT_STACKTRACES)
    include(FetchContent)

    # Also requires one of: libbfd (gnu binutils), libdwarf, libdw (elfutils)
    FetchContent_Declare(backward
        GIT_REPOSITORY https://github.com/bombela/backward-cpp
        GIT_TAG 51f0700452cf71c57d43c2d028277b24cde32502
        SYSTEM          # optional, the Backward include directory will be treated as system directory
    )
    FetchContent_MakeAvailable(backward)
endif()