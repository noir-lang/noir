# copyright 2019 Spilsbury Holdings

set(
    SOURCE_FILES
    blake2-impl.hpp
    blake2s.hpp
    blake2s.cpp
)

add_library(
    blake2s STATIC
    ${SOURCE_FILES}
)

if(TESTING)
    add_executable(
        blake2s_tests
        blake2s.test.cpp
    )

    target_link_libraries(
        blake2s_tests
        PUBLIC
        blake2s
        PRIVATE
        gtest
        gmock
        gtest_main
    )
endif()