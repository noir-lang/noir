add_executable(
    lagrange_base_gen
    lagrange_base_processor.cpp
)

target_link_libraries(
    lagrange_base_gen
    PRIVATE
    srs
)