#pragma once

#include <transcript/transcript.hpp>

namespace stdlib {
namespace recursion {
template <typename Composer> class Transcript {
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

    void add_field_element(const std::string& element_name, const std::vector<uint8_t>& buffer);

    field_t<Composer> get_field_element(const std::string& element_name)
    {
        int cache_idx = check_field_element_cache(element_name);
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

    field_t get_challenge_field_element(const std::string& challenge_name) {}

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
    std::vector<std::string> field_keys;
    std::vector<field_t<Composer>> field_values;

    std::vector<std::string> group_keys;
    std::vector<group_t> group_values;

    std::vector<std::string> challenge_keys;
    std::vector < std::vector<field_t<Composer>> challenge_values;
    transcript::Transcript transcript_base;
};
} // namespace recursion
} // namespace stdlib