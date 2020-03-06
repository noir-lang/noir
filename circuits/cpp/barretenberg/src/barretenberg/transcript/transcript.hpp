#pragma once

#include <array>
#include <map>
#include <string>
#include <vector>

#include "./manifest.hpp"

namespace transcript {
class Transcript {
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;

  public:
    Transcript(const Manifest input_manifest)
        : manifest(input_manifest){};

    Transcript(const std::vector<uint8_t>& input_transcript, const Manifest input_manifest);
    Manifest get_manifest() const { return manifest; }

    void add_element(const std::string& element_name, const std::vector<uint8_t>& buffer);

    std::array<uint8_t, PRNG_OUTPUT_SIZE> apply_fiat_shamir(const std::string& challenge_name);

    std::array<uint8_t, PRNG_OUTPUT_SIZE> get_challenge(const std::string& challenge_name) const;

    std::vector<uint8_t> get_element(const std::string& element_name) const;

    std::vector<uint8_t> export_transcript() const;

  private:
    size_t current_round = 0;
    std::map<std::string, std::vector<uint8_t>> elements;

    std::map<std::string, std::array<uint8_t, PRNG_OUTPUT_SIZE>> challenges;

    std::array<uint8_t, PRNG_OUTPUT_SIZE> current_challenge;

    Manifest manifest;
};
} // namespace transcript
