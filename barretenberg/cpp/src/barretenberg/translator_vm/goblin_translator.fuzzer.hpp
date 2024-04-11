/**
 * @file goblin_translator.fuzzer.hpp
 * @author Rumata888
 * @brief Contains common procedures used by the circuit builder fuzzer and the composer fuzzer
 * @date 2024-02-25
 *
 */
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "goblin_translator_circuit_builder.hpp"

using namespace bb;

using Fr = curve::BN254::ScalarField;
using Fq = curve::BN254::BaseField;
using G1 = curve::BN254::AffineElement;

/**
 * @brief Parse raw operations for ECCOpQueue from the data stream
 *
 * @param data pointer to data
 * @param size size in bytes
 * @return std::vector<ECCOpQueue::ECCVMOperation>
 */
std::vector<ECCOpQueue::ECCVMOperation> parse_operations(const unsigned char* data, size_t size)
{
    std::vector<ECCOpQueue::ECCVMOperation> raw_ops;

    size_t size_left = size;
    // Just iterate and parse until there's no data left
    while (size_left >= sizeof(ECCOpQueue::ECCVMOperation)) {
        raw_ops.emplace_back((ECCOpQueue::ECCVMOperation*)(data + (size - size_left)));
        size_left -= sizeof(ECCOpQueue::ECCVMOperation);
    }
    return raw_ops;
}

/**
 * @brief Try to parse out the batching and evaluating challenges and then the ECCOpQueue from the data
 *
 * @param data pointer to the buffer
 * @param size size of the buffer
 * @return std::optional<std::tuple<Fq, Fq, std::shared_ptr<ECCOpQueue>>>
 */
std::optional<std::tuple<Fq, Fq, std::shared_ptr<ECCOpQueue>>> parse_and_construct_opqueue(const unsigned char* data,
                                                                                           size_t size)
{
    std::vector<ECCOpQueue::ECCVMOperation> raw_ops;

    // Try to parse batching challenge
    size_t size_left = size;
    if (size_left < sizeof(uint256_t)) {
        return {};
    }
    const auto batching_challenge = Fq(*((uint256_t*)data));

    // Try to parse evaluation challenge
    size_left -= sizeof(uint256_t);
    if (size_left < sizeof(uint256_t)) {
        return {};
    }
    const auto x = Fq(*((uint256_t*)data));
    if (x.is_zero()) {
        return {};
    }
    size_left -= sizeof(uint256_t);

    // Try to parse operations
    raw_ops = parse_operations(data + (size - size_left), size_left);
    if (raw_ops.empty()) {
        return {};
    }

    // Add a padding element to avoid non-zero commitments
    const auto p_x = uint256_t(0xd3c208c16d87cfd3, 0xd97816a916871ca8, 0x9b85045b68181585, 0x30644e72e131a02);
    const auto p_y = uint256_t(0x3ce1cc9c7e645a83, 0x2edac647851e3ac5, 0xd0cbe61fced2bc53, 0x1a76dae6d3272396);
    auto padding_element = G1(p_x, p_y);
    auto padding_scalar = -Fr::one();
    auto ecc_op_queue = std::make_shared<ECCOpQueue>();
    ecc_op_queue->set_raw_ops_for_fuzzing(raw_ops);
    ecc_op_queue->mul_accumulate(padding_element, padding_scalar);

    // Return the batching challenge, evaluation challenge and the constructed queue
    return std::make_tuple(batching_challenge, x, ecc_op_queue);
}
