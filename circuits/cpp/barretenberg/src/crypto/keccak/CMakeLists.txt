# copyright 2019 Spilsbury Holdings

set(
    SOURCE_FILES
    hash_types.h
    keccakf1600.c
    keccak.c
    keccak.h
)

add_library(
    keccak STATIC
    ${SOURCE_FILES}
)