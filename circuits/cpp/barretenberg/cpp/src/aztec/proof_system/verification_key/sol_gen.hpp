namespace waffle {

/**
 * Write a solidity file containing the vk params to the given stream.
 * Uses StandardPlonk
 **/
inline void output_vk_sol(std::ostream& os, std::shared_ptr<verification_key> const& key, std::string const& class_name)
{
    const auto print_u256 = [&](const std::string& offset, const barretenberg::fr& element, const std::string& name) {
        os << "            mstore(add(_vk, " << offset << "), " << element << ") // " << name << std::endl;
    };

    const auto print_g1 = [&](const std::string& offsetX,
                              const std::string& offsetY,
                              const barretenberg::g1::affine_element& element,
                              const std::string& name) {
        os << "            mstore(add(_vk, " << offsetX << "), " << element.x << ") // " << name << ".x" << std::endl;
        os << "            mstore(add(_vk, " << offsetY << "), " << element.y << ") // " << name << ".y" << std::endl;
    };

    // clang-format off
    os <<
      "// Verification Key Hash: " << key->sha256_hash() << "\n"
      "// SPDX-License-Identifier: Apache-2.0\n"
      "// Copyright 2022 Aztec\n"
      "pragma solidity >=0.8.4;\n"
      "\n"
      "library " << class_name << " {\n"
      "    function verificationKeyHash() internal pure returns(bytes32) {\n"
      "        return 0x" << key->sha256_hash() << ";\n"
      "    }\n\n"
      "    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {\n"
      "        assembly {\n";
    print_u256("0x00", key->domain.size, "vk.circuit_size");
    print_u256("0x20", key->num_public_inputs, "vk.num_inputs");
    print_u256("0x40", key->domain.root, "vk.work_root");
    print_u256("0x60", key->domain.domain_inverse, "vk.domain_inverse");
    print_g1("0x80", "0xa0", key->commitments.at("Q_1"), "vk.Q1");
    print_g1("0xc0", "0xe0", key->commitments.at("Q_2"), "vk.Q2");
    print_g1("0x100", "0x120", key->commitments.at("Q_3"), "vk.Q3");
    print_g1("0x140", "0x160", key->commitments.at("Q_M"), "vk.QM");
    print_g1("0x180", "0x1a0", key->commitments.at("Q_C"), "vk.QC");
    print_g1("0x1c0", "0x1e0", key->commitments.at("SIGMA_1"), "vk.SIGMA1");
    print_g1("0x200", "0x220", key->commitments.at("SIGMA_2"), "vk.SIGMA2");
    print_g1("0x240", "0x260", key->commitments.at("SIGMA_3"), "vk.SIGMA3");
    os <<
        "            mstore(add(_vk, 0x280), " << (key->contains_recursive_proof ? "0x01" : "0x00") << ") // vk.contains_recursive_proof\n"
        "            mstore(add(_vk, 0x2a0), " << (key->contains_recursive_proof ? key->recursive_proof_public_input_indices[0] : 0) << ") // vk.recursive_proof_public_input_indices\n"
        "            mstore(add(_vk, 0x2c0), " <<  key->reference_string->get_g2x().x.c1 << ") // vk.g2_x.X.c1 \n"
        "            mstore(add(_vk, 0x2e0), " <<  key->reference_string->get_g2x().x.c0 << ") // vk.g2_x.X.c0 \n"
        "            mstore(add(_vk, 0x300), " <<  key->reference_string->get_g2x().y.c1 << ") // vk.g2_x.Y.c1 \n"
        "            mstore(add(_vk, 0x320), " <<  key->reference_string->get_g2x().y.c0 << ") // vk.g2_x.Y.c0 \n"
        "            mstore(_omegaInverseLoc, " << key->domain.root_inverse << ") // vk.work_root_inverse\n"
        "        }\n"
        "    }\n"
        "}\n";

    os << std::flush;
}
} // namespace waffle