# Collect our common/*.hpp files and include in installation
# This is only necessary because nothing in `common/` has an
# implementation and doesn't use the `barretenberg_module` function
file(GLOB_RECURSE HEADER_FILES *.hpp)
target_sources(
    barretenberg_headers
    INTERFACE
    FILE_SET HEADERS
    FILES ${HEADER_FILES}
)
