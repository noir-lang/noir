#pragma once
/**
 * Write a solidity file containing the instance importing vk params to the given stream.
 **/
inline void output_instance(std::ostream& os,
                            std::string const& vk_class_name,
                            std::string const& base_class_name,
                            std::string const& class_name)
{

    std::string vk_filename = "../keys/" + vk_class_name + ".sol";
    std::string base_filename = "../" + base_class_name + ".sol";

    // clang-format off
    os <<
      "// SPDX-License-Identifier: Apache-2.0\n"
      "// Copyright 2023 Aztec\n"
      "pragma solidity >=0.8.4;\n\n"
      "import {" << vk_class_name << " as VK} from \"" << vk_filename << "\";\n"
      "import {" << base_class_name << " as BASE} from \"" << base_filename << "\";\n\n"
      "contract " << class_name << " is BASE {\n"
      "    function getVerificationKeyHash() public pure override(BASE) returns (bytes32) {\n"
      "        return VK.verificationKeyHash();\n"
      "    }\n\n"
      "    function loadVerificationKey(uint256 vk, uint256 _omegaInverseLoc) internal pure virtual override(BASE) {\n"
      "      VK.loadVerificationKey(vk, _omegaInverseLoc);\n"
      "    }\n"
      "}\n";

    os << std::flush;
}
