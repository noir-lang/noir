add_executable(
    rollup_cli
    main.cpp
)

target_link_libraries(
    rollup_cli
    PRIVATE
    rollup_proofs
    env
)