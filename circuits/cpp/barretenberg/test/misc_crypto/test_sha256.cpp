#include <gtest/gtest.h>

#include <barretenberg/curves/bn254/fr.hpp>
#include <barretenberg/misc_crypto/sha256/sha256.hpp>

#include <iostream>
#include <memory>




TEST(misc_sha256, ror)
{
    uint32_t input =    0b000010111011111011101101110100110;
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

    std::vector<uint8_t> expected;

    expected.push_back(0xBA); expected.push_back(0x78); expected.push_back(0x16); expected.push_back(0xBF);
    expected.push_back(0x8F); expected.push_back(0x01); expected.push_back(0xCF); expected.push_back(0xEA);
    expected.push_back(0x41); expected.push_back(0x41); expected.push_back(0x40); expected.push_back(0xDE);
    expected.push_back(0x5D); expected.push_back(0xAE); expected.push_back(0x22); expected.push_back(0x23);
    expected.push_back(0xB0); expected.push_back(0x03); expected.push_back(0x61); expected.push_back(0xA3);
    expected.push_back(0x96); expected.push_back(0x17); expected.push_back(0x7A); expected.push_back(0x9C);
    expected.push_back(0xB4); expected.push_back(0x10); expected.push_back(0xFF); expected.push_back(0x61);
    expected.push_back(0xF2); expected.push_back(0x00); expected.push_back(0x15); expected.push_back(0xAD);

    for (size_t i = 0; i < 32; ++i)
    { 
        EXPECT_EQ(result[i], expected[i]);
    }
}

TEST(misc_sha256, test_NIST_vector_two)
{
    std::string input_str = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

    std::vector<uint8_t> input;
    std::copy(input_str.begin(), input_str.end(), std::back_inserter(input));
    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected;

    expected.push_back(0x24); expected.push_back(0x8D); expected.push_back(0x6A); expected.push_back(0x61);
    expected.push_back(0xD2); expected.push_back(0x06); expected.push_back(0x38); expected.push_back(0xB8);
    expected.push_back(0xE5); expected.push_back(0xC0); expected.push_back(0x26); expected.push_back(0x93);
    expected.push_back(0x0C); expected.push_back(0x3E); expected.push_back(0x60); expected.push_back(0x39);
    expected.push_back(0xA3); expected.push_back(0x3C); expected.push_back(0xE4); expected.push_back(0x59);
    expected.push_back(0x64); expected.push_back(0xFF); expected.push_back(0x21); expected.push_back(0x67);
    expected.push_back(0xF6); expected.push_back(0xEC); expected.push_back(0xED); expected.push_back(0xD4);
    expected.push_back(0x19); expected.push_back(0xDB); expected.push_back(0x06); expected.push_back(0xC1);

    for (size_t i = 0; i < 32; ++i)
    { 
        EXPECT_EQ(result[i], expected[i]);
    }
}


TEST(misc_sha256, test_NIST_vector_three)
{
    std::vector<uint8_t> input;
    input.push_back(0xbd);
    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected;

    expected.push_back(0x68); expected.push_back(0x32); expected.push_back(0x57); expected.push_back(0x20);
    expected.push_back(0xaa); expected.push_back(0xbd); expected.push_back(0x7c); expected.push_back(0x82);
    expected.push_back(0xf3); expected.push_back(0x0f); expected.push_back(0x55); expected.push_back(0x4b);
    expected.push_back(0x31); expected.push_back(0x3d); expected.push_back(0x05); expected.push_back(0x70);
    expected.push_back(0xc9); expected.push_back(0x5a); expected.push_back(0xcc); expected.push_back(0xbb);
    expected.push_back(0x7d); expected.push_back(0xc4); expected.push_back(0xb5); expected.push_back(0xaa);
    expected.push_back(0xe1); expected.push_back(0x12); expected.push_back(0x04); expected.push_back(0xc0);
    expected.push_back(0x8f); expected.push_back(0xfe); expected.push_back(0x73); expected.push_back(0x2b);

    for (size_t i = 0; i < 32; ++i)
    { 
        EXPECT_EQ(result[i], expected[i]);
    }
}


TEST(misc_sha256, test_NIST_vector_four)
{
    std::vector<uint8_t> input;
    input.push_back(0xc9);
    input.push_back(0x8c);
    input.push_back(0x8e);
    input.push_back(0x55);

    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected;

    expected.push_back(0x7a); expected.push_back(0xbc); expected.push_back(0x22); expected.push_back(0xc0);
    expected.push_back(0xae); expected.push_back(0x5a); expected.push_back(0xf2); expected.push_back(0x6c);
    expected.push_back(0xe9); expected.push_back(0x3d); expected.push_back(0xbb); expected.push_back(0x94);
    expected.push_back(0x43); expected.push_back(0x3a); expected.push_back(0x0e); expected.push_back(0x0b);
    expected.push_back(0x2e); expected.push_back(0x11); expected.push_back(0x9d); expected.push_back(0x01);
    expected.push_back(0x4f); expected.push_back(0x8e); expected.push_back(0x7f); expected.push_back(0x65);
    expected.push_back(0xbd); expected.push_back(0x56); expected.push_back(0xc6); expected.push_back(0x1c);
    expected.push_back(0xcc); expected.push_back(0xcd); expected.push_back(0x95); expected.push_back(0x04);

    for (size_t i = 0; i < 32; ++i)
    { 
        EXPECT_EQ(result[i], expected[i]);
    }
}


TEST(misc_sha256, test_NIST_vector_five)
{
    std::string input_str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    input_str += "AAAAAAAAAA";

    std::vector<uint8_t> input;
    std::copy(input_str.begin(), input_str.end(), std::back_inserter(input));
    std::vector<uint8_t> result = sha256::sha256(input);

    std::vector<uint8_t> expected;

    expected.push_back(0xc2); expected.push_back(0xe6); expected.push_back(0x86); expected.push_back(0x82);
    expected.push_back(0x34); expected.push_back(0x89); expected.push_back(0xce); expected.push_back(0xd2);
    expected.push_back(0x01); expected.push_back(0x7f); expected.push_back(0x60); expected.push_back(0x59);
    expected.push_back(0xb8); expected.push_back(0xb2); expected.push_back(0x39); expected.push_back(0x31);
    expected.push_back(0x8b); expected.push_back(0x63); expected.push_back(0x64); expected.push_back(0xf6);
    expected.push_back(0xdc); expected.push_back(0xd8); expected.push_back(0x35); expected.push_back(0xd0);
    expected.push_back(0xa5); expected.push_back(0x19); expected.push_back(0x10); expected.push_back(0x5a);
    expected.push_back(0x1e); expected.push_back(0xad); expected.push_back(0xd6); expected.push_back(0xe4);

    for (size_t i = 0; i < 32; ++i)
    { 
        EXPECT_EQ(result[i], expected[i]);
    }
}
