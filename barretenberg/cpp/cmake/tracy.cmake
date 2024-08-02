include(FetchContent)

# Find the path where we will download the Tracy github repository
# we need this to find where the Tracy header files are for inclusion.
set(TRACY_INCLUDE "${CMAKE_BINARY_DIR}/_deps/tracy-src/public")

# Work around an issue finding threads.
set(CMAKE_THREAD_LIBS_INIT "-lpthread")

# Download the Tracy github project and do an add_subdirectory on it.
FetchContent_Declare(tracy
    GIT_REPOSITORY https://github.com/wolfpld/tracy
    GIT_TAG ffb98a972401c246b2348fb5341252e2ba855d00
)
FetchContent_MakeAvailable(tracy)
