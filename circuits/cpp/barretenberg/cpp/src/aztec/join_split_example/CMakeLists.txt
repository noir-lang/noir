if(NOT WASM)
  link_libraries(leveldb)
endif()
add_subdirectory(proofs)