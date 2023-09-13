if(TESTING)
    include(GoogleTest)
    include(FetchContent)

    FetchContent_Declare(
        GTest
        GIT_REPOSITORY https://github.com/google/googletest.git
        # Version 1.12.1 is not compatible with WASI-SDK 12
        GIT_TAG release-1.10.0
        FIND_PACKAGE_ARGS
    )

    set(BUILD_GMOCK OFF CACHE BOOL "Build with gMock disabled")
    set(INSTALL_GTEST OFF CACHE BOOL "gTest installation disabled")

    FetchContent_MakeAvailable(GTest)

    if (NOT GTest_FOUND)
        # FetchContent_MakeAvailable calls FetchContent_Populate if `find_package` is unsuccessful
        # so these variables will be available if we reach this case
        set_property(DIRECTORY ${gtest_SOURCE_DIR} PROPERTY EXCLUDE_FROM_ALL)
        set_property(DIRECTORY ${gtest_BINARY_DIR} PROPERTY EXCLUDE_FROM_ALL)

        # Disable all warning when compiling gtest
        target_compile_options(
            gtest
            PRIVATE
            -w
        )

        if(WASM)
            target_compile_definitions(
                gtest
                PRIVATE
                -DGTEST_HAS_EXCEPTIONS=0
                -DGTEST_HAS_STREAM_REDIRECTION=0
            )
        endif()

        mark_as_advanced(
            BUILD_GMOCK BUILD_GTEST BUILD_SHARED_LIBS
            gmock_build_tests gtest_build_samples gtest_build_tests
            gtest_disable_pthreads gtest_force_shared_crt gtest_hide_internal_symbols
        )

        add_library(GTest::gtest ALIAS gtest)
        add_library(GTest::gtest_main ALIAS gtest_main)
    endif()

    enable_testing()
endif()
