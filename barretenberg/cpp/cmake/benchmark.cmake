if(NOT TESTING)
    set(BENCHMARKS OFF)
endif()

if(BENCHMARKS)
    include(FetchContent)

    FetchContent_Declare(
        benchmark
        GIT_REPOSITORY https://github.com/google/benchmark
        GIT_TAG v1.7.1
        FIND_PACKAGE_ARGS
    )

    set(BENCHMARK_ENABLE_TESTING OFF CACHE BOOL "Benchmark tests off")
    set(BENCHMARK_ENABLE_INSTALL OFF CACHE BOOL "Benchmark installation off")

    FetchContent_MakeAvailable(benchmark)
    if(NOT benchmark_FOUND)
        # FetchContent_MakeAvailable calls FetchContent_Populate if `find_package` is unsuccessful
        # so these variables will be available if we reach this case
        set_property(DIRECTORY ${benchmark_SOURCE_DIR} PROPERTY EXCLUDE_FROM_ALL)
        set_property(DIRECTORY ${benchmark_BINARY_DIR} PROPERTY EXCLUDE_FROM_ALL)
    endif()
endif()
