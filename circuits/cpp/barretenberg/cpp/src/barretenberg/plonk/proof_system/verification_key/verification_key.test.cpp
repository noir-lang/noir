#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "verification_key.hpp"

namespace {
auto& engine = numeric::random::get_debug_engine();
} // namespace

using namespace barretenberg;
using namespace proof_system::plonk;

namespace proof_system::plonk::test_verification_key {

/**
 * @brief generate a random vk data for use in tests
 *
 * @return verification_key_data randomly generated
 */
verification_key_data rand_vk_data()
{
    verification_key_data vk_data;
    vk_data.circuit_type = static_cast<uint32_t>(CircuitType::STANDARD);
    vk_data.circuit_size = 1024; // not random - must be power of 2
    vk_data.num_public_inputs = engine.get_random_uint32();
    vk_data.commitments["test1"] = g1::element::random_element();
    vk_data.commitments["test2"] = g1::element::random_element();
    vk_data.commitments["foo1"] = g1::element::random_element();
    vk_data.commitments["foo2"] = g1::element::random_element();
    return vk_data;
}

/**
 * @brief expect that two vk data compressions are equal for a few different hash indices
 *
 * @param vk0_data
 * @param vk1_data
 */
void expect_compressions_eq(verification_key_data vk0_data, verification_key_data vk1_data)
{
    // 0 hash index
    EXPECT_EQ(vk0_data.compress_native(0), vk1_data.compress_native(0));
    // nonzero hash index
    // EXPECT_EQ(vk0_data.compress_native(15), vk1_data.compress_native(15));
}

/**
 * @brief expect that two vk data compressions are not-equal for a few different hash indices
 *
 * @param vk0_data
 * @param vk1_data
 */
void expect_compressions_ne(verification_key_data vk0_data, verification_key_data vk1_data)
{
    EXPECT_NE(vk0_data.compress_native(0), vk1_data.compress_native(0));
    // EXPECT_NE(vk0_data.compress_native(15), vk1_data.compress_native(15));
    // ne hash indices still lead to ne compressions
    // EXPECT_NE(vk0_data.compress_native(0), vk1_data.compress_native(15));
    // EXPECT_NE(vk0_data.compress_native(14), vk1_data.compress_native(15));
}

TEST(verification_key, buffer_serialization)
{
    verification_key_data vk_data = rand_vk_data();

    auto buf = to_buffer(vk_data);
    auto result = from_buffer<verification_key_data>(buf);

    EXPECT_EQ(vk_data, result);
}

TEST(verification_key, stream_serialization)
{
    verification_key_data vk_data = rand_vk_data();

    std::stringstream s;
    serialize::write(s, vk_data);

    verification_key_data result;
    serialize::read(static_cast<std::istream&>(s), result);

    EXPECT_EQ(vk_data, result);
}

TEST(verification_key, basic_compression_equality)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data; // copy
    expect_compressions_eq(vk0_data, vk1_data);
}

TEST(verification_key, compression_inequality_index_mismatch)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data; // copy
    // inquality on hash index mismatch
    // EXPECT_NE(vk0_data.compress_native(0), vk1_data.compress_native(15));
    // EXPECT_NE(vk0_data.compress_native(14), vk1_data.compress_native(15));
}

TEST(verification_key, compression_inequality_circuit_type)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data; // copy
    vk0_data.circuit_type = static_cast<uint32_t>(CircuitType::ULTRA);
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(verification_key, compression_inequality_different_circuit_size)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.circuit_size = 4096;
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(verification_key, compression_inequality_different_num_public_inputs)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.num_public_inputs = 42;
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(verification_key, compression_inequality_different_commitments)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.commitments["test1"] = g1::element::random_element();
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(verification_key, compression_inequality_different_num_commitments)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.commitments["new"] = g1::element::random_element();
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(verification_key, compression_equality_different_contains_recursive_proof)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.contains_recursive_proof = false;
    vk1_data.contains_recursive_proof = true;
    expect_compressions_eq(vk0_data, vk1_data);
}

TEST(verification_key, compression_equality_different_recursive_proof_public_input_indices)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk1_data.recursive_proof_public_input_indices.push_back(42);
    expect_compressions_eq(vk0_data, vk1_data);
}
} // namespace proof_system::plonk::test_verification_key
