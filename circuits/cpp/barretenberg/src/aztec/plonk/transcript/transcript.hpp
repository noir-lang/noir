#pragma once
#include "manifest.hpp"
#include <array>
#include <map>
#include <string>
#include <vector>

namespace transcript {
class Transcript {
    static constexpr size_t PRNG_OUTPUT_SIZE = 32;

    struct challenge {
        std::array<uint8_t, PRNG_OUTPUT_SIZE> data;
    };

  public:
    Transcript(const Manifest input_manifest)
        : manifest(input_manifest){};

    Transcript(const std::vector<uint8_t>& input_transcript, const Manifest input_manifest);
    Manifest get_manifest() const { return manifest; }

    void add_element(const std::string& element_name, const std::vector<uint8_t>& buffer);

    void apply_fiat_shamir(const std::string& challenge_name);

    std::array<uint8_t, PRNG_OUTPUT_SIZE> get_challenge(const std::string& challenge_name, const size_t idx = 0) const;

    std::vector<uint8_t> get_element(const std::string& element_name) const;

    std::vector<uint8_t> export_transcript() const;

  private:
    size_t current_round = 0;
    std::map<std::string, std::vector<uint8_t>> elements;

    std::map<std::string, std::vector<challenge>> challenges;

    challenge current_challenge;

    Manifest manifest;
};
} // namespace transcript
