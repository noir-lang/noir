add_executable(honk_bench main.bench.cpp honk.bench.cpp ultra_honk.bench.cpp)

target_link_libraries(
  honk_bench
  stdlib_primitives
  stdlib_sha256
  stdlib_keccak
  stdlib_merkle_tree
  env
  benchmark::benchmark
)

add_custom_target(
    run_honk_bench
    COMMAND honk_bench
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
)