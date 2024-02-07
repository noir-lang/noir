include(GoogleTest)
include(FetchContent)

set(BUILD_GMOCK ON CACHE INTERNAL BOOL "Build with gMock enabled")
set(INSTALL_GTEST OFF CACHE BOOL "gTest installation disabled")

FetchContent_Declare(
    GTest
    GIT_REPOSITORY https://github.com/google/googletest.git
    GIT_TAG v1.13.0 #v1.14.0 does not compile with gcc (compiler bug: https://gcc.gnu.org/bugzilla/show_bug.cgi?id=105329)
    FIND_PACKAGE_ARGS
)

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

endif()

enable_testing()
