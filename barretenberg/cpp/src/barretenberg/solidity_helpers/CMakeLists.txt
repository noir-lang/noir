barretenberg_module(stdlib_solidity_helpers stdlib_sha256 stdlib_blake3s stdlib_blake2s stdlib_pedersen_commitment plonk)

if (NOT(FUZZING))
  add_executable(solidity_key_gen key_gen.cpp)

  add_executable(solidity_proof_gen proof_gen.cpp)

  target_link_libraries(
    solidity_key_gen
    stdlib_solidity_helpers
  )

  target_link_libraries(
    solidity_proof_gen
    stdlib_solidity_helpers
  )
endif()