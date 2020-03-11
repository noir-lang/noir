if(WASM)
    add_definitions(-DBOOST_NO_EXCEPTIONS)
endif()

add_definitions(-DBOOST_SPIRIT_X3_NO_FILESYSTEM)

add_subdirectory(cli)
add_subdirectory(compiler)