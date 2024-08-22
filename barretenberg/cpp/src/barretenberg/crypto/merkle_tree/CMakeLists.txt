# merkle tree is agnostic to hash function
barretenberg_module(
    crypto_merkle_tree
    lmdb
)

if (NOT FUZZING)
    # but the tests use pedersen and poseidon
    target_link_libraries(crypto_merkle_tree_tests PRIVATE stdlib_pedersen_hash stdlib_poseidon2)
    add_dependencies(crypto_merkle_tree_tests lmdb_repo)
    add_dependencies(crypto_merkle_tree_test_objects lmdb_repo)
endif()

add_dependencies(crypto_merkle_tree lmdb_repo)
add_dependencies(crypto_merkle_tree_objects lmdb_repo)

