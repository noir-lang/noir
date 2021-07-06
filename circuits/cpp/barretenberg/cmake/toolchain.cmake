if(NOT TOOLCHAIN)
  set(TOOLCHAIN "x86_64-linux-clang" CACHE STRING "Build toolchain." FORCE)
endif()
message(STATUS "Toolchain: ${TOOLCHAIN}")

include("./cmake/toolchains/${TOOLCHAIN}.cmake")