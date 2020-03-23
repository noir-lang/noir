add_subdirectory(hash)
add_subdirectory(encryption)
add_subdirectory(primitives)
add_subdirectory(recursion)

if(NOT WASM)
  add_subdirectory(merkle_tree)
endif()