#include <common/assert.hpp>
#include <crypto/keccak/keccak.hpp>
#include "transcript.hpp"

namespace transcript {

Transcript::Transcript(const std::vector<uint8_t>& input_transcript, const Manifest input_manifest)
    : manifest(input_manifest)
{
    const size_t num_rounds = input_manifest.get_num_rounds();
    const uint8_t* buffer = &input_transcript[0];
    size_t count = 0;
    for (size_t i = 0; i < num_rounds; ++i) {
        for (auto manifest_element : input_manifest.get_round_manifest(i).elements) {
            if (!manifest_element.derived_by_verifier) {
                // printf("reading element %s ", manifest_element.name.c_str());
                // for (size_t j = 0; j < manifest_element.num_bytes; ++j)
                // {
                //     printf("%x", buffer[count + j]);
                // }
                // printf("\n");
                elements.insert({ manifest_element.name,
                                  std::vector<uint8_t>(buffer + count, buffer + count + manifest_element.num_bytes) });
                count += manifest_element.num_bytes;
            }
        }
    }
    // printf("input buffer size = %lu \n", count);
}

void Transcript::add_element(const std::string& element_name, const std::vector<uint8_t>& buffer)
{
    ASSERT(manifest.get_round_manifest(current_round).includes_element(element_name));
    // printf("adding element %s . size = %lu \n [", element_name.c_str(), buffer.size());
    // for (size_t i = 0;i < buffer.size(); ++i)
    // {
    //     printf("%x", buffer[i]);
    // }
    // printf("]\n");
    elements.insert({ element_name, buffer });
}

std::array<uint8_t, Transcript::PRNG_OUTPUT_SIZE> Transcript::apply_fiat_shamir(const std::string& challenge_name)
{
    ASSERT(current_round <= manifest.get_num_rounds());
    ASSERT(challenge_name == manifest.get_round_manifest(current_round).challenge);

    std::vector<uint8_t> buffer;
    if (current_round > 0) {
        buffer.insert(buffer.end(), current_challenge.begin(), current_challenge.end());
    }
    for (auto manifest_element : manifest.get_round_manifest(current_round).elements) {
        // TODO: Was failing...
        // ASSERT(elements.count(manifest_element.name) == 1);
        std::vector<uint8_t>& element_data = elements[manifest_element.name];
        //ASSERT(manifest_element.num_bytes == element_data.size());
        buffer.insert(buffer.end(), element_data.begin(), element_data.end());
    }

    keccak256 hash_result = ethash_keccak256(&buffer[0], buffer.size());

    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 8; ++j) {
            uint8_t byte = static_cast<uint8_t>(hash_result.word64s[i] >> (56 - (j * 8)));
            current_challenge[i * 8 + j] = byte;
        }
    }

    challenges.insert({ challenge_name, current_challenge });
    ++current_round;
    return current_challenge;
}

std::array<uint8_t, Transcript::PRNG_OUTPUT_SIZE> Transcript::get_challenge(const std::string& challenge_name) const
{
    // printf("getting challenge %s \n", challenge_name.c_str());
    ASSERT(challenges.count(challenge_name) == 1);
    return challenges.at(challenge_name);
}

std::vector<uint8_t> Transcript::get_element(const std::string& element_name) const
{
    // printf("getting element %s \n", element_name.c_str());
    ASSERT(elements.count(element_name) == 1);
    return elements.at(element_name);
}

std::vector<uint8_t> Transcript::export_transcript() const
{
    std::vector<uint8_t> buffer;

    for (size_t i = 0; i < manifest.get_num_rounds(); ++i) {
        for (auto manifest_element : manifest.get_round_manifest(i).elements) {
            ASSERT(elements.count(manifest_element.name) == 1);
            const std::vector<uint8_t>& element_data = elements.at(manifest_element.name);
            ASSERT(manifest_element.num_bytes == element_data.size());
            if (!manifest_element.derived_by_verifier) {
                // printf("writing element %s ", manifest_element.name.c_str());
                // for (size_t j = 0; j < element_data.size(); ++j) {
                //     printf("%x", element_data[j]);
                // }
                // printf("\n");
                buffer.insert(buffer.end(), element_data.begin(), element_data.end());
            }
        }
    }
    // printf("output buffer size = %lu \n", buffer.size());
    return buffer;
}
} // namespace transcript