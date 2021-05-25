#pragma once
#include <map>
#include <plonk/reference_string/reference_string.hpp>
#include <polynomials/evaluation_domain.hpp>

#include <plonk/proof_system/types/polynomial_manifest.hpp>

#include <plonk/proof_system/utils/linearizer.hpp>
#include <plonk/proof_system/utils/kate_verification.hpp>
#include <plonk/proof_system/public_inputs/public_inputs.hpp>
#include <plonk/proof_system/utils/linearizer.hpp>

#include <polynomials/polynomial_arithmetic.hpp>

#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <crypto/pedersen/pedersen.hpp>

#include "../../primitives/uint/uint.hpp"
#include "../../hash/pedersen/pedersen.hpp"

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
        field_t<Composer> out = pedersen<Composer>::compress({
            root,
            domain,
            generator,
        });
        return out;
    }

    static barretenberg::fr compress_native(const barretenberg::evaluation_domain& input)
    {
        barretenberg::fr out = crypto::pedersen::compress_native({
            input.root,
            input.domain,
            input.generator,
        });
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

// stdlib verification key
// converts a verification key into a standard library type, instantiating the key's parameters
// as circuit variables. This allows the recursive verifier to accept arbitrary verification keys,
// where the circuit being verified is not fixed as part of the recursive circuit
template <typename Curve> struct verification_key {
    using Composer = typename Curve::Composer;
    static std::shared_ptr<verification_key> from_witness(Composer* ctx,
                                                          const std::shared_ptr<waffle::verification_key>& input_key)
    {
        std::shared_ptr<verification_key> key = std::make_shared<verification_key>();
        key->base_key = input_key;
        key->n = witness_t<Composer>(ctx, barretenberg::fr(input_key->n));
        key->num_public_inputs = witness_t<Composer>(ctx, input_key->num_public_inputs);
        key->domain = evaluation_domain<Composer>::from_witness(ctx, input_key->domain);
        key->reference_string = input_key->reference_string;
        for (const auto& [tag, value] : input_key->constraint_selectors) {
            key->constraint_selectors.insert({ tag, Curve::g1_ct::from_witness(ctx, value) });
        }
        for (const auto& [tag, value] : input_key->permutation_selectors) {
            key->permutation_selectors.insert({ tag, Curve::g1_ct::from_witness(ctx, value) });
        }
        std::copy(input_key->polynomial_manifest.begin(),
                  input_key->polynomial_manifest.end(),
                  std::back_inserter(key->polynomial_manifest));
        return key;
    }

    static std::shared_ptr<verification_key> from_constants(Composer* ctx,
                                                            const std::shared_ptr<waffle::verification_key>& input_key)
    {
        std::shared_ptr<verification_key> key = std::make_shared<verification_key>();
        key->base_key = input_key;
        key->n = field_t<Composer>(ctx, input_key->n);
        key->num_public_inputs = field_t<Composer>(ctx, input_key->num_public_inputs);

        key->domain = evaluation_domain<Composer>::from_constants(ctx, input_key->domain);

        key->reference_string = input_key->reference_string;

        for (const auto& [tag, value] : input_key->constraint_selectors) {
            key->constraint_selectors.insert({ tag, typename Curve::g1_ct(value) });
        }
        for (const auto& [tag, value] : input_key->permutation_selectors) {
            key->permutation_selectors.insert({ tag, typename Curve::g1_ct(value) });
        }
        std::copy(input_key->polynomial_manifest.begin(),
                  input_key->polynomial_manifest.end(),
                  std::back_inserter(key->polynomial_manifest));
        return key;
    }

    void validate_key_is_in_set(const std::vector<std::shared_ptr<waffle::verification_key>>& keys_in_set)
    {
        const auto circuit_key_compressed = compress();
        bool_t<Composer> is_valid(false);
        for (const auto& key : keys_in_set) {
            barretenberg::fr compressed = compress_native(key);
            is_valid = is_valid || (circuit_key_compressed == compressed);
        }

        is_valid.assert_equal(true);
    }

  private:
    field_t<Composer> compress()
    {
        field_t<Composer> compressed_domain = domain.compress();

        std::vector<field_t<Composer>> key_witnesses;
        key_witnesses.push_back(compressed_domain);
        key_witnesses.push_back(num_public_inputs);
        for (const auto& [tag, selector] : constraint_selectors) {
            key_witnesses.push_back(selector.x.binary_basis_limbs[0].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[1].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[2].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[3].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[0].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[1].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[2].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[3].element);
        }
        for (const auto& [tag, selector] : permutation_selectors) {
            key_witnesses.push_back(selector.x.binary_basis_limbs[0].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[1].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[2].element);
            key_witnesses.push_back(selector.x.binary_basis_limbs[3].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[0].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[1].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[2].element);
            key_witnesses.push_back(selector.y.binary_basis_limbs[3].element);
        }

        field_t<Composer> compressed_key = pedersen<Composer>::compress(key_witnesses);

        return compressed_key;
    }

    barretenberg::fr compress_native(const std::shared_ptr<waffle::verification_key>& key)
    {
        barretenberg::fr compressed_domain = evaluation_domain<Composer>::compress_native(key->domain);

        constexpr size_t num_limb_bits = 68; // TODO GET FROM BIGFIELD
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
        for (const auto& [tag, selector] : key->constraint_selectors) {
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
        for (const auto& [tag, selector] : key->permutation_selectors) {
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
        barretenberg::fr compressed_key = crypto::pedersen::compress_native(key_witnesses);
        return compressed_key;
    }

  public:
    field_t<Composer> n;
    field_t<Composer> num_public_inputs;
    field_t<Composer> z_pow_n;

    evaluation_domain<Composer> domain;

    std::shared_ptr<waffle::VerifierReferenceString> reference_string;

    std::map<std::string, typename Curve::g1_ct> constraint_selectors;
    std::map<std::string, typename Curve::g1_ct> permutation_selectors;

    std::vector<waffle::PolynomialDescriptor> polynomial_manifest;

    size_t program_width = 4;

    std::shared_ptr<waffle::verification_key> base_key;
};

} // namespace recursion
} // namespace stdlib
} // namespace plonk