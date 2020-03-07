# barretenberg
# copyright 2019 Spilsbury Holdings

if(WASM)
    add_definitions(-DBOOST_NO_EXCEPTIONS=1)
endif()

add_definitions(-DBOOST_SPIRIT_X3_NO_FILESYSTEM=1)

set(setup_private_include_dir ${PROJECT_SOURCE_DIR}/src)

add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-unguarded-availability-new -Wno-c99-extensions")
endif()

add_executable(
    noir
    main.cpp
)

set_target_properties(
    noir
    PROPERTIES

    RUNTIME_OUTPUT_DIRECTORY .
)

target_include_directories(
    noir
    SYSTEM PRIVATE
        ${private_include_dir}/boost
)

target_link_libraries(
    noir
    PUBLIC
    barretenberg
    PRIVATE
    ${private_os_libs}
)

target_include_directories(
    noir
    PRIVATE
        ${private_include_dir}
        ${include_dir}
)
