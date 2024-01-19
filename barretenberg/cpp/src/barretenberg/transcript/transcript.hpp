#pragma once

#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/blake3s/blake3s.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"

// #define LOG_CHALLENGES
// #define LOG_INTERACTIONS

namespace bb::honk {

template <typename T, typename... U>
concept Loggable = (std::same_as<T, bb::fr> || std::same_as<T, grumpkin::fr> ||
                    std::same_as<T, bb::g1::affine_element> || std::same_as<T, grumpkin::g1::affine_element> ||
                    std::same_as<T, uint32_t>);

// class TranscriptManifest;
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
    void add_entry(size_t round, const std::string& element_label, size_t element_size)
    {
        manifest[round].entries.emplace_back(element_label, element_size);
    }

    [[nodiscard]] size_t size() const { return manifest.size(); }

    RoundData operator[](const size_t& round) { return manifest[round]; };

    bool operator==(const TranscriptManifest& other) const = default;
};

/**
 * @brief Common transcript class for both parties. Stores the data for the current round, as well as the
 * manifest.
 */
class BaseTranscript {
  public:
    using Proof = std::vector<uint8_t>;

    BaseTranscript() = default;

    /**
     * @brief Construct a new Base Transcript object for Verifier using proof_data
     *
     * @param proof_data
     */
    explicit BaseTranscript(const Proof& proof_data)
        : proof_data(proof_data.begin(), proof_data.end())
    {}
    static constexpr size_t HASH_OUTPUT_SIZE = 32;

    std::ptrdiff_t proof_start = 0;
    size_t num_bytes_written = 0; // the number of bytes written to proof_data by the prover or the verifier
    size_t num_bytes_read = 0;    // the number of bytes read from proof_data by the verifier
    size_t round_number = 0;      // current round for manifest

  private:
    static constexpr size_t MIN_BYTES_PER_CHALLENGE = 128 / 8; // 128 bit challenges
    bool is_first_challenge = true; // indicates if this is the first challenge this transcript is generating
    std::array<uint8_t, HASH_OUTPUT_SIZE> previous_challenge_buffer{}; // default-initialized to zeros
    std::vector<uint8_t> current_round_data;

    // "Manifest" object that records a summary of the transcript interactions
    TranscriptManifest manifest;

    /**
     * @brief Compute next challenge c_next = H( Compress(c_prev || round_buffer) )
     * @details This function computes a new challenge for the current round using the previous challenge
     * and the current round data, if they are exist. It clears the current_round_data if nonempty after
     * computing the challenge to minimize how much we compress. It also sets previous_challenge_buffer
     * to the current challenge buffer to set up next function call.
     * @return std::array<uint8_t, HASH_OUTPUT_SIZE>
     */
    [[nodiscard]] std::array<uint8_t, HASH_OUTPUT_SIZE> get_next_challenge_buffer()
    {
        // Prevent challenge generation if this is the first challenge we're generating,
        // AND nothing was sent by the prover.
        if (is_first_challenge) {
            ASSERT(!current_round_data.empty());
        }

        // concatenate the previous challenge (if this is not the first challenge) with the current round data.
        // TODO(Adrian): Do we want to use a domain separator as the initial challenge buffer?
        // We could be cheeky and use the hash of the manifest as domain separator, which would prevent us from having
        // to domain separate all the data. (See https://safe-hash.dev)
        std::vector<uint8_t> full_buffer;
        if (!is_first_challenge) {
            // if not the first challenge, we can use the previous_challenge_buffer
            full_buffer.insert(full_buffer.end(), previous_challenge_buffer.begin(), previous_challenge_buffer.end());
        } else {
            // Update is_first_challenge for the future
            is_first_challenge = false;
        }
        if (!current_round_data.empty()) {
            full_buffer.insert(full_buffer.end(), current_round_data.begin(), current_round_data.end());
            current_round_data.clear(); // clear the round data buffer since it has been used
        }

        // Pre-hash the full buffer to minimize the amount of data passed to the cryptographic hash function.
        // Only a collision-resistant hash-function like Pedersen is required for this step.
        // Note: this pre-hashing is an efficiency trick that may be discareded if using a SNARK-friendly or in contexts
        // (eg smart contract verification) where the cost of elliptic curve operations is high.
        std::vector<uint8_t> compressed_buffer = to_buffer(crypto::pedersen_hash::hash_buffer(full_buffer));

        // Use a strong hash function to derive the new challenge_buffer.
        auto base_hash = blake3::blake3s(compressed_buffer);

        std::array<uint8_t, HASH_OUTPUT_SIZE> new_challenge_buffer;
        std::copy_n(base_hash.begin(), HASH_OUTPUT_SIZE, new_challenge_buffer.begin());
        // update previous challenge buffer for next time we call this function
        previous_challenge_buffer = new_challenge_buffer;
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

        num_bytes_written += element_bytes.size();
    }

    /**
     * @brief Serializes object and appends it to proof_data
     * @details Calls to_buffer on element to serialize, and modifies proof_data object by appending the serialized
     * bytes to it.
     * @tparam T
     * @param element
     * @param proof_data
     */
    template <typename T> void serialize_to_buffer(const T& element, Proof& proof_data)
    {
        auto element_bytes = to_buffer(element);
        proof_data.insert(proof_data.end(), element_bytes.begin(), element_bytes.end());
    }
    /**
     * @brief Deserializes the bytes starting at offset into the typed element and returns that element.
     * @details Using the template parameter and the offset argument, this function deserializes the bytes with
     * from_buffer and then increments the offset appropriately based on the number of bytes that were deserialized.
     * @tparam T
     * @param proof_data
     * @param offset
     * @return T
     */
    template <typename T> T deserialize_from_buffer(const Proof& proof_data, size_t& offset) const
    {
        constexpr size_t element_size = sizeof(T);
        ASSERT(offset + element_size <= proof_data.size());

        auto element_bytes = std::span{ proof_data }.subspan(offset, element_size);
        offset += element_size;

        T element = from_buffer<T>(element_bytes);

        return element;
    }

  public:
    // Contains the raw data sent by the prover.
    Proof proof_data;

    /**
     * @brief Return the proof data starting at proof_start
     * @details This is useful for when two different provers share a transcript.
     */
    std::vector<uint8_t> export_proof()
    {
        std::vector<uint8_t> result(num_bytes_written);
        std::copy_n(proof_data.begin() + proof_start, num_bytes_written, result.begin());
        proof_start += static_cast<std::ptrdiff_t>(num_bytes_written);
        num_bytes_written = 0;
        return result;
    };

    void load_proof(const std::vector<uint8_t>& proof)
    {
        std::copy(proof.begin(), proof.end(), std::back_inserter(proof_data));
    }

    /**
     * @brief After all the prover messages have been sent, finalize the round by hashing all the data and then create
     * the number of requested challenges.
     * @details Challenges are generated by iteratively hashing over the previous challenge, using
     * get_next_challenge_buffer().
     * TODO(#741): Optimizations for this function include generalizing type of hash, splitting hashes into
     * multiple challenges.
     *
     * @param labels human-readable names for the challenges for the manifest
     * @return std::array<uint256_t, num_challenges> challenges for this round.
     */
    template <typename... Strings> std::array<uint256_t, sizeof...(Strings)> get_challenges(const Strings&... labels)
    {
        constexpr size_t num_challenges = sizeof...(Strings);

        // Add challenge labels for current round to the manifest
        manifest.add_challenge(round_number, labels...);

        // Compute the new challenge buffer from which we derive the challenges.

        // Create challenges from bytes.
        std::array<uint256_t, num_challenges> challenges{};

        // Generate the challenges by iteratively hashing over the previous challenge.
        for (size_t i = 0; i < num_challenges; i++) {
            auto next_challenge_buffer = get_next_challenge_buffer(); // get next challenge buffer
            std::array<uint8_t, sizeof(uint256_t)> field_element_buffer{};
            // copy half of the hash to lower 128 bits of challenge
            // Note: because of how read() from buffers to fields works (in field_declarations.hpp),
            // we use the later half of the buffer
            std::copy_n(next_challenge_buffer.begin(),
                        HASH_OUTPUT_SIZE / 2,
                        field_element_buffer.begin() + HASH_OUTPUT_SIZE / 2);
            challenges[i] = from_buffer<uint256_t>(field_element_buffer);
        }

        // Prepare for next round.
        ++round_number;

        return challenges;
    }

    /**
     * @brief Adds a prover message to the transcript, only intended to be used by the prover.
     *
     * @details Serializes the provided object into `proof_data`, and updates the current round state in
     * consume_prover_element_bytes.
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

#ifdef LOG_INTERACTIONS
        if constexpr (Loggable<T>) {
            info("sent:     ", label, ": ", element);
        }
#endif
        BaseTranscript::consume_prover_element_bytes(label, element_bytes);
    }

    /**
     * @brief Reads the next element of type `T` from the transcript, with a predefined label, only used by verifier.
     *
     * @param label Human readable name for the challenge.
     * @return deserialized element of type T
     */
    template <class T> T receive_from_prover(const std::string& label)
    {
        constexpr size_t element_size = sizeof(T);
        ASSERT(num_bytes_read + element_size <= proof_data.size());

        auto element_bytes = std::span{ proof_data }.subspan(num_bytes_read, element_size);
        num_bytes_read += element_size;

        BaseTranscript::consume_prover_element_bytes(label, element_bytes);

        T element = from_buffer<T>(element_bytes);

#ifdef LOG_INTERACTIONS
        if constexpr (Loggable<T>) {
            info("received: ", label, ": ", element);
        }
#endif
        return element;
    }

    /**
     * @brief For testing: initializes transcript with some arbitrary data so that a challenge can be generated after
     * initialization. Only intended to be used by Prover.
     *
     * @return BaseTranscript
     */
    static std::shared_ptr<BaseTranscript> prover_init_empty()
    {
        auto transcript = std::make_shared<BaseTranscript>();
        constexpr uint32_t init{ 42 }; // arbitrary
        transcript->send_to_verifier("Init", init);
        return transcript;
    };

    /**
     * @brief For testing: initializes transcript based on proof data then receives junk data produced by
     * BaseTranscript::prover_init_empty(). Only intended to be used by Verifier.
     *
     * @param transcript
     * @return BaseTranscript
     */
    static std::shared_ptr<BaseTranscript> verifier_init_empty(const std::shared_ptr<BaseTranscript>& transcript)
    {
        auto verifier_transcript = std::make_shared<BaseTranscript>(transcript->proof_data);
        [[maybe_unused]] auto _ = verifier_transcript->template receive_from_prover<uint32_t>("Init");
        return verifier_transcript;
    };

    uint256_t get_challenge(const std::string& label)
    {
        uint256_t result = get_challenges(label)[0];
#if defined LOG_CHALLENGES || defined LOG_INTERACTIONS
        info("challenge: ", label, ": ", result);
#endif
        return result;
    }

    [[nodiscard]] TranscriptManifest get_manifest() const { return manifest; };

    void print() { manifest.print(); }
};

/**
 * @brief Convert an array of uint256_t's to an array of field elements
 * @details The syntax `std::array<FF, 2> [a, b] = transcript.get_challenges("a", "b")` is unfortunately not allowed
 * (structured bindings must be defined with auto return type), so we need a workaround.
 */
template <typename FF, typename T, size_t N> std::array<FF, N> challenges_to_field_elements(std::array<T, N>&& arr)
{
    std::array<FF, N> result;
    std::move(arr.begin(), arr.end(), result.begin());
    return result;
}
} // namespace bb::honk
