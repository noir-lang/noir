#include "verification_key.hpp"

namespace waffle {

/**
 * Write a solidity file containing the vk params to the given stream.
 **/
inline void output_vk_sol(std::ostream& os, std::shared_ptr<verification_key> const& key, std::string const& class_name)
{
    const auto print_fr = [&](const std::string& name, const barretenberg::fr& element) {
        os << "    vk." << name << " = PairingsBn254.new_fr(" << std::endl;
        os << "      " << element << std::endl;
        os << "    );" << std::endl;
    };
    const auto print_g1 = [&](const std::string& name, const barretenberg::g1::affine_element& element) {
        os << "    vk." << name << " = PairingsBn254.new_g1(" << std::endl;
        os << "      " << element.x << "," << std::endl;
        os << "      " << element.y << std::endl;
        os << "    );" << std::endl;
    };

    // clang-format off
    os <<
      "// SPDX-License-Identifier: GPL-2.0-only\n"
      "// Copyright 2020 Spilsbury Holdings Ltd\n"
      "\n"
      "pragma solidity >=0.6.0 <0.7.0;\n"
      "pragma experimental ABIEncoderV2;\n"
      "\n"
      "import {Types} from '../cryptography/Types.sol';\n"
      "import {PairingsBn254} from '../cryptography/PairingsBn254.sol';\n"
      "\n"
      "library " << class_name << " {\n"
      "  using PairingsBn254 for Types.G1Point;\n"
      "  using PairingsBn254 for Types.G2Point;\n"
      "  using PairingsBn254 for Types.Fr;\n"
      "\n"
      "  function get_verification_key() internal pure returns (Types.VerificationKey memory) {\n"
      "    Types.VerificationKey memory vk;\n"
      "\n"
      "    vk.circuit_size = " << key->domain.size << ";\n"
      "    vk.num_inputs = " << key->num_public_inputs << ";\n";
    print_fr("work_root", key->domain.root);
    print_fr("domain_inverse", key->domain.domain_inverse);
    print_fr("work_root_inverse", key->domain.root_inverse);
    print_g1("Q1", key->constraint_selectors.at("Q_1"));
    print_g1("Q2", key->constraint_selectors.at("Q_2"));
    print_g1("Q3", key->constraint_selectors.at("Q_3"));
    print_g1("Q4", key->constraint_selectors.at("Q_4"));
    print_g1("Q5", key->constraint_selectors.at("Q_5"));
    print_g1("QM", key->constraint_selectors.at("Q_M"));
    print_g1("QC", key->constraint_selectors.at("Q_C"));
    print_g1("QARITH", key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR"));
    print_g1("QECC", key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR"));
    print_g1("QRANGE", key->constraint_selectors.at("Q_RANGE_SELECTOR"));
    print_g1("QLOGIC", key->constraint_selectors.at("Q_LOGIC_SELECTOR"));
    print_g1("sigma_commitments[0]", key->permutation_selectors.at("SIGMA_1"));
    print_g1("sigma_commitments[1]", key->permutation_selectors.at("SIGMA_2"));
    print_g1("sigma_commitments[2]", key->permutation_selectors.at("SIGMA_3"));
    print_g1("sigma_commitments[3]", key->permutation_selectors.at("SIGMA_4"));
    print_fr("permutation_non_residues[0]", 5);
    print_fr("permutation_non_residues[1]", 6);
    print_fr("permutation_non_residues[2]", 7);

    os <<
      "    vk.g2_x = PairingsBn254.new_g2([\n"
      "      " << key->reference_string->get_g2x().x.c1 << ",\n" <<
      "      " << key->reference_string->get_g2x().x.c0 << "\n"
      "    ],[\n"
      "      " << key->reference_string->get_g2x().y.c1 << ",\n" <<
      "      " << key->reference_string->get_g2x().y.c0 << "\n"
      "    ]);\n"
      "    return vk;\n"
      "  }\n"
      "}\n";

    os << std::flush;
}

} // namespace waffle