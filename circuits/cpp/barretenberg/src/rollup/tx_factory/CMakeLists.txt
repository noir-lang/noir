# barretenberg
# copyright 2019 Spilsbury Holdings

add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-unguarded-availability-new -Wno-c99-extensions")
endif()

add_executable(
    tx_factory
    ../prover/join_split_tx.cpp
    main.cpp
)

set_target_properties(
    tx_factory
    PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY .
)

target_link_libraries(
    tx_factory
    PUBLIC
    barretenberg
    PRIVATE
    ${private_os_libs}
)

target_include_directories(
    tx_factory
    PRIVATE
    ${private_include_dir}
    ${include_dir}
)
