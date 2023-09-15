#pragma once

#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/blake3s/blake3s.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"

#include <algorithm>
#include <array>
#include <concepts>
#include <cstddef>
#include <cstdint>
#include <map>
#include <span>
#include <string>
#include <utility>
#include <vector>

namespace proof_system::honk {

class TranscriptManifest {
    struct RoundData {
        std::vector<std::string> challenge_label;
        std::vector<std::pair<std::string, size_t>> entries;

        bool operator==(const RoundData& other) const = default;
    };

    std::map<size_t, RoundData> manifest;

  public:
    void print()
    {
        for (auto& round : manifest) {
            info("Round: ", round.first);
            for (auto& label : round.second.challenge_label) {
                info("\tchallenge: ", label);
            }
            for (auto& entry : round.second.entries) {
                info("\telement (", entry.second, "): ", entry.first);
            }
        }
    }

    template <typename... Strings> void add_challenge(size_t round, Strings&... labels)
    {
        manifest[round].challenge_label = { labels... };
    }
    void add_entry(size_t round, std::string element_label, size_t element_size)
    {
        manifest[round].entries.emplace_back(element_label, element_size);
    }

    [[nodiscard]] size_t size() const { return manifest.size(); }

    RoundData operator[](const size_t& round) { return manifest[round]; };

    bool operator==(const TranscriptManifest& other) const = default;
};

/**
 * @brief Common transcript functionality for both parties. Stores the data for the current round, as well as the
 * manifest.
 *
 * @tparam FF Field from which we sample challenges.
 */
template <typename FF> class BaseTranscript {
    // TODO(Adrian): Make these tweakable
  public:
    static constexpr size_t HASH_OUTPUT_SIZE = 32;

  private:
    static constexpr size_t MIN_BYTES_PER_CHALLENGE = 128 / 8; // 128 bit challenges

    size_t round_number = 0;
    std::array<uint8_t, HASH_OUTPUT_SIZE> previous_challenge_buffer{}; // default-initialized to zeros
    std::vector<uint8_t> current_round_data;

    // "Manifest" object that records a summary of the transcript interactions
    TranscriptManifest manifest;

    /**
     * @brief Compute c_next = H( Compress(c_prev || round_buffer) )
     *
     * @return std::array<uint8_t, HASH_OUTPUT_SIZE>
     */
    [[nodiscard]] std::array<uint8_t, HASH_OUTPUT_SIZE> get_next_challenge_buffer() const
    {
        // Prevent challenge generation if nothing was sent by the prover.
        ASSERT(!current_round_data.empty());

        // concatenate the hash of the previous round (if not the first round) with the current round data.
        // TODO(Adrian): Do we want to use a domain separator as the initial challenge buffer?
        // We could be cheeky and use the hash of the manifest as domain separator, which would prevent us from having
        // to domain separate all the data. (See https://safe-hash.dev)
        std::vector<uint8_t> full_buffer;
        if (round_number > 0) {
            full_buffer.insert(full_buffer.end(), previous_challenge_buffer.begin(), previous_challenge_buffer.end());
        }
        full_buffer.insert(full_buffer.end(), current_round_data.begin(), current_round_data.end());

        // Pre-hash the full buffer to minimize the amount of data passed to the cryptographic hash function.
        // Only a collision-resistant hash-function like Pedersen is required for this step.
        // Note: this pre-hashing is an efficiency trick that may be discareded if using a SNARK-friendly or in contexts
        // (eg smart contract verification) where the cost of elliptic curve operations is high.
        std::vector<uint8_t> compressed_buffer = to_buffer(crypto::pedersen_commitment::compress_native(full_buffer));

        // Use a strong hash function to derive the new challenge_buffer.
        auto base_hash = blake3::blake3s(compressed_buffer);

        std::array<uint8_t, HASH_OUTPUT_SIZE> new_challenge_buffer;
        std::copy_n(base_hash.begin(), HASH_OUTPUT_SIZE, new_challenge_buffer.begin());

        return new_challenge_buffer;
    };

  protected:
    /**
     * @brief Adds challenge elements to the current_round_buffer and updates the manifest.
     *
     * @param label of the element sent
     * @param element_bytes serialized
     */
    void consume_prover_element_bytes(const std::string& label, std::span<const uint8_t> element_bytes)
    {
        // Add an entry to the current round of the manifest
        manifest.add_entry(round_number, label, element_bytes.size());

        current_round_data.insert(current_round_data.end(), element_bytes.begin(), element_bytes.end());
    }

  public:
    /**
     * @brief After all the prover messages have been sent, finalize the round by hashing all the data and then create
     * the number of requested challenges which will be increasing powers of the first challenge.  Finally, reset the
     * state in preparation for the next round.
     *
     * @param labels human-readable names for the challenges for the manifest
     * @return std::array<FF, num_challenges> challenges for this round.
     */
    template <typename... Strings> std::array<FF, sizeof...(Strings)> get_challenges(const Strings&... labels)
    {
        constexpr size_t num_challenges = sizeof...(Strings);

        // Add challenge labels for current round to the manifest
        manifest.add_challenge(round_number, labels...);

        // Compute the new challenge buffer from which we derive the challenges.
        auto next_challenge_buffer = get_next_challenge_buffer();

        // Create challenges from bytes.
        std::array<FF, num_challenges> challenges{};

        std::array<uint8_t, sizeof(FF)> field_element_buffer{};
        std::copy_n(next_challenge_buffer.begin(), HASH_OUTPUT_SIZE, field_element_buffer.begin());

        challenges[0] = from_buffer<FF>(field_element_buffer);

        // TODO(#583): rework the transcript to have a better structure and be able to produce a variable amount of
        // challenges that are not powers of each other
        for (size_t i = 1; i < num_challenges; i++) {
            challenges[i] = challenges[i - 1] * challenges[0];
        }

        // Prepare for next round.
        ++round_number;
        current_round_data.clear();
        previous_challenge_buffer = next_challenge_buffer;

        return challenges;
    }

    FF get_challenge(const std::string& label) { return get_challenges(label)[0]; }

    [[nodiscard]] TranscriptManifest get_manifest() const { return manifest; };

    void print() { manifest.print(); }
};

template <typename FF> class ProverTranscript : public BaseTranscript<FF> {

  public:
    /// Contains the raw data sent by the prover.
    std::vector<uint8_t> proof_data;

    /**
     * @brief Adds a prover message to the transcript.
     *
     * @details Serializes the provided object into `proof_data`, and updates the current round state.
     *
     * @param label Description/name of the object being added.
     * @param element Serializable object that will be added to the transcript
     *
     * @todo Use a concept to only allow certain types to be passed. Requirements are that the object should be
     * serializable.
     *
     */
    template <class T> void send_to_verifier(const std::string& label, const T& element)
    {
        using serialize::write;
        // TODO(Adrian): Ensure that serialization of affine elements (including point at infinity) is consistent.
        // TODO(Adrian): Consider restricting serialization (via concepts) to types T for which sizeof(T) reliably
        // returns the size of T in bytes. (E.g. this is true for std::array but not for std::vector).
        auto element_bytes = to_buffer(element);
        proof_data.insert(proof_data.end(), element_bytes.begin(), element_bytes.end());

        BaseTranscript<FF>::consume_prover_element_bytes(label, element_bytes);
    }

    /**
     * @brief For testing: initializes transcript with some arbitrary data so that a challenge can be generated after
     * initialization
     *
     * @return ProverTranscript
     */
    static ProverTranscript init_empty()
    {
        ProverTranscript<FF> transcript;
        constexpr uint32_t init{ 42 }; // arbitrary
        transcript.send_to_verifier("Init", init);
        return transcript;
    };
};

template <class FF> class VerifierTranscript : public BaseTranscript<FF> {

    /// Contains the raw data sent by the prover.
    std::vector<uint8_t> proof_data_;
    size_t num_bytes_read_ = 0;

  public:
    VerifierTranscript() = default;

    explicit VerifierTranscript(const std::vector<uint8_t>& proof_data)
        : proof_data_(proof_data.begin(), proof_data.end())
    {}

    /**
     * @brief For testing: initializes transcript based on proof data then receives junk data produced by
     * ProverTranscript::init_empty()
     *
     * @param transcript
     * @return VerifierTranscript
     */
    static VerifierTranscript init_empty(const ProverTranscript<FF>& transcript)
    {
        VerifierTranscript<FF> verifier_transcript{ transcript.proof_data };
        [[maybe_unused]] auto _ = verifier_transcript.template receive_from_prover<uint32_t>("Init");
        return verifier_transcript;
    };

    /**
     * @brief Reads the next element of type `T` from the transcript, with a predefined label.
     *
     * @param label Human readable name for the challenge.
     * @return deserialized element of type T
     */
    template <class T> T receive_from_prover(const std::string& label)
    {
        constexpr size_t element_size = sizeof(T);
        ASSERT(num_bytes_read_ + element_size <= proof_data_.size());

        auto element_bytes = std::span{ proof_data_ }.subspan(num_bytes_read_, element_size);
        num_bytes_read_ += element_size;

        BaseTranscript<FF>::consume_prover_element_bytes(label, element_bytes);

        T element = from_buffer<T>(element_bytes);

        return element;
    }
};
} // namespace proof_system::honk
