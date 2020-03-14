#include "sha256.hpp"
#include <gtest/gtest.h>
#include <iostream>
#include <memory>

TEST(misc_sha256, ror)
{
    uint32_t input = 0b000010111011111011101101110100110;
    uint32_t expected = 0b01001100001011101111101110110111;
    uint32_t result = sha256::ror(input, 7);
    EXPECT_EQ(result, expected);
}

TEST(misc_sha256, test_NIST_vector_one)
{
    std::string input_str = "abc";

    std::vector<uint8_t> input;
    std::copy(input_str.begin(), input_str.end(), std::back_inserter(input));
    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected{
        0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE, 0x5D, 0xAE, 0x22, 0x23,
        0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17, 0x7A, 0x9C, 0xB4, 0x10, 0xFF, 0x61, 0xF2, 0x00, 0x15, 0xAD,
    };

    for (size_t i = 0; i < 32; ++i) {
        EXPECT_EQ(result[i], expected[i]);
    }
}

TEST(misc_sha256, test_NIST_vector_two)
{
    std::string input_str = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

    std::vector<uint8_t> input;
    std::copy(input_str.begin(), input_str.end(), std::back_inserter(input));
    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected{
        0x24, 0x8D, 0x6A, 0x61, 0xD2, 0x06, 0x38, 0xB8, 0xE5, 0xC0, 0x26, 0x93, 0x0C, 0x3E, 0x60, 0x39,
        0xA3, 0x3C, 0xE4, 0x59, 0x64, 0xFF, 0x21, 0x67, 0xF6, 0xEC, 0xED, 0xD4, 0x19, 0xDB, 0x06, 0xC1,
    };

    for (size_t i = 0; i < 32; ++i) {
        EXPECT_EQ(result[i], expected[i]);
    }
}

TEST(misc_sha256, test_NIST_vector_three)
{
    std::vector<uint8_t> input;
    input.push_back(0xbd);
    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected{
        0x68, 0x32, 0x57, 0x20, 0xaa, 0xbd, 0x7c, 0x82, 0xf3, 0x0f, 0x55, 0x4b, 0x31, 0x3d, 0x05, 0x70,
        0xc9, 0x5a, 0xcc, 0xbb, 0x7d, 0xc4, 0xb5, 0xaa, 0xe1, 0x12, 0x04, 0xc0, 0x8f, 0xfe, 0x73, 0x2b,
    };

    for (size_t i = 0; i < 32; ++i) {
        EXPECT_EQ(result[i], expected[i]);
    }
}

TEST(misc_sha256, test_NIST_vector_four)
{
    std::vector<uint8_t> input{ 0xc9, 0x8c, 0x8e, 0x55 };

    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected{
        0x7a, 0xbc, 0x22, 0xc0, 0xae, 0x5a, 0xf2, 0x6c, 0xe9, 0x3d, 0xbb, 0x94, 0x43, 0x3a, 0x0e, 0x0b,
        0x2e, 0x11, 0x9d, 0x01, 0x4f, 0x8e, 0x7f, 0x65, 0xbd, 0x56, 0xc6, 0x1c, 0xcc, 0xcd, 0x95, 0x04,
    };

    for (size_t i = 0; i < 32; ++i) {
        EXPECT_EQ(result[i], expected[i]);
    }
}

TEST(misc_sha256, test_NIST_vector_five)
{
    std::string input_str =
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAA";

    std::vector<uint8_t> input;
    std::copy(input_str.begin(), input_str.end(), std::back_inserter(input));
    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected{
        0xc2, 0xe6, 0x86, 0x82, 0x34, 0x89, 0xce, 0xd2, 0x01, 0x7f, 0x60, 0x59, 0xb8, 0xb2, 0x39, 0x31,
        0x8b, 0x63, 0x64, 0xf6, 0xdc, 0xd8, 0x35, 0xd0, 0xa5, 0x19, 0x10, 0x5a, 0x1e, 0xad, 0xd6, 0xe4,
    };

    for (size_t i = 0; i < 32; ++i) {
        EXPECT_EQ(result[i], expected[i]);
    }
}
