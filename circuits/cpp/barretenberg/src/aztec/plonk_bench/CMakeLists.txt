add_executable(plonk_bench plonk.bench.cpp)

target_link_libraries(
  plonk_bench
  stdlib_field
  benchmark
)

add_custom_target(
    run_plonk_bench
    COMMAND plonk_bench
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
)