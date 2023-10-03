// Verification Key Hash: d25c78098b361876a80895103d19d0586b8ffa8d154cf5b300eda3045a21f200
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library RecursiveUltraVerificationKey {
    function verificationKeyHash() internal pure returns(bytes32) {
        return 0xd25c78098b361876a80895103d19d0586b8ffa8d154cf5b300eda3045a21f200;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000040000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000010) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x19ddbcaf3a8d46c15c0176fbb5b95e4dc57088ff13f4d1bd84c6bfa57dcdc0e0) // vk.work_root
            mstore(add(_vk, 0x60), 0x30644259cd94e7dd5045d7a27013b7fcd21c9e3b7fa75222e7bda49b729b0401) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x2e99805b70f3d61c991b8fd84874e57103c7d7ba60cf60cfe871c92ea7cf3248) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x2359a1c894e0f2af06830fb0d9879b974ec6afa1c95cb8f018780238f8b937e9) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x19b500db732e56fd76e45c1608de1a2d10bce43dbac9ee868a578d68c908c332) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x12de1d2b47110c7e547f2c7dbcb1a229e16333a513afea3226cac0e4f4a50157) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x258112db8f43fcd49b658d699abf5990b03e09ef7f55063ec0a1ff303aa59734) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x2ce5f9e6ce609b428c6b5f17e39dd6947af4073516dd61de721f000bed6b7bc3) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x06984f6692d241b7213fe774c3082e54ca2f254cbb5183f5d213ab93eb527541) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x224652f2a786bcc81dfeba13da0a3ffc1bce4abb2870e9cd91f4c26215b878a1) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x047220d936cff4715b088a0876b290f52a08aedfc88eb111d59cfa88b716a702) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x218375143e04327f9c84e1896dc1eb64cdc13a32aafa1ab7dc9e4c84fbbc61e5) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x0555d41fe3fab5369c4251a1b72b185256fc49fed670153b3aaec40dd7237e38) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x1f83575b2fb33a6e90caedcbd326c1a53ee984eaebd0ec73ebb1a89d2aceb708) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x096c8dfb84e1e95247740d3a2924cef13cb580706db4b1cb242fd883efdb3023) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x056a3687ebe14a74c8529fbb845e86e609a8de9d0b0c92dc838541259dc0f770) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x131e9caa1a0182cacf248327946f2b9bb5a2f13ea7d9195f17b534878a719be2) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x13dc17885405d6756deda93c8d20517dd3a9c93c1ff41a20bf692bbf25696d90) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x1cb14db2c39a1500c4ddb1a75622ca726f2abb263b14245a3fa9804e1530ceac) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x181d870ffe1445d30819a652326e80354eba031560fb2168f75fd59adeaa964e) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x15d25401297c7f1d09ebdedae5140ede85d6a93ffcbdcec78f1d4a94905223bb) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x27f3275e48c07d6a03bb03d5bbc658b7ff658fee03fb7939e45bbcbc1f70cd15) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x193112d61b03cb7a9e4f7af25c3c78a3548a7a64de864168141f21a298a1b872) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x178cdc334092b41699bf1f7cb41965f5089dda63fc10ed5b4b6be111c6064d98) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x1c066e533ca2632e3cb88f56e853b8eb8dfc4f037394aaa2b34fd90b0a52767b) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x0407098851cf2da7d0e0d8ae37ef9e32e6cae22f641ae71bd1dc312be948cd8a) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x1ead1faf379b317c6f778a29ebaa9344f3f2c7aeb42a84a284f32e315b429c63) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x1b49e9a04ab1870e6c25cecb5090f1cf5a39d62b393b0e45ca6c0481483958da) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x0c9d9aadf730ecc1d7deb4ea1ab82744f34fe6c3e8bc5a078aee1829b5e36fda) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x25d36ed174687ce321258b6bdac4ae924ad792a03b9aec923eef6f5093657d1f) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x215a055ec0bf7d7ab5e005b4260258aaadfd8ae9005a09060fdd0cee02dc3fea) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x1841eba177a34b1eb908727fe2e54bf33fc82b6e58dfd044acd4ba05ca80c837) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x018eb037682044ebf9cad76f777bf379b94c4d31d4351ce9677ff146a744555c) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x2bf87d72f0aef257c728503c900516f9274ab06eb54804651218438e40f06c25) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x13b003b384fb50e00994bf62a0057f44344be47383d59a7e9f1319d710ab5263) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x1a5f338a3d05fb46ea46855e6c36dbdb23c5f20a56acc795324fe2958189ec39) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x1365fd683dbad2c4c55b02dd33c4b96fde00e5bb3f52be20ead95484e130aee1) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x2da2ba1d27548e452cc863758acf156eb268f577b7d08ba58e7bbf2d28f6f23c) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x16e9fe7ac7109f057245ceb22e31e1e1b8a8fbf1c6962e926ba5b2505e982d05) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x009a46821fcbdf82b50e323c21ea282115016a12ae0f7f59149cd89eb2357407) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x2066e5c64cb0534e6e825d7852d74375602da9d08c69e11ad65e0ccc194adfd7) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x23735d2cb88ddb998c9209a5bd0dc753c3d3bdf908490e7cdb24d053a15558de) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x29cf07d995b647c3b4a8dbd458ec65ad20f4b38cb193258938b5164ae9bc31a3) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x16ee1de144c9d73a3827323482c0d6882c6ffdd3f21f485a801218e30cdaf143) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x1eb4743b386c88a74762c47d79e0c6f1aac09dc83797c0ff06aae5e77ca93b72) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x1361f17743eeee4cd094e4663957646a3766880e287cacb7a6a4378f51408520) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x1ea01b590a95e3b4d542356cc095198a2710aded8b1b4e58f4de2cb21e82b3e3) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x12b17964421b96b6a35f58cf3b88e22ba39765300bd2a7ebd25e19a0ba80664f) // vk.ID4.y
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
