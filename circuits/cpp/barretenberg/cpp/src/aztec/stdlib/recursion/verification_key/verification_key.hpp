#pragma once
#include <map>
#include <srs/reference_string/reference_string.hpp>
#include <polynomials/evaluation_domain.hpp>

#include <proof_system/types/polynomial_manifest.hpp>

#include <plonk/proof_system/utils/linearizer.hpp>
#include <plonk/proof_system/utils/kate_verification.hpp>
#include <plonk/proof_system/public_inputs/public_inputs.hpp>
#include <plonk/proof_system/utils/linearizer.hpp>

#include <polynomials/polynomial_arithmetic.hpp>

#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/pedersen/pedersen_lookup.hpp>

#include "../../primitives/uint/uint.hpp"
#include "../../primitives/memory/rom_table.hpp"
#include "../../hash/pedersen/pedersen.hpp"
#include "../../hash/pedersen/pedersen_plookup.hpp"
#include "../../primitives/curves/bn254.hpp"

namespace plonk {
namespace stdlib {
namespace recursion {

template <typename Composer> struct evaluation_domain {
    static evaluation_domain from_witness(Composer* ctx, const barretenberg::evaluation_domain& input)
    {
        evaluation_domain domain;
        domain.root = witness_t<Composer>(ctx, input.root);
        domain.root_inverse = domain.root.invert();
        domain.domain = witness_t<Composer>(ctx, input.domain);
        domain.domain_inverse = domain.domain.invert();
        domain.generator = witness_t<Composer>(ctx, input.generator);
        domain.generator_inverse = domain.generator.invert();
        domain.size = domain.domain;
        return domain;
    }

    static evaluation_domain from_constants(Composer* ctx, const barretenberg::evaluation_domain& input)
    {
        evaluation_domain domain;
        domain.root = field_t<Composer>(ctx, input.root);
        domain.root_inverse = field_t<Composer>(ctx, input.root_inverse);
        domain.domain = field_t<Composer>(ctx, input.domain);
        domain.domain_inverse = field_t<Composer>(ctx, input.domain_inverse);
        domain.generator = field_t<Composer>(ctx, input.generator);
        domain.generator_inverse = field_t<Composer>(ctx, input.generator_inverse);
        domain.size = domain.domain;
        return domain;
    }

    field_t<Composer> compress() const
    {
        if constexpr (Composer::type == waffle::ComposerType::PLOOKUP) {
            field_t<Composer> out = pedersen_plookup<Composer>::compress({
                root,
                domain,
                generator,
            });
            return out;
        } else {
            field_t<Composer> out = pedersen<Composer>::compress({
                root,
                domain,
                generator,
            });
            return out;
        }
    }

    static barretenberg::fr compress_native(const barretenberg::evaluation_domain& input)
    {
        barretenberg::fr out;
        if constexpr (Composer::type == waffle::ComposerType::PLOOKUP) {
            out = crypto::pedersen::lookup::compress_native({
                input.root,
                input.domain,
                input.generator,
            });
        } else {
            out = crypto::pedersen::compress_native({
                input.root,
                input.domain,
                input.generator,
            });
        }
        return out;
    }

    field_t<Composer> root;
    field_t<Composer> root_inverse;
    field_t<Composer> domain;
    field_t<Composer> domain_inverse;
    field_t<Composer> generator;
    field_t<Composer> generator_inverse;
    uint32<Composer> size;
};

/**
 * @brief Converts a 'native' verification key into a standard library type, instantiating the `input_key` parameter as
 * circuit variables. This allows the recursive verifier to accept arbitrary verification keys, where the circuit being
 * verified is not fixed as part of the recursive circuit.
 */
template <typename Curve> struct verification_key {
    using Composer = typename Curve::Composer;
    static std::shared_ptr<verification_key> from_witness(Composer* ctx,
                                                          const std::shared_ptr<waffle::verification_key>& input_key)
    {
        std::shared_ptr<verification_key> key = std::make_shared<verification_key>();
        // Native data:
        key->context = ctx;
        key->base_key = input_key;
        key->reference_string = input_key->reference_string;
        key->polynomial_manifest = input_key->polynomial_manifest;

        // Circuit types:
        key->n = witness_t<Composer>(ctx, barretenberg::fr(input_key->n));
        key->num_public_inputs = witness_t<Composer>(ctx, input_key->num_public_inputs);
        key->domain = evaluation_domain<Composer>::from_witness(ctx, input_key->domain);

        for (const auto& [tag, value] : input_key->commitments) {
            key->commitments.insert({ tag, Curve::g1_ct::from_witness(ctx, value) });
        }

        return key;
    }

    static std::shared_ptr<verification_key> from_constants(Composer* ctx,
                                                            const std::shared_ptr<waffle::verification_key>& input_key)
    {
        std::shared_ptr<verification_key> key = std::make_shared<verification_key>();
        key->context = ctx;
        key->base_key = input_key;
        key->n = field_t<Composer>(ctx, input_key->n);
        key->num_public_inputs = field_t<Composer>(ctx, input_key->num_public_inputs);

        key->domain = evaluation_domain<Composer>::from_constants(ctx, input_key->domain);

        key->reference_string = input_key->reference_string;

        for (const auto& [tag, value] : input_key->commitments) {
            key->commitments.insert({ tag, typename Curve::g1_ct(value) });
        }

        key->polynomial_manifest = input_key->polynomial_manifest;

        return key;
    }

    void validate_key_is_in_set(const std::vector<std::shared_ptr<waffle::verification_key>>& keys_in_set)
    {
        const auto circuit_key_compressed = compress();
        bool found = false;
        // if we're using Plookup, use a ROM table to index the keys
        if constexpr (Composer::type == waffle::ComposerType::PLOOKUP) {
            field_t<Composer> key_index(witness_t<Composer>(context, 0));
            std::vector<field_t<Composer>> compressed_keys;
            for (size_t i = 0; i < keys_in_set.size(); ++i) {
                barretenberg::fr compressed = compress_native(keys_in_set[i]);
                compressed_keys.emplace_back(compressed);
                if (compressed == circuit_key_compressed.get_value()) {
                    key_index = witness_t<Composer>(context, i);
                    found = true;
                }
            }
            if (!found) {
                context->failure(
                    "verification_key::validate_key_is_in_set failed - input key is not in the provided set!");
            }
            rom_table<Composer> key_table(compressed_keys);

            const auto output_key = key_table[key_index];
            output_key.assert_equal(circuit_key_compressed);
        } else {
            bool_t<Composer> is_valid(false);
            for (const auto& key : keys_in_set) {
                barretenberg::fr compressed = compress_native(key);
                is_valid = is_valid || (circuit_key_compressed == compressed);
            }

            is_valid.assert_equal(true);
        }
    }

  private:
    field_t<Composer> compress()
    {
        field_t<Composer> compressed_domain = domain.compress();

        std::vector<field_t<Composer>> key_witnesses;
        key_witnesses.push_back(compressed_domain);
        key_witnesses.push_back(num_public_inputs);
        for (const auto& [tag, selector] : commitments) {
            key_witnesses.push_back(selector.x.binary_basis_limbs[0].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[1].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[2].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[3].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[0].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[1].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[2].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[3].element);
        }

        field_t<Composer> compressed_key;
        if constexpr (Composer::type == waffle::ComposerType::PLOOKUP) {
            compressed_key = pedersen_plookup<Composer>::compress(key_witnesses);
        } else {
            compressed_key = pedersen<Composer>::compress(key_witnesses);
        }
        return compressed_key;
    }

    barretenberg::fr compress_native(const std::shared_ptr<waffle::verification_key>& key)
    {
        barretenberg::fr compressed_domain = evaluation_domain<Composer>::compress_native(key->domain);

        constexpr size_t num_limb_bits = bn254<waffle::UltraComposer>::fq_ct::NUM_LIMB_BITS;
        const auto split_bigfield_limbs = [](const uint256_t& element) {
            std::vector<barretenberg::fr> limbs;
            limbs.push_back(element.slice(0, num_limb_bits));
            limbs.push_back(element.slice(num_limb_bits, num_limb_bits * 2));
            limbs.push_back(element.slice(num_limb_bits * 2, num_limb_bits * 3));
            limbs.push_back(element.slice(num_limb_bits * 3, num_limb_bits * 4));
            return limbs;
        };

        std::vector<barretenberg::fr> key_witnesses;
        key_witnesses.push_back(compressed_domain);
        key_witnesses.push_back(key->num_public_inputs);
        for (const auto& [tag, selector] : key->commitments) {
            const auto x_limbs = split_bigfield_limbs(selector.x);
            const auto y_limbs = split_bigfield_limbs(selector.y);

            key_witnesses.push_back(x_limbs[0]);
            key_witnesses.push_back(x_limbs[1]);
            key_witnesses.push_back(x_limbs[2]);
            key_witnesses.push_back(x_limbs[3]);

            key_witnesses.push_back(y_limbs[0]);
            key_witnesses.push_back(y_limbs[1]);
            key_witnesses.push_back(y_limbs[2]);
            key_witnesses.push_back(y_limbs[3]);
        }
        barretenberg::fr compressed_key;
        if constexpr (Composer::type == waffle::ComposerType::PLOOKUP) {
            compressed_key = crypto::pedersen::lookup::compress_native(key_witnesses);
        } else {
            compressed_key = crypto::pedersen::compress_native(key_witnesses);
        }
        return compressed_key;
    }

  public:
    // Circuit Types:
    field_t<Composer> n;
    field_t<Composer> num_public_inputs;
    field_t<Composer> z_pow_n;

    evaluation_domain<Composer> domain;

    std::map<std::string, typename Curve::g1_ct> commitments;

    // Native data:

    std::shared_ptr<waffle::VerifierReferenceString> reference_string;

    waffle::PolynomialManifest polynomial_manifest;

    size_t program_width = 4;

    std::shared_ptr<waffle::verification_key> base_key;
    Composer* context;
};

} // namespace recursion
} // namespace stdlib
} // namespace plonk