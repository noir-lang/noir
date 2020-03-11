barretenberg_module(noir stdlib_uint)

target_include_directories(
  noir_objects
  SYSTEM PRIVATE
  ${PROJECT_SOURCE_DIR}/src/boost
)

if(TESTING)
  target_include_directories(
    noir_test_objects
    SYSTEM PRIVATE
    ${PROJECT_SOURCE_DIR}/src/boost
  )
endif()