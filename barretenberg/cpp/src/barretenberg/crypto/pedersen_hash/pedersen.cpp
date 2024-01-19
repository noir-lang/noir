#include "./pedersen.hpp"
#include "../pedersen_commitment/pedersen.hpp"

namespace bb::crypto {

/**
 * @brief Converts input uint8_t buffers into vector of field elements. Used to hash the Transcript in a
 * SNARK-friendly manner for recursive circuits.
 *
 * `buffer` is an unstructured byte array we want to convert these into field elements
 * prior to hashing. We do this by splitting buffer into 31-byte chunks.
 *
 * @param buffer
 * @return std::vector<Fq>
 */
template <typename Curve>
std::vector<typename Curve::BaseField> pedersen_hash_base<Curve>::convert_buffer(const std::vector<uint8_t>& input)
{
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31;
    size_t num_elements = static_cast<size_t>(num_bytes % bytes_per_element != 0) + (num_bytes / bytes_per_element);

    const auto slice = [](const std::vector<uint8_t>& data, const size_t start, const size_t slice_size) {
        uint256_t result(0);
        for (size_t i = 0; i < slice_size; ++i) {
            result = (result << uint256_t(8));
            result += uint256_t(data[i + start]);
        }
        return Fq(result);
    };

    std::vector<Fq> elements;
    for (size_t i = 0; i < num_elements - 1; ++i) {
        size_t bytes_to_slice = bytes_per_element;
        Fq element = slice(input, i * bytes_per_element, bytes_to_slice);
        elements.emplace_back(element);
    }
    size_t bytes_to_slice = num_bytes - ((num_elements - 1) * bytes_per_element);
    Fq element = slice(input, (num_elements - 1) * bytes_per_element, bytes_to_slice);
    elements.emplace_back(element);
    return elements;
}

/**
 * @brief Given a vector of fields, generate a pedersen hash using generators from `context`.
 *
 * @details `context.offset` is used to access offset elements of `context.generators` if required.
 *          e.g. if one desires to compute
 *          `inputs[0] * [generators[hash_index]] + `inputs[1] * [generators[hash_index + 1]]` + ... etc
 *          Potentially useful to ensure multiple hashes with the same domain separator cannot collide.
 *
 * @param inputs what are we hashing?
 * @param context Stores generator metadata + context pointer to the generators we are using for this hash
 * @return Fq (i.e. SNARK circuit scalar field, when hashing using a curve defined over the SNARK circuit scalar field)
 */
template <typename Curve>
typename Curve::BaseField pedersen_hash_base<Curve>::hash(const std::vector<Fq>& inputs, const GeneratorContext context)
{
    Element result = length_generator * Fr(inputs.size());
    return (result + pedersen_commitment_base<Curve>::commit_native(inputs, context)).normalize().x;
}

/**
 * @brief Given an arbitrary length of bytes, convert them to fields and hash the result using the default generators.
 */
template <typename Curve>
typename Curve::BaseField pedersen_hash_base<Curve>::hash_buffer(const std::vector<uint8_t>& input,
                                                                 const GeneratorContext context)
{
    std::vector<Fq> converted = convert_buffer(input);

    if (converted.size() < 2) {
        return hash(converted, context);
    }
    auto result = hash({ converted[0], converted[1] }, context);
    for (size_t i = 2; i < converted.size(); ++i) {
        result = hash({ result, converted[i] }, context);
    }
    return result;
}

template class pedersen_hash_base<curve::Grumpkin>;
} // namespace bb::crypto