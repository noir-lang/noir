add_executable(
    db_cli
    main.cpp
)

target_link_libraries(
    db_cli
    PRIVATE
    stdlib_merkle_tree
    env
)