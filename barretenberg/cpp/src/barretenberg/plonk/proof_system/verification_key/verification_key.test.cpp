#include "verification_key.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/numeric/random/engine.hpp"

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
void expect_compressions_eq(const verification_key_data& vk0_data, const verification_key_data& vk1_data)
{
    // 0 hash index
    EXPECT_EQ(vk0_data.hash_native(0), vk1_data.hash_native(0));
    // nonzero hash index
    // EXPECT_EQ(vk0_data.hash_native(15), vk1_data.hash_native(15));
}

/**
 * @brief expect that two vk data compressions are not-equal for a few different hash indices
 *
 * @param vk0_data
 * @param vk1_data
 */
void expect_compressions_ne(const verification_key_data& vk0_data, const verification_key_data& vk1_data)
{
    EXPECT_NE(vk0_data.hash_native(0), vk1_data.hash_native(0));
    // EXPECT_NE(vk0_data.hash_native(15), vk1_data.hash_native(15));
    // ne hash indices still lead to ne compressions
    // EXPECT_NE(vk0_data.hash_native(0), vk1_data.hash_native(15));
    // EXPECT_NE(vk0_data.hash_native(14), vk1_data.hash_native(15));
}

TEST(VerificationKey, BufferSerialization)
{
    verification_key_data vk_data = rand_vk_data();

    auto buf = to_buffer(vk_data);
    auto result = from_buffer<verification_key_data>(buf);

    EXPECT_EQ(vk_data, result);
}

TEST(VerificationKey, StreamSerialization)
{
    verification_key_data vk_data = rand_vk_data();

    std::stringstream s;
    serialize::write(s, vk_data);

    verification_key_data result;
    serialize::read(static_cast<std::istream&>(s), result);

    EXPECT_EQ(vk_data, result);
}

TEST(VerificationKey, BasicCompressionEquality)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data; // copy
    expect_compressions_eq(vk0_data, vk1_data);
}

TEST(VerificationKey, CompressionInequalityIndexMismatch)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data; // copy
    // inquality on hash index mismatch
    // EXPECT_NE(vk0_data.hash_native(0), vk1_data.hash_native(15));
    // EXPECT_NE(vk0_data.hash_native(14), vk1_data.hash_native(15));
}

TEST(VerificationKey, CompressionInequalityCircuitType)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data; // copy
    vk0_data.circuit_type = static_cast<uint32_t>(CircuitType::ULTRA);
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(VerificationKey, CompressionInequalityDifferentCircuitSize)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.circuit_size = 4096;
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(VerificationKey, CompressionInequalityDifferentNumPublicInputs)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.num_public_inputs = 42;
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(VerificationKey, CompressionInequalityDifferentCommitments)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.commitments["test1"] = g1::element::random_element();
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(VerificationKey, CompressionInequalityDifferentNumCommitments)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.commitments["new"] = g1::element::random_element();
    expect_compressions_ne(vk0_data, vk1_data);
}

TEST(VerificationKey, CompressionEqualityDifferentContainsRecursiveProof)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk0_data.contains_recursive_proof = false;
    vk1_data.contains_recursive_proof = true;
    expect_compressions_eq(vk0_data, vk1_data);
}

TEST(VerificationKey, CompressionEqualityDifferentRecursiveProofPublicInputIndices)
{
    verification_key_data vk0_data = rand_vk_data();
    verification_key_data vk1_data = vk0_data;
    vk1_data.recursive_proof_public_input_indices.push_back(42);
    expect_compressions_eq(vk0_data, vk1_data);
}
} // namespace proof_system::plonk::test_verification_key
