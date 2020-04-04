#pragma once
#include "manifest.hpp"
#include <array>
#include <map>
#include <string>
#include <vector>

namespace transcript {

struct Keccak256Hasher {
    static constexpr size_t SECURITY_PARAMETER_SIZE = 32;
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;

    static std::array<uint8_t, PRNG_OUTPUT_SIZE> hash(std::vector<uint8_t> const& buffer);
};

struct Blake2sHasher {
    static constexpr size_t SECURITY_PARAMETER_SIZE = 16;
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;

    static std::array<uint8_t, PRNG_OUTPUT_SIZE> hash(std::vector<uint8_t> const& input);
};

enum HashType { Keccak256, Blake2s };

class Transcript {
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;
    struct challenge {
        std::array<uint8_t, PRNG_OUTPUT_SIZE> data;
    };

  public:
    Transcript(const Manifest input_manifest,
               const HashType hash_type = HashType::Keccak256,
               const size_t challenge_bytes = 32)
        : num_challenge_bytes(challenge_bytes)
        , hasher(hash_type)
        , manifest(input_manifest)
    {
        compute_challenge_map();
    }

    Transcript(const std::vector<uint8_t>& input_transcript,
               const Manifest input_manifest,
               const HashType hash_type = HashType::Keccak256,
               const size_t challenge_bytes = 32);
    Manifest get_manifest() const { return manifest; }

    void add_element(const std::string& element_name, const std::vector<uint8_t>& buffer);

    void apply_fiat_shamir(const std::string& challenge_name);

    std::array<uint8_t, PRNG_OUTPUT_SIZE> get_challenge(const std::string& challenge_name, const size_t idx = 0) const;

    std::array<uint8_t, PRNG_OUTPUT_SIZE> get_challenge_from_map(const std::string& challenge_name,
                                                                 const std::string& challenge_map_name) const;

    size_t get_num_challenges(const std::string& challenge_name) const;

    std::vector<uint8_t> get_element(const std::string& element_name) const;

    std::vector<uint8_t> export_transcript() const;

    void compute_challenge_map();

  private:
    size_t current_round = 0;
    size_t num_challenge_bytes;
    HashType hasher;
    std::map<std::string, std::vector<uint8_t>> elements;

    std::map<std::string, std::vector<challenge>> challenges;

    challenge current_challenge;

    Manifest manifest;
    std::map<std::string, size_t> challenge_map;
};

} // namespace transcript
