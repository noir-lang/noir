#pragma once

#include <plonk/transcript/transcript.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/bn254/fq.hpp>
#include <ecc/curves/bn254/g1.hpp>

#include "../../primitives/witness/witness.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"
#include "../../hash/blake2s/blake2s.hpp"

namespace plonk {
namespace stdlib {
namespace recursion {
template <typename Composer> class Transcript {
  public:
    using field_pt = field_t<Composer>;
    using witness_pt = witness_t<Composer>;
    using fq_pt = bigfield<Composer, barretenberg::Bn254FqParams>;
    using group_pt = element<Composer, fq_pt, field_pt, barretenberg::Bn254G1Params>;

    Transcript(Composer* in_context, const transcript::Manifest input_manifest)
        : context(in_context)
        , transcript_base(input_manifest, transcript::HashType::Blake2s, 16)
        , current_challenge(in_context)
    {}

    Transcript(Composer* in_context,
               const std::vector<uint8_t>& input_transcript,
               const transcript::Manifest input_manifest)
        : context(in_context)
        , transcript_base(input_transcript, input_manifest, transcript::HashType::Blake2s, 16)
        , current_challenge(in_context)
    /*, transcript_bytes(in_context) */
    {
        // for (size_t i = 0; i < input_transcript.size(); ++i)
        // {
        //     field_t<Composer> data(witness_pt<Composer>(context, input_transcript[i]));
        //     transcript_bytes.write(byte_array<Composer>(data));
        // }
    }

    transcript::Manifest get_manifest() const { return transcript_base.get_manifest(); }

    int check_field_element_cache(const std::string& element_name)
    {
        for (size_t i = 0; i < field_keys.size(); ++i) {
            if (field_keys[i] == element_name) {
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    int check_field_element_vector_cache(const std::string& element_name)
    {
        for (size_t i = 0; i < field_vector_keys.size(); ++i) {
            if (field_vector_keys[i] == element_name) {
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    int check_group_element_cache(const std::string& element_name)
    {
        for (size_t i = 0; i < group_keys.size(); ++i) {
            if (group_keys[i] == element_name) {
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    int check_challenge_cache(const std::string& challenge_name, const size_t challenge_idx)
    {
        for (size_t i = 0; i < challenge_keys.size(); ++i) {
            if (challenge_keys[i] == std::make_pair(challenge_name, challenge_idx)) {
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    void add_field_element(const std::string& element_name, const field_pt& element)
    {
        transcript_base.add_element(element_name, element.get_value().to_buffer());
        field_keys.push_back(element_name);
        field_values.push_back(element);
    }

    void add_group_element(const std::string& element_name, const group_pt& element)
    {
        uint256_t x = element.x.get_value().lo;
        uint256_t y = element.y.get_value().lo;
        barretenberg::g1::affine_element converted{ barretenberg::fq(x), barretenberg::fq(y) };
        transcript_base.add_element(element_name, converted.to_buffer());
        group_keys.push_back(element_name);
        group_values.push_back(element);
    }

    void add_field_element_vector(const std::string& element_name, const std::vector<field_pt>& elements)
    {
        std::vector<barretenberg::fr> values;
        for (size_t i = 0; i < elements.size(); ++i) {
            values.push_back(elements[i].get_value());
        }
        transcript_base.add_element(element_name, barretenberg::fr::to_buffer(values));
        field_vector_keys.push_back(element_name);
        field_vector_values.push_back(elements);
    }

    void apply_fiat_shamir(const std::string& challenge_name)
    {
        transcript_base.apply_fiat_shamir(challenge_name);
        byte_array<Composer> buffer(context);
        if (current_round > 0) {
            buffer.write(current_challenge);
        }
        for (auto manifest_element : get_manifest().get_round_manifest(current_round).elements) {
            if (manifest_element.num_bytes == 32) {
                buffer.write(byte_array<Composer>(get_field_element(manifest_element.name)));
            } else if (manifest_element.num_bytes == 64) {
                buffer.write(byte_array<Composer>(get_group_element(manifest_element.name).to_byte_array()));
            } else if (manifest_element.num_bytes < 32) {
                buffer.write(
                    byte_array<Composer>(get_field_element(manifest_element.name), manifest_element.num_bytes));
            } else {
                std::vector<field_pt> field_array = get_field_element_vector(manifest_element.name);
                for (size_t i = 0; i < field_array.size(); ++i) {
                    buffer.write(byte_array<Composer>(field_array[i]));
                }
            }
        }

        std::vector<byte_array<Composer>> round_challenges;

        byte_array<Composer> base_hash = blake2s(buffer);

        const size_t num_challenges = get_manifest().get_round_manifest(current_round).num_challenges;

        byte_array<Composer> first(field_pt(0), 16);
        first.write(base_hash.slice(0, 16));
        round_challenges.push_back(first);

        if (num_challenges > 1) {
            byte_array<Composer> second(field_pt(0), 16);
            second.write(base_hash.slice(16, 16));
        }

        for (size_t i = 2; i < num_challenges; i += 2) {
            byte_array<Composer> rolling_buffer = base_hash;
            rolling_buffer.write(byte_array<Composer>(field_pt(i / 2), 1));
            byte_array<Composer> hash_output = blake2s(rolling_buffer);

            byte_array<Composer> hi(field_pt(0), 16);
            hi.write(hash_output.slice(0, 16));
            round_challenges.push_back(hi);

            if (i + 1 < num_challenges) {
                byte_array<Composer> lo(field_pt(0), 16);
                lo.write(hash_output.slice(16, 16));
                round_challenges.push_back(lo);
            }
        }

        current_challenge = round_challenges[round_challenges.size() - 1];
        ++current_round;

        challenge_keys.push_back(std::make_pair(challenge_name, num_challenges));

        std::vector<field_pt> challenge_elements;
        for (const auto challenge : round_challenges) {
            challenge_elements.push_back(static_cast<field_pt>(challenge));
        }
        challenge_values.push_back(challenge_elements);
    }

    field_pt get_field_element(const std::string& element_name)
    {
        const int cache_idx = check_field_element_cache(element_name);
        if (cache_idx != -1) {
            return field_values[static_cast<size_t>(cache_idx)];
        }
        barretenberg::fr value =
            barretenberg::fr::serialize_from_buffer(&(transcript_base.get_element(element_name))[0]);
        field_pt result(witness_pt(context, value));
        field_keys.push_back(element_name);
        field_values.push_back(result);
        return result;
    }

    std::vector<field_pt> get_field_element_vector(const std::string& element_name)
    {
        const int cache_idx = check_field_element_vector_cache(element_name);
        if (cache_idx != -1) {
            return field_vector_values[static_cast<size_t>(cache_idx)];
        }
        std::vector<barretenberg::fr> values = barretenberg::fr::from_buffer(transcript_base.get_element(element_name));
        std::vector<field_pt> result;

        for (size_t i = 0; i < values.size(); ++i) {
            result.push_back(witness_pt(context, values[i]));
        }

        field_vector_keys.push_back(element_name);
        field_vector_values.push_back(result);
        return result;
    }

    field_pt get_challenge_field_element(const std::string& challenge_name, const size_t challenge_idx)
    {
        const int cache_idx = check_challenge_cache(challenge_name, challenge_idx);
        ASSERT(cache_idx != -1);
        return challenge_values[static_cast<size_t>(cache_idx)][challenge_idx];
    }

    group_pt get_group_element(const std::string& element_name)
    {
        int cache_idx = check_group_element_cache(element_name);
        if (cache_idx != -1) {
            return group_values[static_cast<size_t>(cache_idx)];
        }
        barretenberg::g1::affine_element value =
            barretenberg::g1::affine_element::serialize_from_buffer(&(transcript_base.get_element(element_name))[0]);
        group_pt result = convert_g1(context, value);
        group_keys.push_back(element_name);
        group_values.push_back(result);
        return result;
    }

    static fq_pt convert_fq(Composer* ctx, const barretenberg::fq& input)
    {
        uint256_t input_u256(input);
        field_pt low(witness_pt(ctx, input_u256.slice(0, 128)));
        field_pt hi(witness_pt(ctx, input_u256.slice(128, 256)));
        return fq_pt(low, hi);
    };

    static group_pt convert_g1(Composer* ctx, const barretenberg::g1::affine_element& input)
    {
        fq_pt x = convert_fq(ctx, input.x);
        fq_pt y = convert_fq(ctx, input.y);
        return group_pt(x, y);
    };

    Composer* context;

  private:
    transcript::Transcript transcript_base;
    byte_array<Composer> current_challenge;

    std::vector<std::string> field_vector_keys;
    std::vector<std::vector<field_pt>> field_vector_values;

    std::vector<std::pair<std::string, size_t>> misc_keys;
    std::vector<field_pt> misc_values;

    std::vector<std::string> field_keys;
    std::vector<field_pt> field_values;

    std::vector<std::string> group_keys;
    std::vector<group_pt> group_values;

    std::vector<std::pair<std::string, size_t>> challenge_keys;
    std::vector<std::vector<field_pt>> challenge_values;

    size_t current_round = 0;
};
} // namespace recursion
} // namespace stdlib
} // namespace plonk