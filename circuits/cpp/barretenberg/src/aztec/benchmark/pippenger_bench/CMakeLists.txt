add_executable(pippenger_bench main.cpp)

target_link_libraries(
  pippenger_bench
  polynomials
  plonk
  env
)

add_custom_target(
    run_pippenger_bench
    COMMAND pippenger_bench
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
)