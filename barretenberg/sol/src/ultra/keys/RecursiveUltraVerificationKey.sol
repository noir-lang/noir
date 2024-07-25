// Verification Key Hash: 9e6cf5dacef11085d9ea83e98b85ebdc37749931c90443898dcd8d18f639dad8
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library RecursiveUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0x9e6cf5dacef11085d9ea83e98b85ebdc37749931c90443898dcd8d18f639dad8;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000040000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000010) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x19ddbcaf3a8d46c15c0176fbb5b95e4dc57088ff13f4d1bd84c6bfa57dcdc0e0) // vk.work_root
            mstore(add(_vk, 0x60), 0x30644259cd94e7dd5045d7a27013b7fcd21c9e3b7fa75222e7bda49b729b0401) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x02c6f00fd259ba9440c68d211969bbd81509b234882d65fc79ee90fdcb6ccfda) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x07f4fc84032451c171ea7150385b54a383fb083cc0c93895e2ef931e8e448345) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x02b407e4c824960a965b5193ad8c6ccf4baaa4c99da5d11b13a2d6af52973ef7) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x021fe5c3dd23b42f696dcd08659b8aa403c8e927f8c6e7b1446f4e9205c0a1c2) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x14f63403b60fb3ccf8325ec20e463e1daa492faf4d0151a8e7366f07c68f1d83) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x207cbbaffb34a0fe5eba27fd30f67e5389b1de65b703ccb78726831208ab600d) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x00ef12b054f19d72f2a6d0e628c6387026afd8a8924eb144ccc9948d4f6c5549) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x0a1cbb57818ceec1d15878315046a7db1238d292307cabafbb97f569df6dcefa) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x0d098b0bcd6db60c47f8e7e9eb1c072972deb39b294907cbc353352ebc2bea85) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x0ff57407d8b18914e30d8583a77f67732f8b2762429a712c55b0c00fb83fe1c2) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x2b01c45f214633bfaea1589083ab9a3a0915a6da362baa3151b1a0e80fb79160) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x0392d6380d2912befda8d98bcddd6050683a814bb84eb7f57e28176033783f11) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x24a6e759b9d12a53f809367cb3cbd00d96dfaa3af623e984bd986886447b642d) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x265e4202aa18f537a514281d72aaea8ab10090da270d8f9901363b4f48bc0610) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x04e5e383b53cf0f3eb3e824dcbc95d7fbb2ca7770bf92a3e86b652a425534714) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x1bb4418c97c423508baf8d7825f2f41066dc4769dc4c9643ebddca0a71b71a87) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x00a2e0e8c69ad29b60904f91a9db016a32a3de05f6ccdf024b5f149e8388484c) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x24be2bffbba65b40f4eeabba7a3660511baad3936c4ec40a6f9e20d194ec3a07) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x28725b01fa9c481b39aef64f5f54f9f967fd976b7ff4be45a9ca50f7500fef4c) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x264e3e4c4529b321c407f802c173d2fb73b03e8ce09fe3de3c11f84b87b99d32) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x1ec8ec2e5a6f36a00042f1199bad7fb25e950c9ce97f59777fd1739f422ce750) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x002526bd09111cbc4d6f6c6e200f627e7ae60fb59bd5f1357d82f386b1009dc9) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x0cc83ed6a722c67efdd44d5b6de2490621fd59c7c1c7a1416c99a6dff933e5d9) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x01eb69a024162e13bc58e174cef5c0d2c7a12bdf3619f78010cfe09cd165c19d) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x257e795ed0c6598cb79a148110eb2ce1dfb2a6378267e0a33f3c1d4dd7aadbcc) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x01d596a895131eb6dbf6c9a89ddd9321ec5ed272d921b4edfed20b8f8ddc80cb) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x167af14f050f637263e94a86a2408a14178c7ea304ffaee2db4b2d20e173832b) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x211fb82fbb784f81f12914fbdb876c4a4b1f3670bf7aa291f661f7541bc8779c) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x09796190fd3ba909c6530c89811df9b5b4f5f2fe6501ec21dd864b20673fc02c) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x00b9c2423e310caa43e1eb83b55f53977fccbed85422df8935635d77d146bf39) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x217dad26ccc0c543ec5750513e9365a5cae8164b08d364efcf4b5890ff05f334) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x1db28433f6bde424423f3587787f81c48101d2dc6e54b431332cb275f8518c62) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x2cc2d90f2da7f4ec16b7fe61babd4fb9b580ecff03c471764dd67a8c433afab5) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x3032b9ff096a43ce326cc63ffc6a86dcb913fb1f7700939f5304f6c6beb24574) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x1f4c58502ca713ed0bffb4ff31ed55e557e83a37d31b8e703aa9219d6158e2d2) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x0b0d5ed5432c5e7b56344c1d26ce0d9f632e8f8aa52505d6c89f6da89f357fa8) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x0869d6ec86b39958a4a10ed67954dc8931a1e5ee901099071c3c0684dd0eddde) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x1fc9d5e1b18c601f367b9551c00f5e541a48aa562cd0adb4369b51a7e99395b6) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x205b387095b6e538a6169c93c9db7d85ec219e2f0304b449f8849f5fde2c659f) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x07d8d408db8702ba4db7fec434fdee2b944313f72b0f94a9dcec74e7b715b3f8) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x2c758668e1cbf0572b139911af3f553c7898f7f07ffdcc58484a1a0acd14a03e) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x159322db7ac7485c5be7ce811a773c5fda9e26b0c47139eda1af6103c5c21b1c) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x026ba63c8620f00298a42a356b18392228d92c4301e8c51e44a3a2e14a6ebc89) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x2a962181e6a7df5a05d1750e7a22b6ec21fc84d8de08524aa75c4ee8f646bd0c) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x2c81aa9e4f466e56d2a6f1a971d431a487379970bb892424e12a0c71c41479b0) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x2e662e641087ed19b9ff866748197ab8a871deded79d2835f32e4bbadef1a889) // vk.ID4.y
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
