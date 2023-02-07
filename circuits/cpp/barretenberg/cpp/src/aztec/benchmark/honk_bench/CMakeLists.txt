add_executable(honk_bench honk.bench.cpp)

target_link_libraries(
  honk_bench
  stdlib_primitives
  env
  benchmark
)

add_custom_target(
    run_honk_bench
    COMMAND honk_bench
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
)