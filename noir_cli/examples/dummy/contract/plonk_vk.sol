
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

    vk.circuit_size = 4;
    vk.num_inputs = 0;
    vk.work_root = PairingsBn254.new_fr(
      0x30644e72e131a029048b6e193fd841045cea24f6fd736bec231204708f703636
    );
    vk.domain_inverse = PairingsBn254.new_fr(
      0x244b3ad628e5381f4a3c3448e1210245de26ee365b4b146cf2e9782ef4000001
    );
    vk.work_root_inverse = PairingsBn254.new_fr(
      0x0000000000000000b3c4d79d41a91758cb49c3517c4604a520cff123608fc9cb
    );
    vk.Q1 = PairingsBn254.new_g1(
      0x1f6209c93bc6229eb133c61d37c10820726e63ae692c44478bdebccf18b210ea,
      0x22513fa3d9509cbad1863d5d0e2ac1339f994737ca7e6b8859d95f44e57c92c0
    );
    vk.Q2 = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.Q3 = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.Q4 = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.Q5 = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.QM = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.QC = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.QARITH = PairingsBn254.new_g1(
      0x1f6209c93bc6229eb133c61d37c10820726e63ae692c44478bdebccf18b210ea,
      0x22513fa3d9509cbad1863d5d0e2ac1339f994737ca7e6b8859d95f44e57c92c0
    );
    vk.QECC = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.QRANGE = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.QLOGIC = PairingsBn254.new_g1(
      0x28cb7541caf15f8d8d22228b1b0a6ea2b2dafc9f6d935b7eb2a1432f9fa358f8,
      0x18a29891b391630617b8e8d9e231795c0fe337c34f9095a79f686b15b97fd23c
    );
    vk.sigma_commitments[0] = PairingsBn254.new_g1(
      0x1e8591e3512bcea936b33ac8bd80d8ff10569e194b1d0fd7094aa9a5889f4455,
      0x1621bb09e04b2b0eebb3a18615d48b2aa2432d70eecbd7a7e0c12ba1aa3a88f7
    );
    vk.sigma_commitments[1] = PairingsBn254.new_g1(
      0x20e2ac2f1ca1e0febe1c98bb59edef6d63e49f4bf957f98c7e5e68fcdba93df5,
      0x0a917404d1bf7749c5961a33cbb8b33d67bea384fa35dbd14cc1e3ac285a545c
    );
    vk.sigma_commitments[2] = PairingsBn254.new_g1(
      0x0a86716e0960c1e8123205b0c0524fa19dab55098ea6ae8dd7b4556b4f381ae1,
      0x04defa748b40a4b0703c45eedccce61e6995bad739b1efd8727c4ed4dabd357f
    );
    vk.sigma_commitments[3] = PairingsBn254.new_g1(
      0x0f18c0da1282f4c923dc27c1277f50fa043f5c340dc490fa5270e6af0d52319e,
      0x11445a5e26eb8c7f87f94018dd8f91b6f22bdca20815bb91122fd2d40e860a6c
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
