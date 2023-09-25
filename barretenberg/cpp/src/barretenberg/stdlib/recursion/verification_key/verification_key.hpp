#pragma once
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <map>

#include "barretenberg/plonk/proof_system/types/polynomial_manifest.hpp"

#include "barretenberg/plonk/proof_system/public_inputs/public_inputs.hpp"

#include "barretenberg/polynomials/polynomial_arithmetic.hpp"

#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen_lookup.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"

#include "../../commitment/pedersen/pedersen.hpp"
#include "../../commitment/pedersen/pedersen_plookup.hpp"
#include "../../primitives/curves/bn254.hpp"
#include "../../primitives/memory/rom_table.hpp"
#include "../../primitives/uint/uint.hpp"

#include "barretenberg/crypto/pedersen_commitment/convert_buffer_to_field.hpp"

namespace proof_system::plonk {
namespace stdlib {
namespace recursion {

/**
 * @brief Constructs a packed buffer of field elements to be fed into a Pedersen compress function
 *        Goal is to concatenate multiple inputs together into a single field element if the inputs are known to be
 * small. Produces a vector of field elements where the maximum number of bits per element is `bits_per_element`.
 *
 * @details When calling `pedersen::compress` on the final buffer, we can skip the range checks normally performed in
 * the compress method, because we know the sums of the scalar slices cannot exceed the field modulus. This requires
 * `bits_per_element < modulus bits`
 * @tparam Builder
 * @tparam bits_per_element
 */
template <class Builder, size_t bits_per_element = 248> struct PedersenPreimageBuilder {
    using field_pt = field_t<Builder>;
    using witness_pt = witness_t<Builder>;

    Builder* context;

    PedersenPreimageBuilder(Builder* ctx = nullptr)
        : context(ctx){};

    field_pt compress(const size_t hash_index)
    {
        // we can only use relaxed range checks in pedersen::compress iff bits_per_element < modulus bits
        static_assert(bits_per_element < uint256_t(barretenberg::fr::modulus).get_msb());

        if (current_bit_counter != 0) {
            const uint256_t down_shift = uint256_t(1) << uint256_t((bits_per_element - current_bit_counter));
            for (auto& x : work_element) {
                x = x / barretenberg::fr(down_shift);
            }
            preimage_data.push_back(field_pt::accumulate(work_element));
        }
        if constexpr (HasPlookup<Builder>) {
            return pedersen_plookup_commitment<Builder>::compress_with_relaxed_range_constraints(preimage_data,
                                                                                                 hash_index);
        } else {
            return pedersen_commitment<Builder>::compress(preimage_data, hash_index);
        }
    }

    /**
     * @brief preimage_data is a bit-array where `bits_per_element` number of bits are packed into a single field
     * element
     */
    std::vector<field_pt> preimage_data;

    /**
     * @brief work_element represents the leading element to be added into `preimage_data`.
     *        Vector is composed of field elements that represent bit chunks of a known length,
     *        such that the sum of the bit chunks < bits_per_element
     */
    std::vector<field_pt> work_element;

    size_t current_bit_counter = 0;

    void add_element(const field_pt& element) { slice_element(element, 256); }

    void add_element_with_existing_range_constraint(const field_pt& element, const size_t num_bits)
    {
        slice_element(element, num_bits);
    }

    /**
     * @brief Populate `preimage_data` with element whose size is known to be `num_bits`.
     * `preimage_data` is treated as a bit-array where `bits_per_element` number of bits are packed into a single field
     * element. `slice_element` will:
     *
     * 1. determine how many bits are remaining in work_element
     * 2. if remaining bits > num_bits, slice `element` into 2 chunks hi/lo
     * 3. fill work_element with `hi` chunk (or the full element if possible)
     * 4. (if work_element is full) combine work_element chunks into a field element and push onto `preimage_data`
     * 4. (if required) create a new work_element and populate with `lo`
     *
     * @param element
     * @param num_bits
     */
    void slice_element(const field_pt& element, const size_t num_bits)
    {
        ASSERT(context != nullptr);
        uint256_t element_u256(element.get_value());
        size_t hi_bits = bits_per_element - current_bit_counter;
        if (hi_bits >= num_bits) {
            // hmm
            size_t new_bit_counter = current_bit_counter + num_bits;
            field_pt hi = element;
            const size_t leftovers = bits_per_element - new_bit_counter;
            field_pt buffer_shift = field_pt(context, barretenberg::fr(uint256_t(1) << ((uint64_t)leftovers)));
            work_element.emplace_back(hi * buffer_shift);
            current_bit_counter = new_bit_counter;
            if (current_bit_counter == bits_per_element) {
                current_bit_counter = 0;
                preimage_data.push_back(field_pt::accumulate(work_element));

                work_element = std::vector<field_pt>();
            }
            return;
        }
        const size_t lo_bits = num_bits - hi_bits;
        field_pt lo = witness_t(context, barretenberg::fr(element_u256.slice(0, lo_bits)));
        field_pt hi = witness_t(context, barretenberg::fr(element_u256.slice(lo_bits, 256)));
        lo.create_range_constraint(lo_bits);
        hi.create_range_constraint(hi_bits);
        field_pt shift(context, barretenberg::fr(uint256_t(1ULL) << (uint64_t)lo_bits));
        if (!element.is_constant() || !lo.is_constant() || !hi.is_constant()) {
            lo.add_two(hi * shift, -element).assert_equal(0);
        }

        constexpr uint256_t modulus = barretenberg::fr::modulus;
        constexpr size_t modulus_bits = modulus.get_msb();

        // If our input is a full field element we must validate the sum of our slices is < p
        if (num_bits >= modulus_bits) {
            const field_pt r_lo = field_pt(context, modulus.slice(0, lo_bits));
            const field_pt r_hi = field_pt(context, modulus.slice(lo_bits, num_bits));

            bool need_borrow = (uint256_t(lo.get_value()) > uint256_t(r_lo.get_value()));
            field_pt borrow = field_pt::from_witness(context, need_borrow);

            // directly call `create_new_range_constraint` to avoid creating an arithmetic gate
            if constexpr (HasPlookup<Builder>) {
                context->create_new_range_constraint(borrow.get_witness_index(), 1, "borrow");
            } else {
                context->create_range_constraint(borrow.get_witness_index(), 1, "borrow");
            }
            // Hi range check = r_hi - y_hi - borrow
            // Lo range check = r_lo - y_lo + borrow * 2^{126}
            field_t res_hi = (r_hi - hi) - borrow;
            field_t res_lo = (r_lo - lo) + (borrow * (uint256_t(1) << lo_bits));

            res_hi.create_range_constraint(modulus_bits + 1 - lo_bits);
            res_lo.create_range_constraint(lo_bits);
        }
        current_bit_counter = (current_bit_counter + num_bits) % bits_per_element;

        // if current_bit_counter == 0 we've rolled over
        if (current_bit_counter == 0) {
            work_element.emplace_back(hi);
            preimage_data.push_back(field_pt::accumulate(work_element));
            preimage_data.push_back(lo);
            work_element = std::vector<field_pt>();
        } else {
            work_element.emplace_back(hi);
            preimage_data.push_back(field_pt::accumulate(work_element));
            field_t lo_shift(context,
                             barretenberg::fr(uint256_t(1ULL) << ((bits_per_element - (uint64_t)current_bit_counter))));
            work_element = std::vector<field_pt>();
            work_element.emplace_back(lo * lo_shift);
        }
    };
};

template <typename Builder> struct evaluation_domain {
    static evaluation_domain from_field_elements(const std::vector<field_t<Builder>>& fields)
    {
        evaluation_domain domain;
        domain.root = fields[0];

        domain.root_inverse = domain.root.invert();
        domain.domain = fields[1];
        domain.domain_inverse = domain.domain.invert();
        domain.generator = fields[2];
        domain.generator_inverse = domain.generator.invert();
        domain.size = domain.domain;
        return domain;
    }

    static evaluation_domain from_witness(Builder* ctx, const barretenberg::evaluation_domain& input)
    {
        evaluation_domain domain;
        domain.root = witness_t<Builder>(ctx, input.root);
        domain.root_inverse = domain.root.invert();
        domain.domain = witness_t<Builder>(ctx, input.domain);
        domain.domain_inverse = domain.domain.invert();
        domain.generator = witness_t<Builder>(ctx, input.generator);
        domain.generator_inverse = domain.generator.invert();
        domain.size = domain.domain;
        return domain;
    }

    static evaluation_domain from_constants(Builder* ctx, const barretenberg::evaluation_domain& input)
    {
        evaluation_domain domain;
        domain.root = field_t<Builder>(ctx, input.root);
        domain.root_inverse = field_t<Builder>(ctx, input.root_inverse);
        domain.domain = field_t<Builder>(ctx, input.domain);
        domain.domain_inverse = field_t<Builder>(ctx, input.domain_inverse);
        domain.generator = field_t<Builder>(ctx, input.generator);
        domain.generator_inverse = field_t<Builder>(ctx, input.generator_inverse);
        domain.size = domain.domain;
        return domain;
    }

    field_t<Builder> root;
    field_t<Builder> root_inverse;
    field_t<Builder> domain;
    field_t<Builder> domain_inverse;
    field_t<Builder> generator;
    field_t<Builder> generator_inverse;
    uint32<Builder> size;
};

template <typename Curve> struct verification_key {
    using Builder = typename Curve::Builder;

    static std::shared_ptr<verification_key> from_field_elements(
        Builder* ctx,
        const std::vector<field_t<Builder>>& fields,
        bool inner_proof_contains_recursive_proof = false,
        std::array<uint32_t, 16> recursive_proof_public_input_indices = {})
    {
        std::vector<fr> fields_raw;
        std::shared_ptr<verification_key> key = std::make_shared<verification_key>();
        key->context = ctx;

        key->polynomial_manifest = PolynomialManifest(Builder::CIRCUIT_TYPE);
        key->domain = evaluation_domain<Builder>::from_field_elements({ fields[0], fields[1], fields[2] });

        key->n = fields[3];
        key->num_public_inputs = fields[4];

        // NOTE: For now `contains_recursive_proof` and `recursive_proof_public_input_indices` need to be circuit
        // constants!
        key->contains_recursive_proof = inner_proof_contains_recursive_proof;
        for (size_t i = 0; i < 16; ++i) {
            auto x = recursive_proof_public_input_indices[i];
            key->recursive_proof_public_input_indices.emplace_back(x);
        }

        size_t count = 22;
        for (const auto& descriptor : key->polynomial_manifest.get()) {
            if (descriptor.source == PolynomialSource::SELECTOR || descriptor.source == PolynomialSource::PERMUTATION) {

                const auto x_lo = fields[count++];
                const auto x_hi = fields[count++];
                const auto y_lo = fields[count++];
                const auto y_hi = fields[count++];
                const typename Curve::BaseField x(x_lo, x_hi);
                const typename Curve::BaseField y(y_lo, y_hi);
                const typename Curve::Group element(x, y);

                key->commitments.insert({ std::string(descriptor.commitment_label), element });
            }
        }

        return key;
    }

    /**
     * @brief Converts a 'native' verification key into a standard library type, instantiating the `input_key` parameter
     * as circuit variables. This allows the recursive verifier to accept arbitrary verification keys, where the circuit
     * being verified is not fixed as part of the recursive circuit.
     */
    static std::shared_ptr<verification_key> from_witness(Builder* ctx,
                                                          const std::shared_ptr<plonk::verification_key>& input_key)
    {
        std::shared_ptr<verification_key> key = std::make_shared<verification_key>();
        // Native data:
        key->context = ctx;
        key->reference_string = input_key->reference_string;
        key->polynomial_manifest = input_key->polynomial_manifest;

        // Circuit types:
        key->n = witness_t<Builder>(ctx, barretenberg::fr(input_key->circuit_size));
        key->num_public_inputs = witness_t<Builder>(ctx, input_key->num_public_inputs);
        key->domain = evaluation_domain<Builder>::from_witness(ctx, input_key->domain);
        key->contains_recursive_proof = input_key->contains_recursive_proof;
        key->recursive_proof_public_input_indices = input_key->recursive_proof_public_input_indices;
        for (const auto& [tag, value] : input_key->commitments) {
            // We do not perform on_curve() circuit checks when constructing the Curve::Group element.
            // The assumption is that the circuit creator is honest and that the verification key hash (or some other
            // method) will be used to ensure the provided key matches the key produced by the circuit creator.
            // If the circuit creator is not honest, the entire set of circuit constraints being proved over cannot be
            // trusted!
            const typename Curve::BaseField x = Curve::BaseField::from_witness(ctx, value.x);
            const typename Curve::BaseField y = Curve::BaseField::from_witness(ctx, value.y);
            key->commitments.insert({ tag, typename Curve::Group(x, y) });
        }

        return key;
    }

    static std::shared_ptr<verification_key> from_constants(Builder* ctx,
                                                            const std::shared_ptr<plonk::verification_key>& input_key)
    {
        std::shared_ptr<verification_key> key = std::make_shared<verification_key>();
        key->context = ctx;
        key->n = field_t<Builder>(ctx, input_key->circuit_size);
        key->num_public_inputs = field_t<Builder>(ctx, input_key->num_public_inputs);
        key->contains_recursive_proof = input_key->contains_recursive_proof;
        key->recursive_proof_public_input_indices = input_key->recursive_proof_public_input_indices;

        key->domain = evaluation_domain<Builder>::from_constants(ctx, input_key->domain);

        for (const auto& [tag, value] : input_key->commitments) {
            key->commitments.insert({ tag, typename Curve::Group(value) });
        }

        key->reference_string = input_key->reference_string;
        key->polynomial_manifest = input_key->polynomial_manifest;

        return key;
    }

    void validate_key_is_in_set(const std::vector<std::shared_ptr<plonk::verification_key>>& keys_in_set)
    {
        const auto circuit_key_compressed = compress();
        bool found = false;
        // if we're using Plookup, use a ROM table to index the keys
        if constexpr (HasPlookup<Builder>) {
            field_t<Builder> key_index(witness_t<Builder>(context, 0));
            std::vector<field_t<Builder>> compressed_keys;
            for (size_t i = 0; i < keys_in_set.size(); ++i) {
                barretenberg::fr compressed = compress_native(keys_in_set[i]);
                compressed_keys.emplace_back(compressed);
                if (compressed == circuit_key_compressed.get_value()) {
                    key_index = witness_t<Builder>(context, i);
                    found = true;
                }
            }
            if (!found) {
                context->failure(
                    "verification_key::validate_key_is_in_set failed - input key is not in the provided set!");
            }
            rom_table<Builder> key_table(compressed_keys);

            const auto output_key = key_table[key_index];
            output_key.assert_equal(circuit_key_compressed);
        } else {
            bool_t<Builder> is_valid(false);
            for (const auto& key : keys_in_set) {
                barretenberg::fr compressed = compress_native(key);
                is_valid = is_valid || (circuit_key_compressed == compressed);
            }

            is_valid.assert_equal(true);
        }
    }

  public:
    field_t<Builder> compress(size_t const hash_index = 0)
    {
        PedersenPreimageBuilder<Builder> preimage_buffer(context);

        field_t<Builder> circuit_type =
            witness_t<Builder>::create_constant_witness(context, static_cast<uint32_t>(Builder::CIRCUIT_TYPE));
        domain.generator.create_range_constraint(16, "domain.generator");
        domain.domain.create_range_constraint(32, "domain.generator");
        num_public_inputs.create_range_constraint(32, "num_public_inputs");
        preimage_buffer.add_element_with_existing_range_constraint(circuit_type, 8);
        preimage_buffer.add_element_with_existing_range_constraint(domain.generator, 16); // coset generator is small
        preimage_buffer.add_element_with_existing_range_constraint(domain.domain, 32);
        preimage_buffer.add_element_with_existing_range_constraint(num_public_inputs, 32);
        constexpr size_t limb_bits = Curve::BaseField::NUM_LIMB_BITS;
        constexpr size_t last_limb_bits = 256 - (limb_bits * 3);
        for (const auto& [tag, selector] : commitments) {
            const auto& x = selector.x;
            const auto& y = selector.y;
            preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[3].element, last_limb_bits);
            preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[2].element, limb_bits);
            preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[1].element, limb_bits);
            preimage_buffer.add_element_with_existing_range_constraint(y.binary_basis_limbs[0].element, limb_bits);
            preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[3].element, last_limb_bits);
            preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[2].element, limb_bits);
            preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[1].element, limb_bits);
            preimage_buffer.add_element_with_existing_range_constraint(x.binary_basis_limbs[0].element, limb_bits);
        }
        preimage_buffer.add_element(domain.root);
        field_t<Builder> compressed_key = preimage_buffer.compress(hash_index);
        return compressed_key;
    }

    static barretenberg::fr compress_native(const std::shared_ptr<plonk::verification_key>& key,
                                            const size_t hash_index = 0)
    {
        std::vector<uint8_t> preimage_data;

        preimage_data.push_back(static_cast<uint8_t>(Builder::CIRCUIT_TYPE));

        const uint256_t domain = key->domain.domain;
        const uint256_t generator = key->domain.generator;
        const uint256_t num_public_inputs = key->num_public_inputs;

        ASSERT(domain < (uint256_t(1) << 32));
        ASSERT(generator < (uint256_t(1) << 16));
        ASSERT(num_public_inputs < (uint256_t(1) << 32));

        write(preimage_data, static_cast<uint16_t>(uint256_t(key->domain.generator)));
        write(preimage_data, static_cast<uint32_t>(uint256_t(key->domain.domain)));
        write(preimage_data, static_cast<uint32_t>(key->num_public_inputs));
        for (const auto& [tag, selector] : key->commitments) {
            write(preimage_data, selector.y);
            write(preimage_data, selector.x);
        }

        write(preimage_data, key->domain.root);

        barretenberg::fr compressed_key;
        if constexpr (HasPlookup<Builder>) {
            compressed_key = from_buffer<barretenberg::fr>(
                crypto::pedersen_commitment::lookup::compress_native(preimage_data, hash_index));
        } else {
            compressed_key = crypto::pedersen_commitment::compress_native(preimage_data, hash_index);
        }
        return compressed_key;
    }

  public:
    // Circuit Types:
    field_t<Builder> n;
    field_t<Builder> num_public_inputs;
    field_t<Builder> z_pow_n;

    evaluation_domain<Builder> domain;

    std::map<std::string, typename Curve::Group> commitments;

    // Native data:

    std::shared_ptr<barretenberg::srs::factories::VerifierCrs<curve::BN254>> reference_string;

    PolynomialManifest polynomial_manifest;
    // Used to check in the circuit if a proof contains any aggregated state.
    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    size_t program_width = 4;
    Builder* context;
};

} // namespace recursion
} // namespace stdlib
} // namespace proof_system::plonk
