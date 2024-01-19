namespace bb {

/**
 * Write a solidity file containing the vk params to the given stream.
 * Uses StandardPlonk
 **/
inline void output_vk_sol_standard(std::ostream& os,
                                   std::shared_ptr<plonk::verification_key> const& key,
                                   std::string const& class_name)
{
    const auto print_u256 = [&](const std::string& offset, const bb::fr& element, const std::string& name) {
        os << "            mstore(add(_vk, " << offset << "), " << element << ") // " << name << std::endl;
    };

    const auto print_g1 = [&](const std::string& offsetX,
                              const std::string& offsetY,
                              const bb::g1::affine_element& element,
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


/**
 * Write a solidity file containing the vk params to the given stream.
 * Uses UltraPlonk
 **/
inline void output_vk_sol_ultra(std::ostream& os, std::shared_ptr<plonk::verification_key> const& key, std::string const& class_name)
{
    const auto print_u256 = [&](const std::string& offset, const bb::fr& element, const std::string& name) {
        os << "            mstore(add(_vk, " << offset << "), " << element << ") // " << name << std::endl;
    };

    const auto print_g1 = [&](const std::string& offsetX,
                              const std::string& offsetY,
                              const bb::g1::affine_element& element,
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
    print_g1("0x140", "0x160", key->commitments.at("Q_4"), "vk.Q4");
    print_g1("0x180", "0x1a0", key->commitments.at("Q_M"), "vk.Q_M");
    print_g1("0x1c0", "0x1e0", key->commitments.at("Q_C"), "vk.Q_C");
    print_g1("0x200", "0x220", key->commitments.at("Q_ARITHMETIC"), "vk.Q_ARITHMETIC");
    print_g1("0x240", "0x260", key->commitments.at("Q_SORT"), "vk.QSORT");
    print_g1("0x280", "0x2a0", key->commitments.at("Q_ELLIPTIC"), "vk.Q_ELLIPTIC");
    print_g1("0x2c0", "0x2e0", key->commitments.at("Q_AUX"), "vk.Q_AUX");
    print_g1("0x300", "0x320", key->commitments.at("SIGMA_1"), "vk.SIGMA1");
    print_g1("0x340", "0x360", key->commitments.at("SIGMA_2"), "vk.SIGMA2");
    print_g1("0x380", "0x3a0", key->commitments.at("SIGMA_3"), "vk.SIGMA3");
    print_g1("0x3c0", "0x3e0", key->commitments.at("SIGMA_4"), "vk.SIGMA4");
    print_g1("0x400", "0x420", key->commitments.at("TABLE_1"), "vk.TABLE1");
    print_g1("0x440", "0x460", key->commitments.at("TABLE_2"), "vk.TABLE2");
    print_g1("0x480", "0x4a0", key->commitments.at("TABLE_3"), "vk.TABLE3");
    print_g1("0x4c0", "0x4e0", key->commitments.at("TABLE_4"), "vk.TABLE4");
    print_g1("0x500", "0x520", key->commitments.at("TABLE_TYPE"), "vk.TABLE_TYPE");
    print_g1("0x540", "0x560", key->commitments.at("ID_1"), "vk.ID1");
    print_g1("0x580", "0x5a0", key->commitments.at("ID_2"), "vk.ID2");
    print_g1("0x5c0", "0x5e0", key->commitments.at("ID_3"), "vk.ID3");
    print_g1("0x600", "0x620", key->commitments.at("ID_4"), "vk.ID4");
    os <<
        "            mstore(add(_vk, 0x640), " << (key->contains_recursive_proof ? "0x01" : "0x00") << ") // vk.contains_recursive_proof\n"
        "            mstore(add(_vk, 0x660), " << (key->contains_recursive_proof ? key->recursive_proof_public_input_indices[0] : 0) << ") // vk.recursive_proof_public_input_indices\n"
        "            mstore(add(_vk, 0x680), " <<  key->reference_string->get_g2x().x.c1 << ") // vk.g2_x.X.c1 \n"
        "            mstore(add(_vk, 0x6a0), " <<  key->reference_string->get_g2x().x.c0 << ") // vk.g2_x.X.c0 \n"
        "            mstore(add(_vk, 0x6c0), " <<  key->reference_string->get_g2x().y.c1 << ") // vk.g2_x.Y.c1 \n"
        "            mstore(add(_vk, 0x6e0), " <<  key->reference_string->get_g2x().y.c0 << ") // vk.g2_x.Y.c0 \n"
        "            mstore(_omegaInverseLoc, " << key->domain.root_inverse << ") // vk.work_root_inverse\n"
        "        }\n"
        "    }\n"
        "}\n";

    os << std::flush;
}

/**
 * @brief Wrapper method to output a solidity verification key. Composer type determined from key
 *
 * @param os
 * @param key
 * @param class_name
 */
inline void output_vk_sol(std::ostream& os, std::shared_ptr<plonk::verification_key> const& key, std::string const& class_name)
{
    CircuitType circuit_type = static_cast<CircuitType>(key->circuit_type);
    switch (circuit_type) {
    case CircuitType::STANDARD: {
        return output_vk_sol_standard(os, key, class_name);
        break;
    }
    case CircuitType::ULTRA: {
        return output_vk_sol_ultra(os, key, class_name);
        break;
    }
    default: {
        std::cerr << "bb::output_vk_sol unsupported composer type. Defaulting to standard composer" << std::endl;
        return output_vk_sol_standard(os, key, class_name);
    }
    }
}
} // namespace bb
