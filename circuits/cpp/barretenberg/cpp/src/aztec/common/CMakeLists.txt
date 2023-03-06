# Collect our common/*.hpp files and include in installation
file(GLOB_RECURSE HEADER_FILES *.hpp)
target_sources(
    barretenberg_headers
    INTERFACE
    FILE_SET HEADERS
    FILES ${HEADER_FILES}
)
