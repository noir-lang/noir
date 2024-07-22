// Verification Key Hash: bfb307aa80c044280f7b66742bc68eb8075aef056ac704ffd7d5cbe1d83268fa
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library RecursiveUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0xbfb307aa80c044280f7b66742bc68eb8075aef056ac704ffd7d5cbe1d83268fa;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000040000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000010) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x19ddbcaf3a8d46c15c0176fbb5b95e4dc57088ff13f4d1bd84c6bfa57dcdc0e0) // vk.work_root
            mstore(add(_vk, 0x60), 0x30644259cd94e7dd5045d7a27013b7fcd21c9e3b7fa75222e7bda49b729b0401) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x2872629d2db8cb63feef4ab8ab250d2b8bd2fbdb622942962872f05b5e742d95) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x1948d2b7b800c7a93368e1987b7871e7df65246af142f6573596865684913cc9) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x27c1cc0a26221de00043568e1ecb193fc349be273624f69a9c3e766470f936cb) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x0ba47ff2bea29b40ad0a9c8650a8833662b461389862423a7a73bbb54c7d4f0e) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x11710dd57cc6fa657dccc53cc681de5fc97d324f1f0c72e20c08371ee4f65e5d) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x244fd7fe0b18f6ff86a773655b34fb76abcd5217877e7df959e3e0151000ccd5) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x065e12520e3b7419d0c3b557d7a26f24d68763744b74a922993992148cfef300) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x2d5af9ed366fe901d76b2d5dee01535679a6adec0d88d9192dfea384ad56b4ee) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x2c2cf9cdde5d02feb8f244b62f37d794c2dff66622a55c77188f30aa0eb96e2a) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x188afa06880347bd0a7181a6ee022742889e5c325df2a46ed325816c20472717) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x2db1866af181147eb2deb1a81a237a80ede43cd670afed4ed707f5452043e41c) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x07686d5c3d3166a9f171b4c6ea91e787b0988b78a4bc833894f0ff38d9098ea7) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x1a906065dbb4b466aa34c76b50d49941b8f0cc5257ae7825093eb555cf093ecd) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x16be7a84788d5d7f80fa66a850bd2a61977180e4caaaa44193458847e96f4df3) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x12c6fa152ae4645fab9ce20db8ea7dee0eac2cc6562840f4d025036e7a06cdd0) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x26a190dcd1abf61afc12ff56da242a29c196d1f75b3c0c6922dc026702489c2b) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x2d7d1a26c4e4d806f73753d6f33c887df548d60adfc903867189947022643261) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x2d71cb325bb3a6af51e99830f0700d4ccedacbfd21ec7dffebd7e9c8cab386a6) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x30094b60cb5228b662501dd1942ea7d989694530497df30d9158b3933773f7f0) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x04f2cc1c0aa3a57dc8173be9dec656884aec11a6f154eb2d0a31e7a5b3bc65bb) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x14c1d865fc203cf3248b978de69ca2cbed717176a83931dcaaabddf84aac1125) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x03413319a4878452335c3c679a952a71dad895d92acd1db566a8b72726b34f34) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x01130e52ac74730e6fad05aeb3ee1e3e14feba55956bd944159ce65e854f6bb4) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x0ee3c32db6c8483ae225f58ffce0c57e757852401b26f666506619529e6d830e) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x1cdf6e1d1a2c00394de5e91aeafd134cf75f2c8b8014bf21d6074c2f66aff68c) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x24fe45588eeb54159a8e9aa2e3c3a3ab90f78f0e509dfcc3cf3c03fffd5e2693) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x2294016984ba84bd39822af4780b4fa91f06ac9f46d081c6e4639dc0091eff2b) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x2070976b4eb47279553017922492bd0e8b78e57dd8bf424309f133fd02e9f8b5) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x09796190fd3ba909c6530c89811df9b5b4f5f2fe6501ec21dd864b20673fc02c) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x00b9c2423e310caa43e1eb83b55f53977fccbed85422df8935635d77d146bf39) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x217dad26ccc0c543ec5750513e9365a5cae8164b08d364efcf4b5890ff05f334) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x1db28433f6bde424423f3587787f81c48101d2dc6e54b431332cb275f8518c62) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x2cc2d90f2da7f4ec16b7fe61babd4fb9b580ecff03c471764dd67a8c433afab5) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x3032b9ff096a43ce326cc63ffc6a86dcb913fb1f7700939f5304f6c6beb24574) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x1f4c58502ca713ed0bffb4ff31ed55e557e83a37d31b8e703aa9219d6158e2d2) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x0b0d5ed5432c5e7b56344c1d26ce0d9f632e8f8aa52505d6c89f6da89f357fa8) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x14cc772886149da159eca12ceab8b0cb5d63043e354699ef54bc13e2aff0a57f) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x1874e1387aa83a45cc599a97c8027a53c92c0934c1cb59537aeda8b9edf8b019) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x16fe41844f95ddeb66b8d54eae7dcb87df2fafe782e09dd0034e016474742136) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x05f0dbcfa556fd26f1563cbdf8f8d2d5f92ad46ff561655383bf5efa341b58af) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x0e2e21b159efae413496f5987159ae35a22970d2e81865c2d19ae32170305b4d) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x01c6cd8f3faba6127b043644fa40aec9bc07ef5f5dd1fc73abadc8ff25b28bb4) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x0cf6b92268f315107f92225c8c7853c89907f4f016727c8e038b74aad81585af) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x1955c97edb1cd3637736cce7a911b64626fcdd5eee63a882f9058d901713b5e3) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x012ef051b06db65bdf78950b5d3ca6b1bdfef77e381e9220aa25d72c63518f27) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x265f2a5c1d100fdc81183e099e52de0751578056574fb56fc4c7814a31e5fc86) // vk.ID4.y
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
