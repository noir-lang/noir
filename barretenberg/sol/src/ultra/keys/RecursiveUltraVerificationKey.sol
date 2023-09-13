// Verification Key Hash: b665bc769f274feb94ea7f9997fa684b414aa8b9b9bac0227c7ce2e1cbd3d115
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library RecursiveUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0xb665bc769f274feb94ea7f9997fa684b414aa8b9b9bac0227c7ce2e1cbd3d115;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000040000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000010) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x19ddbcaf3a8d46c15c0176fbb5b95e4dc57088ff13f4d1bd84c6bfa57dcdc0e0) // vk.work_root
            mstore(add(_vk, 0x60), 0x30644259cd94e7dd5045d7a27013b7fcd21c9e3b7fa75222e7bda49b729b0401) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x16f7fc6133c8fb2dab06c57392df697a53357ecd918d749d1c981dcd0ee6d849) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x2ba047103f9f86b84058d718a082e2faa53e50109e7cb880d2cbb7a1bf98da89) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x1b9d146737dbb7759e0cad93ad4a7669880a062aceb7b46b8485327976d7285c) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x11de7c3d638acc90e7f844c08658d0588da864268e00576d26aaca3cf49af350) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x1466840d8ad2dfde3a55d4c98412a05807bbe8aac33c27ba100c1e621fbebba0) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x2198ce44955b8ac6e21ddcbb66acd9df7596ad9e5fcf22f2227e8bbb51fe44ee) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x18b96a49db3644e2986f811b8c104e8eb88aa5eb9aec0ca109322a64885688bd) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x2ffec963826849cabd279a2b9f9a26f81518eb65d882f47a32470fc52f53def0) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x09dd725897471fddc177b241d7abc402705acfa452707388fa62666ad454598c) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x03a46eb7ed69136e109e2761fb707da7cee18b3d05e581f24d77853b3b03581e) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x304db51670cb2c59e3088431803e82bce8c81b38eefa267871ae2103ca7842ca) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x1d7ec7d8d4a74e337de26b7adaecb8beb03d8cd647aa180bc08de840038710d5) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x1db65122bf0f0a58fe07bd7342d3e26b07923041cb7d2158d13fb7b5328da40e) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x1691db1eeedbcb4f7646959cf363c00b7e26812a225edf5a6972d815270770f5) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x2a63b6a306e30d87f4b8597cbd1dcecff5fc7cacb774247fca6531e3d347ada4) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x2849d2901fcd1f048924fb77e9451ad45d80f9f842418146b1fde0a7c752fc5f) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x0e42866979ddac27ac729352dd0f844da4fb5a1c3e2480b5b940acd12304c700) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x017ac9a40547e866bdb914dc2b73661c0ec8aa67956c8c9bf406795f75e15c53) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x1ad08199bf79952adff0aa3a9c04a26f18ad7deed1fbed0548f2c83ddf913ef9) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x151df9277b110c615c058f7f783105d03cab938f23884afed1897d0049715d21) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x0bd26d62138b721fdc08fd7d52cd3dfaa37399eb416af0ec6237f9ec1a63a5c0) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x103282cd2ef4210ac390d70a1cba58c6792a5d872ae0337615f8ac9997d300ef) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x08abaa91c69ffa73d80d9a9562020c2a104771f07cf4099cbbe9a0071befb1cc) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x1a82e5cd4a2c3de77afb2ca76c89b54991a4db3939a5c24806af01a0f69a2366) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x26d50e2d19c429d1a2987d5249b88e388f93339fc05f52939fa2e1f4be653918) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x0a49cd57e79633ea43cc3172e819327ce260682d8b571d0964678a153c17e959) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x1c82f3e7c57b08ef90fda6fe39427b815a835c8559b64eac0a4b213998f6802c) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x098bad014a270b6f5e4c90cbd299c15c5fd190457f0e78a5f849243e86688868) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x215a055ec0bf7d7ab5e005b4260258aaadfd8ae9005a09060fdd0cee02dc3fea) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x1841eba177a34b1eb908727fe2e54bf33fc82b6e58dfd044acd4ba05ca80c837) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x018eb037682044ebf9cad76f777bf379b94c4d31d4351ce9677ff146a744555c) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x2bf87d72f0aef257c728503c900516f9274ab06eb54804651218438e40f06c25) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x13b003b384fb50e00994bf62a0057f44344be47383d59a7e9f1319d710ab5263) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x1a5f338a3d05fb46ea46855e6c36dbdb23c5f20a56acc795324fe2958189ec39) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x1365fd683dbad2c4c55b02dd33c4b96fde00e5bb3f52be20ead95484e130aee1) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x2da2ba1d27548e452cc863758acf156eb268f577b7d08ba58e7bbf2d28f6f23c) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x0ef908712f03ce2e4db3ef557abbde7c584d8c831165ba40ab43124526c53cc1) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x009dd642bc5eb1869048b59d2052645208cc5a14537814568d9c985c93319e55) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x0f973c9af1150675ae6dac1ea8ea366e5b8db13bb9c2237ab11c40dfb644ebf5) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x06b0c966f9edab490ac15a176d35d56996cc66854268197989a53ab0d1368188) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x09e719130bb46416efa070d08d82cc07fe0ed3bd8685616b92b4b9619e0807b2) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x18f35ee01438dda2443da27299404d09ccfff098a0ceac2e9a10bf2a96bc11ac) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x0cb835c737d324b9ff5bba45988dc4921104803b7e37649f8c628f0de26361ac) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x18ca0ac87859387aa32c6939f7a4a0d322879a3fdb1ef85d06addcddc13acea5) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x0047304b09efd9315a96d9e802c9a50c1964076026e5f17aff825d6cfc38d823) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x21c9f3aa4cbe8ee21422052f7c22d3d8a5a9a89c262a5a5cb52d8802f6106c49) // vk.ID4.y
            mstore(add(_vk, 0x640), 0x01) // vk.contains_recursive_proof
            mstore(add(_vk, 0x660), 0) // vk.recursive_proof_public_input_indices
            mstore(add(_vk, 0x680), 0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1) // vk.g2_x.X.c1
            mstore(add(_vk, 0x6a0), 0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0) // vk.g2_x.X.c0
            mstore(add(_vk, 0x6c0), 0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4) // vk.g2_x.Y.c1
            mstore(add(_vk, 0x6e0), 0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55) // vk.g2_x.Y.c0
            mstore(_omegaInverseLoc, 0x036853f083780e87f8d7c71d111119c57dbe118c22d5ad707a82317466c5174c) // vk.work_root_inverse
        }
    }
}
