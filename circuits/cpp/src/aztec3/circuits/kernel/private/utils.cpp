#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/mock/mock_kernel_circuit.hpp"

#include <barretenberg/barretenberg.hpp>

#include <unistd.h>

#include <memory>

namespace {
using NT = aztec3::utils::types::NativeTypes;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::mock::mock_kernel_circuit;

}  // namespace

namespace aztec3::circuits::kernel::private_kernel::utils {

/**
 * @brief Utility for reading into a vector<uint8_t> from file
 *
 * @param filename
 * @return std::vector<uint8_t>
 */
std::vector<uint8_t> read_buffer_from_file(const std::string& filename)
{
    std::ifstream file(filename, std::ios::binary);
    std::vector<uint8_t> buf;

    if (file.is_open()) {
        // Get the file size by seeking to the end of the file
        file.seekg(0, std::ios::end);
        const std::streampos fileSize = file.tellg();
        file.seekg(0, std::ios::beg);

        // Resize the vector to hold the file contents
        buf.resize(static_cast<size_t>(fileSize));

        // Read the file contents into the vector
        file.read(reinterpret_cast<char*>(buf.data()), static_cast<std::streamsize>(fileSize));

        file.close();
    } else {
        std::cout << "Unable to open the file: " << filename << std::endl;
    }

    return buf;
}

/**
 * @brief Utility for constructing a proof from proof_data read from a file
 * @details Currently hard coded to read an UltraPlonk proof
 *
 * @return NT::Proof
 */
NT::Proof get_proof_from_file()
{
    NT::Proof proof;
    const std::string proof_data_file = "../src/aztec3/circuits/kernel/private/fixtures/ultra_plonk_proof.dat";
    proof.proof_data = read_buffer_from_file(proof_data_file);
    return proof;
}

/**
 * @brief Utility for constructing a verification key from vrification_key_data stored in file
 * @details This verification key cooresponds to the UP proof stored in ultra_plonk_proof.dat
 * @return std::shared_ptr<NT::VK>
 */
std::shared_ptr<NT::VK> get_verification_key_from_file()
{
    const std::string vk_data_file = "../src/aztec3/circuits/kernel/private/fixtures/ultra_plonk_verification_key.dat";
    auto vk_buf = utils::read_buffer_from_file(vk_data_file);
    NT::VK new_vk;
    const uint8_t* vk_iter = vk_buf.data();
    read(vk_iter, new_vk);

    return std::make_shared<NT::VK>(new_vk);
}

/**
 * @brief Create a fake verification key
 *
 * @details will not work with real circuits
 *
 * @return std::shared_ptr<NT::VK> fake verification key
 */
std::shared_ptr<NT::VK> fake_vk()
{
    std::map<std::string, NT::bn254_point> commitments;
    commitments["FAKE"] = *new NT::bn254_point(NT::fq(0), NT::fq(0));
    NT::VKData vk_data = { .circuit_type = static_cast<uint32_t>(CircuitType::ULTRA),
                           .circuit_size = 2048,
                           .num_public_inputs = 116,
                           .commitments = commitments,
                           .contains_recursive_proof = false,
                           .recursive_proof_public_input_indices = {} };
    return std::make_shared<NT::VK>(std::move(vk_data), barretenberg::srs::get_crs_factory()->get_verifier_crs());
}

/**
 * @brief Create a dummy "previous kernel"
 *
 * @details For use in the first iteration of the  kernel circuit
 *
 * @param real_vk_proof should the vk and proof included be real and usable by real circuits?
 * @return PreviousKernelData<NT> the previous kernel data for use in the kernel circuit
 */
PreviousKernelData<NT> dummy_previous_kernel(bool real_vk_proof = false)
{
    PreviousKernelData<NT> const init_previous_kernel{};

    auto crs_factory = barretenberg::srs::get_crs_factory();
    Builder mock_kernel_builder;
    auto mock_kernel_public_inputs = mock_kernel_circuit(mock_kernel_builder, init_previous_kernel.public_inputs);

    NT::Proof const mock_kernel_proof =
        real_vk_proof ? get_proof_from_file() : NT::Proof{ .proof_data = std::vector<uint8_t>(64, 0) };

    std::shared_ptr<NT::VK> const mock_kernel_vk = real_vk_proof ? get_verification_key_from_file() : fake_vk();

    PreviousKernelData<NT> previous_kernel = {
        .public_inputs = mock_kernel_public_inputs,
        .proof = mock_kernel_proof,
        .vk = mock_kernel_vk,
    };

    // TODO(rahul) assertions don't work in wasm and it isn't worth updating barratenberg to handle our error code
    // mechanism. Apparently we are getting rid of this function (dummy_previous_kernel()) soon anyway.
    assert(!mock_kernel_builder.failed());

    return previous_kernel;
}

}  // namespace aztec3::circuits::kernel::private_kernel::utils