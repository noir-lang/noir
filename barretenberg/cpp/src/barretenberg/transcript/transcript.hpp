#pragma once

// #define LOG_CHALLENGES
// #define LOG_INTERACTIONS

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/fields/field_conversion.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include <concepts>

namespace bb {

template <typename T, typename... U>
concept Loggable =
    (std::same_as<T, bb::fr> || std::same_as<T, grumpkin::fr> || std::same_as<T, bb::g1::affine_element> ||
     std::same_as<T, grumpkin::g1::affine_element> || std::same_as<T, uint32_t>);

// class TranscriptManifest;
class TranscriptManifest {
    struct RoundData {
        std::vector<std::string> challenge_label;
        std::vector<std::pair<std::string, size_t>> entries;

        void print()
        {
            for (auto& label : challenge_label) {
                info("\tchallenge: ", label);
            }
            for (auto& entry : entries) {
                info("\telement (", entry.second, "): ", entry.first);
            }
        }

        bool operator==(const RoundData& other) const = default;
    };

    std::map<size_t, RoundData> manifest;

  public:
    void print()
    {
        for (auto& round : manifest) {
            info("Round: ", round.first);
            round.second.print();
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

struct NativeTranscriptParams {
    using Fr = bb::fr;
    using Proof = HonkProof;
    static Fr hash(const std::vector<Fr>& data);
    template <typename T> static inline T convert_challenge(const Fr& challenge)
    {
        return bb::field_conversion::convert_challenge<T>(challenge);
    }
    template <typename T> static constexpr size_t calc_num_bn254_frs()
    {
        return bb::field_conversion::calc_num_bn254_frs<T>();
    }
    template <typename T> static inline T convert_from_bn254_frs(std::span<const Fr> frs)
    {
        return bb::field_conversion::convert_from_bn254_frs<T>(frs);
    }
    template <typename T> static inline std::vector<Fr> convert_to_bn254_frs(const T& element)
    {
        return bb::field_conversion::convert_to_bn254_frs(element);
    }
};

/**
 * @brief Common transcript class for both parties. Stores the data for the current round, as well as the
 * manifest.
 */
template <typename TranscriptParams> class BaseTranscript {
  public:
    using Fr = typename TranscriptParams::Fr;
    using Proof = typename TranscriptParams::Proof;

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
    size_t num_frs_written = 0; // the number of bb::frs written to proof_data by the prover or the verifier
    size_t num_frs_read = 0;    // the number of bb::frs read from proof_data by the verifier
    size_t round_number = 0;    // current round for manifest

  private:
    bool is_first_challenge = true; // indicates if this is the first challenge this transcript is generating
    Fr previous_challenge{};        // default-initialized to zeros
    std::vector<Fr> current_round_data;

    // "Manifest" object that records a summary of the transcript interactions
    TranscriptManifest manifest;

    /**
     * @brief Compute next challenge c_next = H( Compress(c_prev || round_buffer) )
     * @details This function computes a new challenge for the current round using the previous challenge
     * and the current round data, if they are exist. It clears the current_round_data if nonempty after
     * computing the challenge to minimize how much we compress. It also sets previous_challenge
     * to the current challenge buffer to set up next function call.
     * @return std::array<Fr, HASH_OUTPUT_SIZE>
     */
    [[nodiscard]] Fr get_next_challenge_buffer()
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
        std::vector<Fr> full_buffer;
        if (!is_first_challenge) {
            // if not the first challenge, we can use the previous_challenge
            full_buffer.emplace_back(previous_challenge);
        } else {
            // Update is_first_challenge for the future
            is_first_challenge = false;
        }
        if (!current_round_data.empty()) {
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/832): investigate why
            // full_buffer.insert(full_buffer.end(), current_round_data.begin(), current_round_data.end()); fails to
            // compile with gcc
            std::copy(current_round_data.begin(), current_round_data.end(), std::back_inserter(full_buffer));
            current_round_data.clear(); // clear the round data buffer since it has been used
        }

        // Hash the full buffer with poseidon2, which is believed to be a collision resistant hash function and a random
        // oracle, removing the need to pre-hash to compress and then hash with a random oracle, as we previously did
        // with Pedersen and Blake3s.
        Fr new_challenge = TranscriptParams::hash(full_buffer);

        // update previous challenge buffer for next time we call this function
        previous_challenge = new_challenge;
        return new_challenge;
    };

  protected:
    /**
     * @brief Adds challenge elements to the current_round_buffer and updates the manifest.
     *
     * @param label of the element sent
     * @param element_frs serialized
     */
    void consume_prover_element_frs(const std::string& label, std::span<const Fr> element_frs)
    {
        // Add an entry to the current round of the manifest
        manifest.add_entry(round_number, label, element_frs.size());

        current_round_data.insert(current_round_data.end(), element_frs.begin(), element_frs.end());

        num_frs_written += element_frs.size();
    }

    /**
     * @brief Serializes object and appends it to proof_data
     * @details Calls to_buffer on element to serialize, and modifies proof_data object by appending the serialized
     * frs to it.
     * @tparam T
     * @param element
     * @param proof_data
     */
    template <typename T> void serialize_to_buffer(const T& element, Proof& proof_data)
    {
        auto element_frs = TranscriptParams::template convert_to_bn254_frs(element);
        proof_data.insert(proof_data.end(), element_frs.begin(), element_frs.end());
    }
    /**
     * @brief Deserializes the frs starting at offset into the typed element and returns that element.
     * @details Using the template parameter and the offset argument, this function deserializes the frs with
     * from_buffer and then increments the offset appropriately based on the number of frs that were deserialized.
     * @tparam T
     * @param proof_data
     * @param offset
     * @return T
     */
    template <typename T> T deserialize_from_buffer(const Proof& proof_data, size_t& offset) const
    {
        constexpr size_t element_fr_size = TranscriptParams::template calc_num_bn254_frs<T>();
        ASSERT(offset + element_fr_size <= proof_data.size());

        auto element_frs = std::span{ proof_data }.subspan(offset, element_fr_size);
        offset += element_fr_size;

        auto element = TranscriptParams::template convert_from_bn254_frs<T>(element_frs);

        return element;
    }

  public:
    // Contains the raw data sent by the prover.
    Proof proof_data;

    /**
     * @brief Return the proof data starting at proof_start
     * @details This is useful for when two different provers share a transcript.
     */
    std::vector<Fr> export_proof()
    {
        std::vector<Fr> result(num_frs_written);
        std::copy_n(proof_data.begin() + proof_start, num_frs_written, result.begin());
        proof_start += static_cast<std::ptrdiff_t>(num_frs_written);
        num_frs_written = 0;
        return result;
    };

    void load_proof(const std::vector<Fr>& proof)
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
     * @return std::array<Fr, num_challenges> challenges for this round.
     */
    template <typename ChallengeType, typename... Strings>
    std::array<ChallengeType, sizeof...(Strings)> get_challenges(const Strings&... labels)
    {
        constexpr size_t num_challenges = sizeof...(Strings);

        // Add challenge labels for current round to the manifest
        manifest.add_challenge(round_number, labels...);

        // Compute the new challenge buffer from which we derive the challenges.

        // Create challenges from Frs.
        std::array<ChallengeType, num_challenges> challenges{};

        // Generate the challenges by iteratively hashing over the previous challenge.
        for (size_t i = 0; i < num_challenges; i++) {
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/741): Optimize this by truncating hash to 128
            // bits or by splitting hash into 2 challenges.
            /*
            auto next_challenge_buffer = get_next_challenge_buffer(); // get next challenge buffer
            Fr field_element_buffer = next_challenge_buffer;
            // copy half of the hash to lower 128 bits of challenge Note: because of how read() from buffers to fields
            works (in field_declarations.hpp), we use the later half of the buffer
            // std::copy_n(next_challenge_buffer.begin(),
            //             HASH_OUTPUT_SIZE / 2,
            //             field_element_buffer.begin() + HASH_OUTPUT_SIZE / 2);
            */
            auto challenge_buffer = get_next_challenge_buffer();
            challenges[i] = TranscriptParams::template convert_challenge<ChallengeType>(challenge_buffer);
        }

        // Prepare for next round.
        ++round_number;

        return challenges;
    }

    /**
     * @brief Adds a prover message to the transcript, only intended to be used by the prover.
     *
     * @details Serializes the provided object into `proof_data`, and updates the current round state in
     * consume_prover_element_frs.
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
        // TODO(Adrian): Ensure that serialization of affine elements (including point at infinity) is consistent.
        // TODO(Adrian): Consider restricting serialization (via concepts) to types T for which sizeof(T) reliably
        // returns the size of T in frs. (E.g. this is true for std::array but not for std::vector).
        // convert element to field elements
        auto element_frs = TranscriptParams::convert_to_bn254_frs(element);
        proof_data.insert(proof_data.end(), element_frs.begin(), element_frs.end());

#ifdef LOG_INTERACTIONS
        if constexpr (Loggable<T>) {
            info("sent:     ", label, ": ", element);
        }
#endif
        BaseTranscript::consume_prover_element_frs(label, element_frs);
    }

    /**
     * @brief Reads the next element of type `T` from the transcript, with a predefined label, only used by verifier.
     *
     * @param label Human readable name for the challenge.
     * @return deserialized element of type T
     */
    template <class T> T receive_from_prover(const std::string& label)
    {
        const size_t element_size = TranscriptParams::template calc_num_bn254_frs<T>();
        ASSERT(num_frs_read + element_size <= proof_data.size());

        auto element_frs = std::span{ proof_data }.subspan(num_frs_read, element_size);
        num_frs_read += element_size;

        BaseTranscript::consume_prover_element_frs(label, element_frs);

        auto element = TranscriptParams::template convert_from_bn254_frs<T>(element_frs);

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
        [[maybe_unused]] auto _ = verifier_transcript->template receive_from_prover<Fr>("Init");
        return verifier_transcript;
    };

    template <typename ChallengeType> ChallengeType get_challenge(const std::string& label)
    {
        ChallengeType result = get_challenges<ChallengeType>(label)[0];
#if defined LOG_CHALLENGES || defined LOG_INTERACTIONS
        info("challenge: ", label, ": ", result);
#endif
        return result;
    }

    [[nodiscard]] TranscriptManifest get_manifest() const { return manifest; };

    void print() { manifest.print(); }
};

template <typename Builder>
static bb::StdlibProof<Builder> convert_proof_to_witness(Builder* builder, const HonkProof& proof)
{
    bb::StdlibProof<Builder> result;
    for (const auto& element : proof) {
        result.push_back(bb::stdlib::witness_t<Builder>(builder, element));
    }
    return result;
}

using NativeTranscript = BaseTranscript<NativeTranscriptParams>;

///////////////////////////////////////////
// Solidity Transcript
///////////////////////////////////////////

// This is a compatible wrapper around the keccak256 function from ethash
inline bb::fr keccak_hash_uint256(std::vector<bb::fr> const& data)
// Losing 2 bits of this is not an issue -> we can just reduce mod p
{
    // cast into uint256_t
    std::vector<uint8_t> buffer = to_buffer(data);

    keccak256 hash_result = ethash_keccak256(&buffer[0], buffer.size());
    for (auto& word : hash_result.word64s) {
        if (is_little_endian()) {
            word = __builtin_bswap64(word);
        }
    }
    std::array<uint8_t, 32> result;

    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 8; ++j) {
            uint8_t byte = static_cast<uint8_t>(hash_result.word64s[i] >> (56 - (j * 8)));
            result[i * 8 + j] = byte;
        }
    }

    auto result_fr = from_buffer<bb::fr>(result);

    return result_fr;
}

struct KeccakTranscriptParams {
    using Fr = bb::fr;
    using Proof = HonkProof;

    static inline Fr hash(const std::vector<Fr>& data) { return keccak_hash_uint256(data); }

    template <typename T> static inline T convert_challenge(const Fr& challenge)
    {
        return bb::field_conversion::convert_challenge<T>(challenge);
    }
    template <typename T> static constexpr size_t calc_num_bn254_frs()
    {
        return bb::field_conversion::calc_num_bn254_frs<T>();
    }
    template <typename T> static inline T convert_from_bn254_frs(std::span<const Fr> frs)
    {
        return bb::field_conversion::convert_from_bn254_frs<T>(frs);
    }
    template <typename T> static inline std::vector<Fr> convert_to_bn254_frs(const T& element)
    {
        // TODO(md): Need to refactor this to be able to NOT just be field elements - Im working about it in the
        // verifier for keccak resulting in twice as much hashing
        return bb::field_conversion::convert_to_bn254_frs(element);
    }
};

using KeccakTranscript = BaseTranscript<KeccakTranscriptParams>;

} // namespace bb
