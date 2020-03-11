# barretenberg
# copyright 2019 Spilsbury Holdings

if(WASM)
    add_definitions(-DBOOST_NO_EXCEPTIONS=1)
endif()

add_definitions(-DBOOST_SPIRIT_X3_NO_FILESYSTEM=1)

add_executable(
    noir_cli
    main.cpp
)

target_include_directories(
    noir_cli
    SYSTEM PRIVATE
    ${PROJECT_SOURCE_DIR}/src/boost
)

target_link_libraries(
    noir_cli
    PRIVATE
    noir
)