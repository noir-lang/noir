
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

    vk.circuit_size = 16;
    vk.num_inputs = 0;
    vk.work_root = PairingsBn254.new_fr(
      0x21082ca216cbbf4e1c6e4f4594dd508c996dfbe1174efb98b11509c6e306460b
    );
    vk.domain_inverse = PairingsBn254.new_fr(
      0x2d5e098bb31e86271ccb415b196942d755b0a9c3f21dd9882fa3d63ab1000001
    );
    vk.work_root_inverse = PairingsBn254.new_fr(
      0x02e40daf409556c02bfc85eb303402b774954d30aeb0337eb85a71e6373428de
    );
    vk.Q1 = PairingsBn254.new_g1(
      0x19148a0dc66f32abbdf94f78ba453590a22a88c325fb1530b8e1303785e7cc14,
      0x1c81dde811a1686c0f992874c151e5d5d9dce7421f97b7298c953df3b2318253
    );
    vk.Q2 = PairingsBn254.new_g1(
      0x0b2f8e59c28b02e5e2279a4dcc2d009693a37eb93cb55abcea461bdfc5d89587,
      0x138e7805ab21d8bfe79a908b4410531c9306b7738b24df6c2d991de2823d6e1b
    );
    vk.Q3 = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.Q4 = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.Q5 = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.QM = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.QC = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.QARITH = PairingsBn254.new_g1(
      0x19148a0dc66f32abbdf94f78ba453590a22a88c325fb1530b8e1303785e7cc14,
      0x1c81dde811a1686c0f992874c151e5d5d9dce7421f97b7298c953df3b2318253
    );
    vk.QECC = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.QRANGE = PairingsBn254.new_g1(
      0x001cf4fbd27f1d93535c5fbcc892576c67c253ff0f01b07d154cad398584f8dc,
      0x1442f3ed7beb24cd24138b14407237e675f4d9877ac8f42a2b30a96e04beedbf
    );
    vk.QLOGIC = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.sigma_commitments[0] = PairingsBn254.new_g1(
      0x1b6798c393508d0ccf3ea782b27d53e05e08929b2df2da95e6ea8f60a3ab60c3,
      0x02b84bb490989ea05c551626e0273da5ca710f6c2b64717c862e1d44970d2081
    );
    vk.sigma_commitments[1] = PairingsBn254.new_g1(
      0x0532fe4bb46c211226ee93799f318d6dbcb5a7a0e9badc03a3972cfa5c777141,
      0x27745edfb651c78e9bd537e12638a8cb1302ccb797862a39722cb42349fdfbda
    );
    vk.sigma_commitments[2] = PairingsBn254.new_g1(
      0x127be70ae2f85fbc3abb0e8e3a8105993242f5f6654dd3bef9b5c9d0b084564b,
      0x1bcef3607c3aef5d10b591dd8bb23453b419eabb80b51c7863c878862de46777
    );
    vk.sigma_commitments[3] = PairingsBn254.new_g1(
      0x1fde4d30739f48301455632a5de8b86960d94c6994a1c5022fdb853b22c44de2,
      0x2f07185a41cd606213cdf10556db7346d0b6711ed798e1b119e16b9a4b5f0ddc
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
