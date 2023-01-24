if(NOT TESTING)
    set(BENCHMARKS OFF)
endif()

if(BENCHMARKS)
    include(FetchContent)

    FetchContent_Declare(
        benchmark
        GIT_REPOSITORY https://github.com/google/benchmark
        GIT_TAG v1.6.1
    )

    FetchContent_GetProperties(benchmark)
    if(NOT benchmark_POPULATED)
        fetchcontent_populate(benchmark)
        set(BENCHMARK_ENABLE_TESTING OFF CACHE BOOL "Benchmark tests off")
        add_subdirectory(${benchmark_SOURCE_DIR} ${benchmark_BINARY_DIR} EXCLUDE_FROM_ALL)
    endif()
endif()
