barretenberg_module(crypto_blake3s_full)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    # Necessary options to get compilation working in WASM,
    # where size_t is 32 bits
    #
    # Fixing this error would be more desirable than just disabling the error,
    # but since this is a reference implementation we trust that the code is 
    # correct and that an overflow does not happen. 
    target_compile_options(crypto_blake3s_full_objects PRIVATE -Wno-shorten-64-to-32)
endif()