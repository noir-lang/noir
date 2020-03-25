#pragma once

#include <transcript/transcript.hpp>

#include "../primitives/bigfield/bigfield.hpp"
#include "../primitives/biggroup/biggroup.hpp"
#include "../primitives/bool/bool.hpp"
#include "../primitives/field/field.hpp"

namespace stdlib {
namespace recursion {
template <typename Composer> class Transcript {
    using bool_t = bool_t<Composer>;
    using field_t = field_t<Composer>;
    using witness_t = witness_t<Composer>;
    using fq_t = bigfield<Composer, barretenberg::Bn254FqParams>;
    using group_t = element<Composer, fq_t, field_t>;

    Transcript(Composer* in_context, const std::vector<uint8_t>& input_transcript, const Manifest input_manifest)
        : context(in_context)
        , transcript_base(input_transcript, input_manifest, transcript::HashType::Blake2s, 16)
    {}

    Manifest get_manifest() const { return transcript_base.get_manifest(); }

    int check_field_element_cache(const std::string& element_name)
    {
        for (int i = 0; i < (int)field_keys.size(); ++i) {
            if (field_keys[i] == element_name) {
                return i;
            }
        }
        return -1;
    }

    int check_challenge_cache(const std::string& challenge_name, const size_t challenge_idx)
    {
        for (int i = 0; i < (int)challenge_keys.size(); ++i) {
            if (challenge_keys[i] == std::make_pair(challenge_name, challenge_idx)) {
                return i;
            }
        }
        return -1;
    }

    void add_field_element(const std::string& element_name, const field_t<Composer>& element)
    {
        add_element(element_name, element.get_value().to_buffer());
        field_keys.push_back(element_name);
        field_values.push_back(element);
    }

    void apply_fiat_shamir(const std::string& challenge_name)
    {
        std::vector<field_t> buffer;
        if (current_round > 0) {
            buffer.push_back(current_challenge);
        }
        for (auto manifest_element : manifest.get_round_manifest(current_round).elements) {
            if (manifest_element.num_bytes == 32) {
                buffer.push_back(get_field_element(manifest_element.name));
            } else if (manifest_element.num_bytes == 64) {
            }
            ASSERT(elements.count(manifest_element.name) == 1);
            std::vector<uint8_t>& element_data = elements.at(manifest_element.name);
            ASSERT(manifest_element.num_bytes == element_data.size());
            buffer.insert(buffer.end(), element_data.begin(), element_data.end());
        }
    }

    field_t<Composer> get_field_element(const std::string& element_name)
    {
        const int cache_idx = check_field_element_cache(element_name);
        if (cache_idx != -1) {
            return field_values[cache_idx];
        }
        barretenberg::fr value =
            barretenberg::fr::serialize_from_buffer(&(transcript_base.get_element(element_name))[0]);
        field_t<Composer> result(witness_t<Composer>(context, value));
        field_keys.push_back(element_name);
        field_values.push_back(result);
        return result;
    }

    field_t get_challenge_field_element(const std::string& challenge_name, const size_t challenge_idx)
    {
        const int cache_idx = check_challenge_cache(element_name, challenge_idx);
        if (cache_idx != -1) {
            return challenge_values[cache_idx][challenge_idx];
        }
        barretenberg::fr value =
            barretenberg::fr::serialize_from_buffer(&(transcript_base.get_challenge(element_name, idx))[0]);
        field_t<Composer> result(witness_t<Composer>(context, value));
        field_keys.push_back(std::make_pair(element_name, challenge_idx));
        field_values.push_back(result);
        return result;
    }

    group_t get_group_element(const std::string& element_name)
    {
        int cache_idx = check_group_element_cache(element_name);
        if (cache_idx != -1) {
            return group_values[cache_idx];
        }
        barretenberg::g1::affine_element value =
            barretenberg::g1::affine_element::serialize_from_buffer(&(transcript_base.get_element(element_name))[0]);
        group_t result = convert_g1(value);
        group_keys.push_back(element_name);
        group_values.push_back(result);
        return result;
    }

    Composer* context;

  private:
    fq_t convert_fq = [&](const barretenberg::fq& input) {
        field_t<Composer> low(context);
        field_t<Composer> high(context);
        uint256_t input_u256(input);
        field_t low(witness_t(context, input_u256.slice(0, 128));
        field_t hi(witness_t(context, input_u256.slice(128, 256)));
        return fq(context, low, hi);
    };

    group_t convert_g1 = [&](const barretenberg::g1::affine_element& input) {
        fq_t x = convert_fq(input.x);
        fq_t y = convert_fq(input.y);
        return group_t(context, x, y);
    };
    std::vector<std::string> field_keys;
    std::vector<field_t<Composer>> field_values;

    std::vector<std::string> group_keys;
    std::vector<group_t> group_values;

    std::vector<std::pair<std::string, int>> challenge_keys;
    std::vector < std::vector<field_t<Composer>> challenge_values;
    transcript::Transcript transcript_base;
};
} // namespace recursion
} // namespace stdlib