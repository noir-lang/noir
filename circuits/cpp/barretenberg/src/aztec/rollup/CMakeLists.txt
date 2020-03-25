add_subdirectory(pedersen_note)
add_subdirectory(tx)
add_subdirectory(client_proofs)

if(NOT WASM)
  add_subdirectory(prover)
  add_subdirectory(tx_factory)
  add_subdirectory(cli)
endif()