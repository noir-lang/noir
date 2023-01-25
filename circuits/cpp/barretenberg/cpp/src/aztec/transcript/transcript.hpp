#pragma once
#include "manifest.hpp"
#include <array>
#include <map>
#include <string>
#include <vector>
#include <exception>

#include "../proof_system/verification_key/verification_key.hpp"

namespace transcript {

struct Keccak256Hasher {
    static constexpr size_t SECURITY_PARAMETER_SIZE = 32;
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;

    static std::array<uint8_t, PRNG_OUTPUT_SIZE> hash(std::vector<uint8_t> const& buffer);
};

struct Blake3sHasher {
    static constexpr size_t SECURITY_PARAMETER_SIZE = 16;
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;

    static std::array<uint8_t, PRNG_OUTPUT_SIZE> hash(std::vector<uint8_t> const& input);
};

enum HashType { Keccak256, PedersenBlake3s, PlookupPedersenBlake3s };

/**
 * Transcript is used by the Prover to store round values
 * and derive challenges. The verifier uses it to parse serialized
 * values and get the data and challenges back.
 *
 */
class Transcript {
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;
    struct challenge {
        std::array<uint8_t, PRNG_OUTPUT_SIZE> data;
    };

  public:
    typedef waffle::verification_key Key;

    /**
     * Create a new transcript for Prover based on the manifest.
     *
     * @param input_manifes The manifest with round descriptions.
     * @param hash_type The hash to use for Fiat-Shamir.
     * @param challenge_bytes The number of bytes per challenge to generate.
     *
     */
    Transcript(const Manifest input_manifest,
               const HashType hash_type = HashType::Keccak256,
               const size_t challenge_bytes = 32)
        : num_challenge_bytes(challenge_bytes)
        , hasher(hash_type)
        , manifest(input_manifest)
    {
        // Just to be safe, because compilers can be weird.
        current_challenge.data = {};
        compute_challenge_map();
    }

    /**
     * Parse a serialized version of an input_transcript into a deserialized
     * one based on the manifest.
     *
     * @param input_transcript Serialized transcript.
     * @param input_manifest The manifest which governs the parsing.
     * @param hash_type The hash used for Fiat-Shamir
     * @param challenge_bytes The number of bytes per challenge to generate.
     *
     */
    Transcript(const std::vector<uint8_t>& input_transcript,
               const Manifest input_manifest,
               const HashType hash_type = HashType::Keccak256,
               const size_t challenge_bytes = 32);

    Manifest get_manifest() const { return manifest; }

    void add_element(const std::string& element_name, const std::vector<uint8_t>& buffer);

    void apply_fiat_shamir(const std::string& challenge_name /*, const bool debug = false*/);

    bool has_challenge(const std::string& challenge_name) const;

    std::array<uint8_t, PRNG_OUTPUT_SIZE> get_challenge(const std::string& challenge_name, const size_t idx = 0) const;

    int get_challenge_index_from_map(const std::string& challenge_map_name) const;

    std::array<uint8_t, PRNG_OUTPUT_SIZE> get_challenge_from_map(const std::string& challenge_name,
                                                                 const std::string& challenge_map_name) const;

    size_t get_num_challenges(const std::string& challenge_name) const;

    std::vector<uint8_t> get_element(const std::string& element_name) const;

    size_t get_element_size(const std::string& element_name) const;

    std::vector<uint8_t> export_transcript() const;

    void compute_challenge_map();

    void mock_inputs_prior_to_challenge(const std::string& challenge_name, size_t circuit_size = 1);

    void print();

  private:
    // The round of the protocol
    size_t current_round = 0;
    size_t num_challenge_bytes;
    HashType hasher;
    std::map<std::string, std::vector<uint8_t>> elements;

    std::map<std::string, std::vector<challenge>> challenges;

    challenge current_challenge;

    Manifest manifest;
    std::map<std::string, int> challenge_map;
};

} // namespace transcript
