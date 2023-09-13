if (WASM)
    link_directories(${CMAKE_SOURCE_DIR}/barretenberg/cpp/build-wasm/lib)
else()
    link_directories(${CMAKE_SOURCE_DIR}/barretenberg/cpp/build/lib)
endif()

add_subdirectory(circuits)
add_subdirectory(oracle)
add_subdirectory(dbs)
add_subdirectory(utils)

if (WASM)

    # We can't build a wasm module by just linking to the libraries as that produces, nothing.
    # There are a couple of other ways to avoiding listing all the object files here and leveraging the dependency
    # tree, but they come with the problem that they will import the 'env' object files. We explicitly want to avoid
    # that as functions in 'env' should be implemented in JS itself.
    # It turns out that just explicitly telling the wasm module which object files to include was easiest.
    add_executable(
        aztec3-circuits.wasm
        $<TARGET_OBJECTS:aztec3_circuits_apps_objects>
        $<TARGET_OBJECTS:aztec3_circuits_abis_objects>
        $<TARGET_OBJECTS:aztec3_circuits_kernel_objects>
        $<TARGET_OBJECTS:aztec3_circuits_rollup_objects>
    )
    target_link_libraries(aztec3-circuits.wasm barretenberg wasi)

    target_link_options(
        aztec3-circuits.wasm
        PRIVATE
        # TODO revisit implications of whole-archive
        -nostartfiles -Wl,--whole-archive -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576
    )
endif()
