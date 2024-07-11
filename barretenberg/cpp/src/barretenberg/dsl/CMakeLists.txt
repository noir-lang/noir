
# We do not need to bloat barretenberg.wasm with gzip functionality in a browser context as the browser can do this

set(DSL_DEPENDENCIES
    plonk
    ultra_honk
    client_ivc
    stdlib_sha256
    stdlib_aes128
    stdlib_keccak
    stdlib_poseidon2
    crypto_merkle_tree
    stdlib_schnorr
    ultra_honk
    stdlib_honk_recursion)

if (NOT WASM)
    list(APPEND DSL_DEPENDENCIES libdeflate::libdeflate_static)
endif()

barretenberg_module(
    dsl
    ${DSL_DEPENDENCIES}
)
