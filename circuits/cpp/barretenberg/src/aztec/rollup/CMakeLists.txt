link_libraries(leveldb)

add_subdirectory(pedersen_note)
add_subdirectory(client_proofs)

if(NOT WASM)
  add_subdirectory(db_cli)
  add_subdirectory(rollup_proofs)
  add_subdirectory(rollup_cli)
  add_subdirectory(tx_factory)
endif()