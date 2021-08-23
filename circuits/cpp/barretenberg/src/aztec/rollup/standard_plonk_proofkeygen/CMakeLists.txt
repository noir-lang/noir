add_executable(
    standard_plonk_proofkeygen
    main.cpp
)

target_link_libraries(
    standard_plonk_proofkeygen
    PRIVATE
    stdlib_sha256
    stdlib_primitives
    env
)