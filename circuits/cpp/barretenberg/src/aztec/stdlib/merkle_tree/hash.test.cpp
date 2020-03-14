#include "hash.hpp"
#include <gtest/gtest.h>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/types/turbo.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

TEST(stdlib_merkle_tree, compress_native_vs_circuit)
{
    fr x = uint256_t(0x5ec473eb273a8011, 0x50160109385471ca, 0x2f3095267e02607d, 0x02586f4a39e69b86);
    Composer composer = Composer();
    witness_ct y = witness_ct(&composer, x);
    auto z = plonk::stdlib::pedersen::compress(y, y);
    auto zz = crypto::pedersen::compress_native(x, x);
    EXPECT_EQ(z.get_value(), zz);
}

TEST(stdlib_merkle_tree, hash_value_native_vs_circuit)
{
    std::string x = std::string(64, '\1');
    Composer composer = Composer();
    byte_array_ct y(&composer, x);
    field_ct z = plonk::stdlib::merkle_tree::hash_value(y);
    fr zz = plonk::stdlib::merkle_tree::hash_value_native(x);
    EXPECT_EQ(z.get_value(), zz);
}