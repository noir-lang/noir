FetchContent_Declare(
    leveldb
    GIT_REPOSITORY https://github.com/google/leveldb.git
    GIT_TAG 1.22
)

set(LEVELDB_BUILD_TESTS OFF)

FetchContent_GetProperties(leveldb)
if(NOT leveldb_POPULATED)
    FetchContent_Populate(leveldb)
    add_subdirectory(${leveldb_SOURCE_DIR} ${leveldb_BINARY_DIR} EXCLUDE_FROM_ALL)
endif()

target_compile_options(leveldb PRIVATE -Wno-sign-conversion -Wno-unused-parameter -Wno-shorten-64-to-32 -Wno-implicit-int-conversion -Wno-conversion -Wno-implicit-fallthrough)

barretenberg_module(stdlib_merkle_tree stdlib_byte_array stdlib_field stdlib_blake2s stdlib_pedersen leveldb)

# In order to ensure we can get the leveldb includes. Better way?
target_link_libraries(
    stdlib_merkle_tree_objects
    PUBLIC
    leveldb
)

target_link_libraries(
    stdlib_merkle_tree
    PUBLIC
    leveldb
)

target_link_libraries(
    stdlib_merkle_tree_test_objects
    PUBLIC
    leveldb
)
