#pragma once

#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/transcript/transcript.hpp"

#include "../../commitment/pedersen/pedersen.hpp"
#include "../../commitment/pedersen/pedersen_plookup.hpp"
#include "../../hash/blake3s/blake3s.hpp"
#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/curves/bn254.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/witness/witness.hpp"
#include "../verification_key/verification_key.hpp"

namespace proof_system::plonk::stdlib::recursion {
template <typename Composer> class Transcript {
  public:
    using field_pt = field_t<Composer>;
    using witness_pt = witness_t<Composer>;
    using fq_pt = bigfield<Composer, barretenberg::Bn254FqParams>;
    using group_pt = element<Composer, fq_pt, field_pt, barretenberg::g1>;
    using Key = verification_key<stdlib::bn254<Composer>>;

    Transcript(Composer* in_context, const transcript::Manifest input_manifest)
        : context(in_context)
        , transcript_base(input_manifest, transcript::HashType::PedersenBlake3s, 16)
        , current_challenge(in_context)
    {}

    Transcript(Composer* in_context,
               const std::vector<uint8_t>& input_transcript,
               const transcript::Manifest input_manifest)
        : context(in_context)
        , transcript_base(input_transcript, input_manifest, transcript::HashType::PedersenBlake3s, 16)
        , current_challenge(in_context)
    /*, transcript_bytes(in_context) */
    {
        // for (size_t i = 0; i < input_transcript.size(); ++i)
        // {
        //     field_t<Composer> data(witness_pt<Composer>(context, input_transcript[i]));
        //     transcript_bytes.write(byte_array<Composer>(data));
        // }
    }

    /**
     * @brief Construct a new Transcript object using a proof represented as a field_pt vector
     *
     * N.B. If proof is represented as a uint8_t vector, Transcript will convert into witnesses in-situ.
     * Use this constructor method if the proof is *already present* as circuit witnesses!
     * @param in_context
     * @param input_manifest
     * @param field_buffer
     * @param num_public_inputs
     */
    Transcript(Composer* in_context,
               const transcript::Manifest input_manifest,
               const std::vector<field_pt>& field_buffer,
               const size_t num_public_inputs)
        : context(in_context)
        , transcript_base(input_manifest, transcript::HashType::PlookupPedersenBlake3s, 16)
        , current_challenge(in_context)
    {
        size_t count = 0;

        const auto num_rounds = input_manifest.get_num_rounds();
        for (size_t i = 0; i < num_rounds; ++i) {
            for (auto manifest_element : input_manifest.get_round_manifest(i).elements) {
                if (!manifest_element.derived_by_verifier) {
                    if (manifest_element.num_bytes == 32 && manifest_element.name != "public_inputs") {
                        add_field_element(manifest_element.name, field_buffer[count++]);
                    } else if (manifest_element.num_bytes == 64 && manifest_element.name != "public_inputs") {
                        const auto x_lo = field_buffer[count++];
                        const auto x_hi = field_buffer[count++];
                        const auto y_lo = field_buffer[count++];
                        const auto y_hi = field_buffer[count++];
                        fq_pt x(x_lo, x_hi);
                        fq_pt y(y_lo, y_hi);
                        group_pt element(x, y);
                        add_group_element(manifest_element.name, element);
                    } else {
                        ASSERT(manifest_element.name == "public_inputs");
                        std::vector<field_pt> public_inputs;
                        for (size_t i = 0; i < num_public_inputs; ++i) {
                            public_inputs.emplace_back(field_buffer[count++]);
                        }
                        add_field_element_vector(manifest_element.name, public_inputs);
                    }
                }
            }
        }
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
                ASSERT(challenge_values[i].size() > challenge_idx);
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
        field_pt working_element(context);

        // maximum number of bytes we can store in a field element w/o wrapping modulus is 31.
        // while we could store more *bits*, we want `preimage_buffer` to mirror how data is formatted
        // when we serialize field/group elements natively (i.e. a byte array)
        static constexpr size_t NUM_BITS_PER_PREIMAGE_ELEMENT = 31UL * 8UL;
        PedersenPreimageBuilder<Composer, NUM_BITS_PER_PREIMAGE_ELEMENT> preimage_buffer(context);
        if (current_round > 0) {
            preimage_buffer.add_element(current_challenge);
        }
        for (auto manifest_element : get_manifest().get_round_manifest(current_round).elements) {
            if (manifest_element.num_bytes == 32 && manifest_element.name != "public_inputs") {
                preimage_buffer.add_element(get_field_element(manifest_element.name));
            } else if (manifest_element.num_bytes == 64 && manifest_element.name != "public_inputs") {
                group_pt point = get_circuit_group_element(manifest_element.name);

                // In our buffer, we want to represent each field element as occupying 256 bits of data (to match what
                // the native transcript does)
                const auto& x = point.x;
                const auto& y = point.y;
                constexpr size_t last_limb_bits = 256 - (fq_pt::NUM_LIMB_BITS * 3);
                preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[3].element,
                                                                           last_limb_bits);
                preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[2].element,
                                                                           fq_pt::NUM_LIMB_BITS);
                preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[1].element,
                                                                           fq_pt::NUM_LIMB_BITS);
                preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[0].element,
                                                                           fq_pt::NUM_LIMB_BITS);
                preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[3].element,
                                                                           last_limb_bits);
                preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[2].element,
                                                                           fq_pt::NUM_LIMB_BITS);
                preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[1].element,
                                                                           fq_pt::NUM_LIMB_BITS);
                preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[0].element,
                                                                           fq_pt::NUM_LIMB_BITS);

            } else if (manifest_element.name == "public_inputs") {
                std::vector<field_pt> field_array = get_field_element_vector(manifest_element.name);
                for (size_t i = 0; i < field_array.size(); ++i) {
                    preimage_buffer.add_element(field_array[i]);
                }
            } else if (manifest_element.num_bytes < 32 && manifest_element.name != "public_inputs") {
                // TODO(zac): init round data is being grabbed out of the manifest and not the vkey
                preimage_buffer.add_element_with_existing_range_constraint(get_field_element(manifest_element.name),
                                                                           manifest_element.num_bytes * 8);
            }
        }
        std::vector<field_pt> round_challenges_new;

        field_pt T0;
        T0 = preimage_buffer.compress(0);

        // helper method to slice a challenge into 128-bit slices
        const auto slice_into_halves = [&](const field_pt& in, const size_t low_bits = 128) {
            uint256_t v = in.get_value();
            uint256_t lo = v.slice(0, low_bits);
            uint256_t hi = v.slice(low_bits, 256);

            field_pt y_lo = field_pt::from_witness(context, lo);
            field_pt y_hi = field_pt::from_witness(context, hi);

            y_lo.create_range_constraint(low_bits);
            y_hi.create_range_constraint(254 - low_bits);

            in.add_two(-y_lo, -y_hi * (uint256_t(1) << low_bits)).assert_equal(0);

            // Validate the sum of our two halves does not exceed the circuit modulus over the integers
            constexpr uint256_t modulus = fr::modulus;
            const field_pt r_lo = field_pt(context, modulus.slice(0, low_bits));
            const field_pt r_hi = field_pt(context, modulus.slice(low_bits, 256));

            bool need_borrow = (uint256_t(y_lo.get_value()) > uint256_t(r_lo.get_value()));
            field_pt borrow = field_pt::from_witness(context, need_borrow);

            // directly call `create_new_range_constraint` to avoid creating an arithmetic gate
            if constexpr (HasPlookup<Composer>) {
                context->create_new_range_constraint(borrow.get_witness_index(), 1, "borrow");
            } else {
                context->create_range_constraint(borrow.get_witness_index(), 1, "borrow");
            }

            // Hi range check = r_hi - y_hi - borrow
            // Lo range check = r_lo - y_lo + borrow * 2^{126}
            field_pt res_hi = (r_hi - y_hi) - borrow;
            field_pt res_lo = (r_lo - y_lo) + (borrow * (uint256_t(1) << low_bits));

            res_hi.create_range_constraint(modulus.get_msb() + 1 - low_bits);
            res_lo.create_range_constraint(low_bits);

            return std::array<field_pt, 2>{ y_lo, y_hi };
        };

        field_pt base_hash;
        if constexpr (HasPlookup<Composer>) {
            base_hash = stdlib::pedersen_plookup_commitment<Composer>::compress(std::vector<field_pt>{ T0 }, 0);
        } else {
            base_hash = stdlib::pedersen_commitment<Composer>::compress(std::vector<field_pt>{ T0 }, 0);
        }
        auto hash_halves = slice_into_halves(base_hash);
        round_challenges_new.push_back(hash_halves[1]);

        if (num_challenges > 1) {
            round_challenges_new.push_back(hash_halves[0]);
        }
        base_hash = (slice_into_halves(base_hash, 8)[1] * 256).normalize();

        // This block of code only executes for num_challenges > 2, which (currently) only happens in the nu round
        // when we need to generate short scalars. In this case, we generate 32-byte challenges and split them in
        // half to get the relevant challenges.
        for (size_t i = 2; i < num_challenges; i += 2) {
            // TODO(@zac-williamson) make this a Poseidon hash not a Pedersen hash
            field_pt hash_output;
            if constexpr (HasPlookup<Composer>) {
                hash_output = stdlib::pedersen_plookup_commitment<Composer>::compress(
                    std::vector<field_pt>{ (base_hash + field_pt(i / 2)).normalize() }, 0);
            } else {
                hash_output = stdlib::pedersen_commitment<Composer>::compress(
                    std::vector<field_pt>{ (base_hash + field_pt(i / 2)).normalize() }, 0);
            }
            auto hash_halves = slice_into_halves(hash_output);
            round_challenges_new.push_back(hash_halves[1]);
            if (i + 1 < num_challenges) {
                round_challenges_new.push_back(hash_halves[0]);
            }
        }
        current_challenge = round_challenges_new[round_challenges_new.size() - 1];
        ++current_round;
        challenge_keys.push_back(challenge_name);

        std::vector<field_pt> challenge_elements;
        for (const auto& challenge : round_challenges_new) {
            challenge_elements.push_back(challenge);
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
    field_pt current_challenge;

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
} // namespace proof_system::plonk::stdlib::recursion
