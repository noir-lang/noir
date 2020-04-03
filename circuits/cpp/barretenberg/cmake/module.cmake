# copyright 2019 Spilsbury Holdings
#
# usage: barretenberg_module(module_name [dependencies ...])
#
# Scans for all .cpp files in a subdirectory, and creates a library named <module_name>.
# Scans for all .test.cpp files in a subdirectory, and creates a gtest binary named <module name>_tests.
# Scans for all .bench.cpp files in a subdirectory, and creates a benchmark binary named <module name>_bench.
#
# We have to get a bit complicated here, due to the fact CMake will not parallelise the building of object files
# between dependent targets, due to the potential of post-build code generation steps etc.
# To work around this, we create "object libraries" containing the object files.
# Then we declare executables/libraries that are to be built from these object files.
# These assets will only be linked as their dependencies complete, but we can parallelise the compilation at least.

function(barretenberg_module MODULE_NAME)
    file(GLOB_RECURSE SOURCE_FILES *.cpp)
    file(GLOB_RECURSE HEADER_FILES *.hpp)
    list(FILTER SOURCE_FILES EXCLUDE REGEX ".*\.(test|bench).cpp$")

    if(SOURCE_FILES)
        add_library(
            ${MODULE_NAME}_objects
            OBJECT
            ${SOURCE_FILES}
        )

        add_library(
            ${MODULE_NAME}
            STATIC
            $<TARGET_OBJECTS:${MODULE_NAME}_objects>
        )

        target_link_libraries(
            ${MODULE_NAME}
            PUBLIC
            ${ARGN}
        )

        set(MODULE_LINK_NAME ${MODULE_NAME})
    endif()

    file(GLOB_RECURSE TEST_SOURCE_FILES *.test.cpp)
    if(TESTING AND TEST_SOURCE_FILES)
        add_library(
            ${MODULE_NAME}_test_objects
            OBJECT
            ${TEST_SOURCE_FILES}
        )

        target_link_libraries(
            ${MODULE_NAME}_test_objects
            PRIVATE
            gtest
        )

        add_executable(
            ${MODULE_NAME}_tests
            $<TARGET_OBJECTS:${MODULE_NAME}_test_objects>
        )

        if(WASM)
            target_link_options(
                ${MODULE_NAME}_tests
                PRIVATE
                -Wl,-z,stack-size=8388608
            )
        endif()

        target_link_libraries(
            ${MODULE_NAME}_tests
            PRIVATE
            ${MODULE_LINK_NAME}
            ${ARGN}
            gtest
            gtest_main
        )

        if(NOT WASM)
            # Currently haven't found a way to easily wrap the calls in wasmtime when run from ctest.
            gtest_discover_tests(${MODULE_NAME}_tests WORKING_DIRECTORY ${CMAKE_BINARY_DIR})
        endif()
    endif()

    file(GLOB_RECURSE BENCH_SOURCE_FILES *.bench.cpp)
    if(BENCHMARKS AND BENCH_SOURCE_FILES)
        add_library(
            ${MODULE_NAME}_bench_objects
            OBJECT
            ${BENCH_SOURCE_FILES}
        )

        target_link_libraries(
            ${MODULE_NAME}_bench_objects
            PRIVATE
            benchmark
        )

        add_executable(
            ${MODULE_NAME}_bench
            $<TARGET_OBJECTS:${MODULE_NAME}_bench_objects>
        )

        target_link_libraries(
            ${MODULE_NAME}_bench
            PRIVATE
            ${MODULE_LINK_NAME}
            ${ARGN}
            benchmark
        )

        add_custom_target(
            run_${MODULE_NAME}_bench
            COMMAND ${MODULE_NAME}_bench
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
        )
    endif()
endfunction()