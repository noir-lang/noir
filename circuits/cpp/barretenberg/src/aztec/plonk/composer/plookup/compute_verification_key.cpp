#include "compute_verification_key.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/types/polynomial_manifest.hpp>
#include <plonk/reference_string/reference_string.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>

using namespace barretenberg;

namespace waffle {
namespace plookup_composer {

std::shared_ptr<verification_key> compute_verification_key(std::shared_ptr<proving_key> const& circuit_proving_key,
                                                           std::shared_ptr<VerifierReferenceString> const& vrs)
{

    std::array<fr*, 22> poly_coefficients;
    poly_coefficients[0] = circuit_proving_key->constraint_selectors.at("q_1").get_coefficients();
    poly_coefficients[1] = circuit_proving_key->constraint_selectors.at("q_2").get_coefficients();
    poly_coefficients[2] = circuit_proving_key->constraint_selectors.at("q_3").get_coefficients();
    poly_coefficients[3] = circuit_proving_key->constraint_selectors.at("q_4").get_coefficients();
    poly_coefficients[4] = circuit_proving_key->constraint_selectors.at("q_5").get_coefficients();
    poly_coefficients[5] = circuit_proving_key->constraint_selectors.at("q_m").get_coefficients();
    poly_coefficients[6] = circuit_proving_key->constraint_selectors.at("q_c").get_coefficients();
    poly_coefficients[7] = circuit_proving_key->constraint_selectors.at("q_arith").get_coefficients();
    poly_coefficients[8] = circuit_proving_key->constraint_selectors.at("q_ecc_1").get_coefficients();
    poly_coefficients[9] = circuit_proving_key->constraint_selectors.at("q_range").get_coefficients();
    poly_coefficients[10] = circuit_proving_key->constraint_selectors.at("q_logic").get_coefficients();
    poly_coefficients[11] = circuit_proving_key->constraint_selectors.at("q_elliptic").get_coefficients();

    poly_coefficients[12] = circuit_proving_key->permutation_selectors.at("sigma_1").get_coefficients();
    poly_coefficients[13] = circuit_proving_key->permutation_selectors.at("sigma_2").get_coefficients();
    poly_coefficients[14] = circuit_proving_key->permutation_selectors.at("sigma_3").get_coefficients();
    poly_coefficients[15] = circuit_proving_key->permutation_selectors.at("sigma_4").get_coefficients();

    poly_coefficients[16] = circuit_proving_key->constraint_selectors.at("table_value_1").get_coefficients();
    poly_coefficients[17] = circuit_proving_key->constraint_selectors.at("table_value_2").get_coefficients();
    poly_coefficients[18] = circuit_proving_key->constraint_selectors.at("table_value_3").get_coefficients();
    poly_coefficients[19] = circuit_proving_key->constraint_selectors.at("table_value_4").get_coefficients();
    poly_coefficients[20] = circuit_proving_key->constraint_selectors.at("table_index").get_coefficients();
    poly_coefficients[21] = circuit_proving_key->constraint_selectors.at("table_type").get_coefficients();

    std::vector<barretenberg::g1::affine_element> commitments;
    commitments.resize(22);

    for (size_t i = 0; i < 22; ++i) {
        commitments[i] =
            g1::affine_element(scalar_multiplication::pippenger(poly_coefficients[i],
                                                                circuit_proving_key->reference_string->get_monomials(),
                                                                circuit_proving_key->n,
                                                                circuit_proving_key->pippenger_runtime_state));
    }

    auto circuit_verification_key =
        std::make_shared<verification_key>(circuit_proving_key->n, circuit_proving_key->num_public_inputs, vrs);

    circuit_verification_key->constraint_selectors.insert({ "Q_1", commitments[0] });
    circuit_verification_key->constraint_selectors.insert({ "Q_2", commitments[1] });
    circuit_verification_key->constraint_selectors.insert({ "Q_3", commitments[2] });
    circuit_verification_key->constraint_selectors.insert({ "Q_4", commitments[3] });
    circuit_verification_key->constraint_selectors.insert({ "Q_5", commitments[4] });
    circuit_verification_key->constraint_selectors.insert({ "Q_M", commitments[5] });
    circuit_verification_key->constraint_selectors.insert({ "Q_C", commitments[6] });
    circuit_verification_key->constraint_selectors.insert({ "Q_ARITHMETIC_SELECTOR", commitments[7] });
    circuit_verification_key->constraint_selectors.insert({ "Q_FIXED_BASE_SELECTOR", commitments[8] });
    circuit_verification_key->constraint_selectors.insert({ "Q_RANGE_SELECTOR", commitments[9] });
    circuit_verification_key->constraint_selectors.insert({ "Q_LOGIC_SELECTOR", commitments[10] });
    circuit_verification_key->constraint_selectors.insert({ "Q_ELLIPTIC", commitments[11] });

    circuit_verification_key->permutation_selectors.insert({ "SIGMA_1", commitments[12] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_2", commitments[13] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_3", commitments[14] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_4", commitments[15] });

    circuit_verification_key->constraint_selectors.insert({ "TABLE_1", commitments[16] });
    circuit_verification_key->constraint_selectors.insert({ "TABLE_2", commitments[17] });
    circuit_verification_key->constraint_selectors.insert({ "TABLE_3", commitments[18] });
    circuit_verification_key->constraint_selectors.insert({ "TABLE_4", commitments[19] });

    circuit_verification_key->constraint_selectors.insert({ "TABLE_INDEX", commitments[20] });
    circuit_verification_key->constraint_selectors.insert({ "TABLE_TYPE", commitments[21] });

    std::copy(plookup_polynomial_manifest,
              plookup_polynomial_manifest + 29,
              std::back_inserter(circuit_verification_key->polynomial_manifest));

    return circuit_verification_key;
}

} // namespace plookup_composer
} // namespace waffle