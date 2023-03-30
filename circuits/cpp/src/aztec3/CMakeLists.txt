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
    target_link_libraries(aztec3-circuits.wasm barretenberg)

    # With binaryen installed, it seems its wasm backend optimiser gets invoked automatically.
    # Due to either a bug in the optimiser, or non-standards compliant c++ in crypto/aes, tests start failing with
    # -O3 level optimisations. We force down to -O2 for current workaround.
    # Presumably the -O3 when compiling the object files is fine as it's llvms IR optimiser.
    # The backend optimiser is presumably triggered after linking.
    target_link_options(
        aztec3-circuits.wasm
        PRIVATE
        # TODO revisit implications of whole-archive
        -nostartfiles -O2 -Wl,--whole-archive -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576
    )

    find_program(WASM_OPT wasm-opt)

    if(NOT WASM_OPT)
        message(FATAL_ERROR "wasm-opt executable not found. Please install binaryen.")
    endif()

    add_custom_command(
        TARGET aztec3-circuits.wasm
        POST_BUILD
        COMMAND wasm-opt "$<TARGET_FILE:aztec3-circuits.wasm>" -O2 --asyncify -o "$<TARGET_FILE:aztec3-circuits.wasm>"
        VERBATIM
    )
endif()
