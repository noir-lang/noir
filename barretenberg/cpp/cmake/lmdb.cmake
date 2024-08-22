include(ExternalProject)

set(LMDB_PREFIX "${CMAKE_BINARY_DIR}/_deps/lmdb")
set(LMDB_INCLUDE "${LMDB_PREFIX}/src/lmdb_repo/libraries/liblmdb")
set(LMDB_LIB "${LMDB_INCLUDE}/liblmdb.a")
set(LMDB_OBJECT "${LMDB_INCLUDE}/*.o")

ExternalProject_Add(
    lmdb_repo
    PREFIX ${LMDB_PREFIX}
    GIT_REPOSITORY "https://github.com/LMDB/lmdb.git"
    GIT_TAG ddd0a773e2f44d38e4e31ec9ed81af81f4e4ccbb
    BUILD_IN_SOURCE YES
    CONFIGURE_COMMAND "" # No configure step
    BUILD_COMMAND make -C libraries/liblmdb -e XCFLAGS=-fPIC liblmdb.a
    INSTALL_COMMAND ""
    UPDATE_COMMAND "" # No update step
    BUILD_BYPRODUCTS ${LMDB_LIB} ${LMDB_INCLUDE}
)

add_library(lmdb STATIC IMPORTED GLOBAL)
add_dependencies(lmdb lmdb_repo)
set_target_properties(lmdb PROPERTIES IMPORTED_LOCATION ${LMDB_LIB})

add_library(lmdb_objects OBJECT IMPORTED GLOBAL)
add_dependencies(lmdb_objects lmdb_repo)
set_target_properties(lmdb_objects PROPERTIES IMPORTED_LOCATION ${LMDB_OBJECT})
