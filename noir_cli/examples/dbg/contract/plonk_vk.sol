
// SPDX-License-Identifier: GPL-2.0-only
// Copyright 2020 Spilsbury Holdings Ltd

pragma solidity >=0.6.0 <0.7.0;
pragma experimental ABIEncoderV2;

import {Types} from '../cryptography/Types.sol';
import {PairingsBn254} from '../cryptography/PairingsBn254.sol';

library PLONK_VK {
  using PairingsBn254 for Types.G1Point;
  using PairingsBn254 for Types.G2Point;
  using PairingsBn254 for Types.Fr;

  function get_verification_key() internal pure returns (Types.VerificationKey memory) {
    Types.VerificationKey memory vk;

    vk.circuit_size = 16384;
    vk.num_inputs = 0;
    vk.work_root = PairingsBn254.new_fr(
      0x2d965651cdd9e4811f4e51b80ddca8a8b4a93ee17420aae6adaa01c2617c6e85
    );
    vk.domain_inverse = PairingsBn254.new_fr(
      0x30638ce1a7661b6337a964756aa75257c6bf4778d89789ab819ce60c19b04001
    );
    vk.work_root_inverse = PairingsBn254.new_fr(
      0x281c036f06e7e9e911680d42558e6e8cf40976b0677771c0f8eee934641c8410
    );
    vk.Q1 = PairingsBn254.new_g1(
      0x1a8b64dd2f4c02ac608424797cd5f08f43c03c10cad22385f6b7efa30d95a157,
      0x1f42eed12509e6643e7e76cb0d66948df3207c74859b367b597cc738256a2730
    );
    vk.Q2 = PairingsBn254.new_g1(
      0x2789192942e43ce038e77ac34d39c3af54392a1565ad6aad515e33683281a18f,
      0x1d7916c56e54ba1c2d3af1fee734bd09d23d3b54eca30390cb16b5a6bd736857
    );
    vk.Q3 = PairingsBn254.new_g1(
      0x2df0474a4a61592e0325a7c3ae076188d586bff31b1c4220cedd47ce527fb70c,
      0x0dc26a552c4ec9900642e276be158003916f3b050d4e68561287476c83d1d6d4
    );
    vk.Q4 = PairingsBn254.new_g1(
      0x172bee8396003f3ecf604f572c09751bd228d0b24d3ad02fc84cc86993cdc67c,
      0x11443bf0e5b989aaf1bb47530cdd40782cfa6310e200c2193f2ea6afa9af7671
    );
    vk.Q5 = PairingsBn254.new_g1(
      0x1efb3001e685db5fd338f45b8e814cf58f38bb9bdf9bbf90a48d5987d922622e,
      0x22b812eff23aadae8f57e65144b6a9f8af73cb2b997d6630dfd815650ad4d7f9
    );
    vk.QM = PairingsBn254.new_g1(
      0x1b51ba4f4f0ef00e7abd944842ea07c98cb050583f990fb49910b88e8b17ad45,
      0x10a2456ba2c249c706f5a017b6d5f7ccf45100b6edae4d22780ad450902e6885
    );
    vk.QC = PairingsBn254.new_g1(
      0x156db15aaf68eb4bbd650017ac44a4b43e4f08e9a859caf834b3baa49489462f,
      0x1ecaece28388cfa13598faf4bf049a659b021e4c670ce09a27a95455d7ae0a06
    );
    vk.QARITH = PairingsBn254.new_g1(
      0x1d6fb33d50403abbab284a4497d40909f945fcccd9af38b6ca2f3956550cbdb9,
      0x1fed9a10c63f7812353fe404cef2ef29a8c8d895c8225e005d25d4485cac0787
    );
    vk.QECC = PairingsBn254.new_g1(
      0x18c2ad6a07192f135fd3a4cd94fb91e6097f90e9ea1c4f8f4743f6547e37c71a,
      0x1ed7587cf8b1039baa690dc832f954b53b1200d320346660d01fc2270f5aaaf7
    );
    vk.QRANGE = PairingsBn254.new_g1(
      0x0d457f06fd884cdf385e6e69a35c83f599664779e7ec0d4d0691bdf29cfe922d,
      0x1ae458682082cf4a2ffaf6d914385168cf6d33b45e9f7fc01dc6bf6f4fff5547
    );
    vk.QLOGIC = PairingsBn254.new_g1(
      0x2aea5f7d8ef8f5865d1f567dad738dcbadcecebddf73c30f10a530022285c8c7,
      0x036fae6e1743b6de5eafb1447d14dae56583cb709a7035f9438c643ca11ddbd7
    );
    vk.sigma_commitments[0] = PairingsBn254.new_g1(
      0x1d996d6f24b7ea7455169c382b332c5637fc8b63a0f39b002af5aa68fdf7d1dc,
      0x2b9f0d04a412de43856cf8d29a21baf6bbfbd9349282b2e9c46fcdf55e264d5b
    );
    vk.sigma_commitments[1] = PairingsBn254.new_g1(
      0x0ddd6063c25422e756d91eec42b3032000a20cc3adb0806341944cbced1e9d7d,
      0x1c88e5558215f5e50a311f8969a943f5a28d568c2a32df9d5b6bf533060aa0ab
    );
    vk.sigma_commitments[2] = PairingsBn254.new_g1(
      0x078f199e6b24789393041baae5270ae518c5b3d9449e24f1d290dd770821c68c,
      0x1322c9fff937bcbbd210356e32387e448847cfebd24957a22da54f8a63cfe62a
    );
    vk.sigma_commitments[3] = PairingsBn254.new_g1(
      0x25dd948becb64e64a78d50e49d8e44afe3e79277688329cee7b6647cea7ff1f7,
      0x15e064b78dd4397de155c116d86daaadf395e4bf9e06ed13baa9335bd31d1b4f
    );
    vk.permutation_non_residues[0] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000005
    );
    vk.permutation_non_residues[1] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000006
    );
    vk.permutation_non_residues[2] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000007
    );
    vk.contains_recursive_proof = false;
    vk.recursive_proof_indices[0] = 0;
    vk.recursive_proof_indices[1] = 0;
    vk.recursive_proof_indices[2] = 0;
    vk.recursive_proof_indices[3] = 0;
    vk.recursive_proof_indices[4] = 0;
    vk.recursive_proof_indices[5] = 0;
    vk.recursive_proof_indices[6] = 0;
    vk.recursive_proof_indices[7] = 0;
    vk.recursive_proof_indices[8] = 0;
    vk.recursive_proof_indices[9] = 0;
    vk.recursive_proof_indices[10] = 0;
    vk.recursive_proof_indices[11] = 0;
    vk.recursive_proof_indices[12] = 0;
    vk.recursive_proof_indices[13] = 0;
    vk.recursive_proof_indices[14] = 0;
    vk.recursive_proof_indices[15] = 0;
    vk.g2_x = PairingsBn254.new_g2([
      0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1,
      0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0
    ],[
      0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4,
      0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55
    ]);
    return vk;
  }
}
