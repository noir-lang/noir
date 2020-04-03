# barretenberg
# copyright 2019 Spilsbury Holdings

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
    env
)