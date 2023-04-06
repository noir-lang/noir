barretenberg_module(stdlib_merkle_tree stdlib_primitives stdlib_blake3s stdlib_pedersen_hash)

if(NOT WASM)
    include(FetchContent)
    FetchContent_Declare(
        leveldb
        GIT_REPOSITORY https://github.com/google/leveldb.git
        GIT_TAG 1.22
        FIND_PACKAGE_ARGS
    )

    # Disable some leveldb targets before we call FetchContent_MakeAvailable
    # so they are configured correctly if it needs to fetch
    set(LEVELDB_BUILD_TESTS OFF CACHE BOOL "LevelDB tests off")
    set(LEVELDB_BUILD_BENCHMARKS OFF CACHE BOOL "LevelDB benchmarks off")

    FetchContent_MakeAvailable(leveldb)

    if (leveldb_FOUND)
        # Globally installed leveldb needs Threads available as Threads::Threads as discovered by `find_package`
        find_package(Threads REQUIRED)

        foreach(target IN LISTS stdlib_merkle_tree_lib_targets stdlib_merkle_tree_exe_targets)
            target_link_libraries(${target} PRIVATE leveldb::leveldb)
        endforeach()
    else()
        # FetchContent_MakeAvailable calls FetchContent_Populate if `find_package` is unsuccessful
        # so these variables will be available if we reach this case
        set_property(DIRECTORY ${leveldb_SOURCE_DIR} PROPERTY EXCLUDE_FROM_ALL)
        set_property(DIRECTORY ${leveldb_BINARY_DIR} PROPERTY EXCLUDE_FROM_ALL)

        # Silence all compiler warnings from LevelDB
        target_compile_options(
            leveldb
            PRIVATE
            -w
        )

        foreach(target IN LISTS stdlib_merkle_tree_lib_targets stdlib_merkle_tree_exe_targets)
            target_link_libraries(${target} PRIVATE leveldb)
        endforeach()
    endif()
endif()
