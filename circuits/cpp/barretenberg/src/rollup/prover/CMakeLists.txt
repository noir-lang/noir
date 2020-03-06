# barretenberg
# copyright 2019 Spilsbury Holdings

add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-unguarded-availability-new -Wno-c99-extensions")
endif()

add_executable(
    rollup_proof
    create.cpp
    destroy.cpp
    join_split.cpp
    join_split_tx.cpp
    join.cpp
    main.cpp
    note.cpp
    split.cpp
)

set_target_properties(
    rollup_proof
    PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY .
)

target_link_libraries(
    rollup_proof
    PUBLIC
    barretenberg
    PRIVATE
    leveldb
    ${private_os_libs}
)

target_include_directories(
    rollup_proof
    PRIVATE
    ${private_include_dir}
    ${include_dir}
)
