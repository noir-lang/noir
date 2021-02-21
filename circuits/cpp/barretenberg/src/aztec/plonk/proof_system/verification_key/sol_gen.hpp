#include "verification_key.hpp"

namespace waffle {

/**
 * Write a solidity file containing the vk params to the given stream.
 **/
inline void output_vk_sol(std::ostream& os, std::shared_ptr<verification_key> const& key, std::string const& class_name)
{
    const auto print_u256 =
        [&](const std::string& name, const barretenberg::fr& element, const std::string& postlabel) {
            os << "            " << name << element << postlabel << std::endl;
        };

    const auto print_g1 =
        [&](const std::string& offset, const barretenberg::g1::affine_element& element, const std::string& postlabel) {
            os << "            mstore(mload(add(vk, " << offset << ")), " << element.x << ")" << postlabel << std::endl;
            os << "            mstore(add(mload(add(vk, " << offset << ")), 0x20), " << element.y << ")" << std::endl;
        };

    // clang-format off
    os <<
      "// SPDX-License-Identifier: GPL-2.0-only\n"
      "// Copyright 2020 Spilsbury Holdings Ltd\n"
      "\n"
      "pragma solidity >=0.6.0 <0.8.0;\n"
      "pragma experimental ABIEncoderV2;\n"
      "\n"
      "import {Types} from '../cryptography/Types.sol';\n"
      "import {Bn254Crypto} from '../cryptography/Bn254Crypto.sol';\n"
      "\n"
      "library " << class_name << " {\n"
      "    using Bn254Crypto for Types.G1Point;\n"
      "    using Bn254Crypto for Types.G2Point;\n"
      "\n"
      "    function get_verification_key() internal pure returns (Types.VerificationKey memory) {\n"
      "        Types.VerificationKey memory vk;\n"
      "\n"
      "        assembly {\n"
      "            mstore(add(vk, 0x00), " << key->domain.size << ") // vk.circuit_size\n"
      "            mstore(add(vk, 0x20), " << key->num_public_inputs << ") // vk.num_inputs\n";

    print_u256("mstore(add(vk, 0x40),", key->domain.root, ") // vk.work_root");
    print_u256("mstore(add(vk, 0x60),", key->domain.domain_inverse, ") // vk.domain_inverse");
    print_u256("mstore(add(vk, 0x80),", key->domain.root_inverse, ") // vk.work_root_inverse");
    print_g1("0xa0", key->constraint_selectors.at("Q_1"), "//vk.Q1");
    print_g1("0xc0", key->constraint_selectors.at("Q_2"), "//vk.Q2");
    print_g1("0xe0", key->constraint_selectors.at("Q_3"), "//vk.Q3");
    print_g1("0x100", key->constraint_selectors.at("Q_4"), "//vk.Q4");
    print_g1("0x120", key->constraint_selectors.at("Q_5"), "//vk.Q5");
    print_g1("0x140", key->constraint_selectors.at("Q_M"), "//vk.QM");
    print_g1("0x160", key->constraint_selectors.at("Q_C"), "//vk.QC");
    print_g1("0x180", key->constraint_selectors.at("Q_ARITHMETIC_SELECTOR"), "//vk.QARITH");
    print_g1("0x1a0", key->constraint_selectors.at("Q_FIXED_BASE_SELECTOR"), "//vk.QECC");
    print_g1("0x1c0", key->constraint_selectors.at("Q_RANGE_SELECTOR"), "//vk.QRANGE");
    print_g1("0x1e0", key->constraint_selectors.at("Q_LOGIC_SELECTOR"), "//vk.QLOGIC");
    print_g1("0x200", key->permutation_selectors.at("SIGMA_1"), "//vk.SIGMA1");
    print_g1("0x220", key->permutation_selectors.at("SIGMA_2"), "//vk.SIGMA2");
    print_g1("0x240", key->permutation_selectors.at("SIGMA_3"), "//vk.SIGMA3");
    print_g1("0x260", key->permutation_selectors.at("SIGMA_4"), "//vk.SIGMA4");
    os <<
      "            mstore(add(vk, 0x280), " << (key->contains_recursive_proof ? "0x01" : "0x00") << ") // vk.contains_recursive_proof\n"
      "            mstore(add(vk, 0x2a0), " << (key->contains_recursive_proof ? key->recursive_proof_public_input_indices[0] : 0) << ") // vk.recursive_proof_public_input_indices\n"
      "            mstore(mload(add(vk, 0x2c0)), " << key->reference_string->get_g2x().x.c1 << ") // vk.g2_x.X.c1\n"
      "            mstore(add(mload(add(vk, 0x2c0)), 0x20), " << key->reference_string->get_g2x().x.c0 << ") // vk.g2_x.X.c0\n"
      "            mstore(add(mload(add(vk, 0x2c0)), 0x40), " << key->reference_string->get_g2x().y.c1 << ") // vk.g2_x.Y.c1\n"
      "            mstore(add(mload(add(vk, 0x2c0)), 0x60), " << key->reference_string->get_g2x().y.c0 << ") // vk.g2_x.Y.c0\n"
      "        }\n"
      "        return vk;\n"
      "    }\n"
      "}\n";

    os << std::flush;
}
} // namespace waffle