#pragma once

#include <ecc/curves/bn254/fq.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/bn254/g1.hpp>
#include <plonk/transcript/transcript.hpp>

#include "../../primitives/curves/bn254.hpp"
#include "../verification_key/verification_key.hpp"
#include "../../hash/blake2s/blake2s.hpp"
#include "../../hash/pedersen/pedersen.hpp"
#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/witness/witness.hpp"

namespace plonk {
namespace stdlib {
namespace recursion {
template <typename Composer> class Transcript {
  public:
    using field_pt = field_t<Composer>;
    using witness_pt = witness_t<Composer>;
    using fq_pt = bigfield<Composer, barretenberg::Bn254FqParams>;
    using group_pt = element<Composer, fq_pt, field_pt, barretenberg::g1>;
    using pedersen = plonk::stdlib::pedersen<Composer>;
    using Key = plonk::stdlib::recursion::verification_key<stdlib::bn254<Composer>>;

    Transcript(Composer* in_context, const transcript::Manifest input_manifest)
        : context(in_context)
        , transcript_base(input_manifest, transcript::HashType::PedersenBlake2s, 16)
        , current_challenge(in_context)
    {}

    Transcript(Composer* in_context,
               const std::vector<uint8_t>& input_transcript,
               const transcript::Manifest input_manifest)
        : context(in_context)
        , transcript_base(input_transcript, input_manifest, transcript::HashType::PedersenBlake2s, 16)
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

    int check_field_element_cache(const std::string& element_name) const
    {
        for (size_t i = 0; i < field_keys.size(); ++i) {
            if (field_keys[i] == element_name) {
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    int check_field_element_vector_cache(const std::string& element_name) const
    {
        for (size_t i = 0; i < field_vector_keys.size(); ++i) {
            if (field_vector_keys[i] == element_name) {
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    int check_group_element_cache(const std::string& element_name) const
    {
        for (size_t i = 0; i < group_keys.size(); ++i) {
            if (group_keys[i] == element_name) {
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    int check_challenge_cache(const std::string& challenge_name, const size_t challenge_idx) const
    {
        for (size_t i = 0; i < challenge_keys.size(); ++i) {
            if (challenge_keys[i] == challenge_name) {
                ASSERT(challenge_values[i].size() < challenge_idx);
                return static_cast<int>(i);
            }
        }
        return -1;
    }

    void add_field_element(const std::string& element_name, const field_pt& element)
    {
        std::vector<uint8_t> buffer = element.get_value().to_buffer();
        const size_t element_size = transcript_base.get_element_size(element_name);
        // uint8_t* begin = &buffer[0] + 32 - element_size;
        // uint8_t* end = &buffer[buffer.size()];
        std::vector<uint8_t> sliced_buffer(buffer.end() - (std::ptrdiff_t)element_size, buffer.end());
        transcript_base.add_element(element_name, sliced_buffer);
        field_keys.push_back(element_name);
        field_values.push_back(element);
    }
    // 0x28b96cad7dce47b8e727159ce2adbdd119a4435d6edcba6361209301bd53bd0f
    // 0xb96cad7dce47b8e727159ce2adbdd10019a4435d6edcba6361209301bd53bd0f
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
        transcript_base.add_element(element_name, to_buffer(values));
        field_vector_keys.push_back(element_name);
        field_vector_values.push_back(elements);
    }

    void apply_fiat_shamir(const std::string& challenge_name)
    {
        const size_t num_challenges = get_manifest().get_round_manifest(current_round).num_challenges;
        transcript_base.apply_fiat_shamir(challenge_name);

        if (num_challenges == 0) {
            ++current_round;
            return;
        }
        const size_t bytes_per_element = 31;

        // split work_element into 2 limbs and insert into element_buffer
        // each entry in element_buffer is 31 bytes
        const auto split = [&](field_pt& work_element,
                               std::vector<field_pt>& element_buffer,
                               const field_pt& element,
                               size_t& current_byte_counter,
                               const size_t num_bytes) {
            uint256_t element_u256(element.get_value());
            size_t hi_bytes = bytes_per_element - current_byte_counter;
            if (hi_bytes >= num_bytes) {
                // hmm
                size_t new_byte_counter = current_byte_counter + num_bytes;
                field_pt hi = element;
                const size_t leftovers = bytes_per_element - new_byte_counter;
                field_pt buffer_shift =
                    field_pt(context, barretenberg::fr(uint256_t(1) << ((uint64_t)leftovers * 8ULL)));
                work_element = work_element + (hi * buffer_shift);
                work_element = work_element.normalize();
                current_byte_counter = new_byte_counter;
                if (current_byte_counter == bytes_per_element) {
                    current_byte_counter = 0;
                    element_buffer.push_back(work_element);
                    work_element = field_pt(context, barretenberg::fr(0));
                }
                return;
            }
            const size_t lo_bytes = num_bytes - hi_bytes;
            field_pt lo = witness_t(context, barretenberg::fr(element_u256.slice(0, lo_bytes * 8)));
            field_pt hi = witness_t(context, barretenberg::fr(element_u256.slice(lo_bytes * 8, 256)));
            context->create_range_constraint(lo.witness_index, lo_bytes * 8);
            context->create_range_constraint(hi.witness_index, hi_bytes * 8);
            field_pt shift(context, barretenberg::fr(uint256_t(1ULL) << (uint64_t)lo_bytes * 8ULL));
            field_pt sum = lo + (hi * shift);
            sum = sum.normalize();

            if (element.witness_index != IS_CONSTANT) {
                context->assert_equal(sum.witness_index, element.witness_index);
            } else if (sum.witness_index != IS_CONSTANT) {
                context->assert_equal_constant(sum.witness_index, element.get_value());
            }
            current_byte_counter = (current_byte_counter + num_bytes) % bytes_per_element;

            // if current_byte_counter == 0 we've rolled over
            if (current_byte_counter == 0) {
                element_buffer.push_back(work_element);
                element_buffer.push_back(lo);
                work_element = field_pt(context, 0);
            } else {
                work_element = work_element + hi;

                element_buffer.push_back(work_element);

                field_t lo_shift(
                    context, barretenberg::fr(uint256_t(1ULL) << ((31ULL - (uint64_t)current_byte_counter) * 8ULL)));
                work_element = (lo * lo_shift);
                work_element = work_element.normalize();
            }
        };

        std::vector<field_pt> compression_buffer;
        field_pt working_element(context);

        size_t byte_counter = 0;
        if (current_round > 0) {
            split(working_element, compression_buffer, field_pt(current_challenge), byte_counter, 32);
        }
        for (auto manifest_element : get_manifest().get_round_manifest(current_round).elements) {
            if (manifest_element.num_bytes == 32) {
                split(working_element,
                      compression_buffer,
                      get_field_element(manifest_element.name),
                      byte_counter,
                      manifest_element.num_bytes);
            } else if (manifest_element.num_bytes == 64) {
                group_pt point = get_circuit_group_element(manifest_element.name);

                field_pt y_hi =
                    point.y.binary_basis_limbs[2].element + (point.y.binary_basis_limbs[3].element * fq_pt::shift_1);
                field_pt y_lo =
                    point.y.binary_basis_limbs[0].element + (point.y.binary_basis_limbs[1].element * fq_pt::shift_1);
                field_pt x_hi =
                    point.x.binary_basis_limbs[2].element + (point.x.binary_basis_limbs[3].element * fq_pt::shift_1);
                field_pt x_lo =
                    point.x.binary_basis_limbs[0].element + (point.x.binary_basis_limbs[1].element * fq_pt::shift_1);
                const size_t lo_bytes = fq_pt::NUM_LIMB_BITS / 4;
                const size_t hi_bytes = 32 - lo_bytes;

                split(working_element, compression_buffer, y_hi, byte_counter, hi_bytes);
                split(working_element, compression_buffer, y_lo, byte_counter, lo_bytes);
                split(working_element, compression_buffer, x_hi, byte_counter, hi_bytes);
                split(working_element, compression_buffer, x_lo, byte_counter, lo_bytes);
            } else if (manifest_element.name == "public_inputs") {
                std::vector<field_pt> field_array = get_field_element_vector(manifest_element.name);
                for (size_t i = 0; i < field_array.size(); ++i) {
                    split(working_element, compression_buffer, field_array[i], byte_counter, 32);
                }
            } else if (manifest_element.num_bytes < 32) {
                split(working_element,
                      compression_buffer,
                      get_field_element(manifest_element.name),
                      byte_counter,
                      manifest_element.num_bytes);
            }
        }

        std::vector<byte_array<Composer>> round_challenges;

        if (byte_counter != 0) {
            const uint256_t down_shift = uint256_t(1) << uint256_t((bytes_per_element - byte_counter) * 8);
            working_element = working_element / barretenberg::fr(down_shift);
            working_element = working_element.normalize();

            compression_buffer.push_back(working_element);
        }

        field_pt T0 = pedersen::compress(compression_buffer, true);
        byte_array<Composer> compressed_buffer(T0);

        byte_array<Composer> base_hash = blake2s(compressed_buffer);

        byte_array<Composer> first(field_pt(0), 16);
        first.write(base_hash.slice(0, 16));
        round_challenges.push_back(first);

        if (num_challenges > 1) {
            byte_array<Composer> second(field_pt(0), 16);
            second.write(base_hash.slice(16, 16));
            round_challenges.push_back(second);
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

        challenge_keys.push_back(challenge_name);

        std::vector<field_pt> challenge_elements;
        for (const auto challenge : round_challenges) {
            challenge_elements.push_back(static_cast<field_pt>(challenge));
        }
        challenge_values.push_back(challenge_elements);
    }

    field_pt get_field_element(const std::string& element_name) const
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

    std::vector<field_pt> get_field_element_vector(const std::string& element_name) const
    {
        const int cache_idx = check_field_element_vector_cache(element_name);
        if (cache_idx != -1) {
            return field_vector_values[static_cast<size_t>(cache_idx)];
        }
        std::vector<barretenberg::fr> values = many_from_buffer<fr>(transcript_base.get_element(element_name));
        std::vector<field_pt> result;

        for (size_t i = 0; i < values.size(); ++i) {
            result.push_back(witness_pt(context, values[i]));
        }

        field_vector_keys.push_back(element_name);
        field_vector_values.push_back(result);
        return result;
    }

    bool has_challenge(const std::string& challenge_name) const
    {
        return transcript_base.has_challenge(challenge_name);
    }

    field_pt get_challenge_field_element(const std::string& challenge_name, const size_t challenge_idx = 0) const
    {
        const int cache_idx = check_challenge_cache(challenge_name, challenge_idx);
        ASSERT(cache_idx != -1);
        return challenge_values[static_cast<size_t>(cache_idx)][challenge_idx];
    }

    field_pt get_challenge_field_element_from_map(const std::string& challenge_name,
                                                  const std::string& challenge_map_name) const
    {
        const int challenge_idx = transcript_base.get_challenge_index_from_map(challenge_map_name);
        if (challenge_idx == -1) {
            return field_pt(nullptr, 1);
        }
        const int cache_idx = check_challenge_cache(challenge_name, static_cast<size_t>(challenge_idx));
        ASSERT(cache_idx != -1);
        return challenge_values[static_cast<size_t>(cache_idx)][static_cast<size_t>(challenge_idx)];
    }

    barretenberg::g1::affine_element get_group_element(const std::string& element_name) const
    {
        barretenberg::g1::affine_element value =
            barretenberg::g1::affine_element::serialize_from_buffer(&(transcript_base.get_element(element_name))[0]);
        return value;
    }

    group_pt get_circuit_group_element(const std::string& element_name) const
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

    static fq_pt convert_fq(Composer* ctx, const barretenberg::fq& input) { return fq_pt::from_witness(ctx, input); };

    static group_pt convert_g1(Composer* ctx, const barretenberg::g1::affine_element& input)
    {
        return group_pt::from_witness(ctx, input);
    };

    size_t get_num_challenges(const std::string& challenge_name) const
    {
        return transcript_base.get_num_challenges(challenge_name);
    }

    Composer* context;

  private:
    transcript::Transcript transcript_base;
    byte_array<Composer> current_challenge;

    mutable std::vector<std::string> field_vector_keys;
    mutable std::vector<std::vector<field_pt>> field_vector_values;

    mutable std::vector<std::string> field_keys;
    mutable std::vector<field_pt> field_values;

    mutable std::vector<std::string> group_keys;
    mutable std::vector<group_pt> group_values;

    std::vector<std::string> challenge_keys;
    std::vector<std::vector<field_pt>> challenge_values;

    size_t current_round = 0;
};
} // namespace recursion
} // namespace stdlib
} // namespace plonk