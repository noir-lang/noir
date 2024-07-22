// Verification Key Hash: 37857017e148045fbf1d4c2a44593a697c670712dd978c475904b8dc36169f18
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library EcdsaUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0x37857017e148045fbf1d4c2a44593a697c670712dd978c475904b8dc36169f18;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000010000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000006) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x00eeb2cb5981ed45649abebde081dcff16c8601de4347e7dd1628ba2daac43b7) // vk.work_root
            mstore(add(_vk, 0x60), 0x30641e0e92bebef818268d663bcad6dbcfd6c0149170f6d7d350b1b1fa6c1001) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x0740309101d13ea04888ecf9966d509116884ac038bdf2c07646806c87882e5f) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x2fa84f63c12178cc8ac1f4e8588df585bde0d219e278e4ee201ffba99a3411e4) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x0558758bd7554eeff4f8134d8ae5698a017ce2a129925bd48801548c77fbe63c) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x0fb3ffe7ccc0570d878afafb118b383e81957c7b45b1b6ec827687c32e041b48) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x29b23512911ac602dad1322d794b14d93969dfce065e7d7f3c27eca231a7ce6b) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x181fa6dd3e9381e50a4803813066991c57a2da9124bc3578a50c368e64806238) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x00eaa74d50fa51c361607c4975d0901d985c3702e451ff6f90919ef5f30fb953) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x2943e906ab4660823fe6cc999101066309f99157af7a6fa948f75bac0527237a) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x0bb76b97b2e497edbda41d64eeeb4240482861bfe02220944a1f1b92939d31b7) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x2777a52a5cbeb31f3f16c2268492e233bdd5cf02fd0c4e68e404347eda2670ac) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x0ec4edd551546f895838153731c4241ba099c255f4dc3d0bd88fc901cdc941bc) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x09bcf0a0c13ae4e765801b6b8acc8eef3da55fb43ff5a9305dd75c66d1287a0d) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x2bf60d0b3adda58dfe2be290fcebb58851071646f7f715a5a0063dba93168b83) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x24562762b7b7219f77188e1e0c1b4db96493b1c7e41a5f1e7c0130ab5b720f43) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x04cf13b71400ed5ba47983d1d0795124e7ea3caf4f7d24f7713fe2205e2f80df) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x15cc2c54fdc490a08833bd1a449bb0fbf26cd0590c648b45f7d83da966d5e9e8) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x21245d6c0a4d2ff12b21a825f39f30e8f8cf9b259448d111183e975828539576) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x16a409532c8a1693536e93b6ce9920bfc2e6796e8dfe404675a0cdf6ee77ee7a) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x212c745dcc4a77000c8811ed40afb24af308b4df977301025f700d34da365259) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x1f948ad81820af56f3eca2ae3d7e99f0d0a7edb380cf5ebb9f33a9cba3cc3003) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x2fb17d2149521c0656a73395466a295e30cde468aca4124e34235609ede6bcf7) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x2e9d5f4cb83ec03ecdd8bb1c8e8319f991e80d8b1af6f420f75ffc1851da6d77) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x003841079f4625f2c236862b62391a7770a7ac22c538c4af8d70b8dba8e62a4e) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x0b55d7bab80f1febf20f1faca1e71c5f917d1eab85fe03df8bcbec192f25da58) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x26ee5759c003a3e371be8fbbe1cb783bce386b4583d1ee22e3abb86a1b0c59f2) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x00678775ed828729a75de81d07419b825d017c80d7e1520f659dc23bb476cf0f) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x282a0813e6ce49d1d2e70a13fcdc9aa4622b2218a83e8eca83e7c72747c37eea) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x218a25ab5d221802dab140aad610b6ed71674a8cb9d899b85cafed786a778213) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x18f7cf965339d9c9d190296fa92f915767b0a8da455975f3e03fa98439fd7110) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x0eecc02f9d44125407adbf00d56b086afd1adc5de536450afe05de382761b32f) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x0bdfe662ea9f40f125ca5f7e99a8c6ba09b87ba8313864316745df862946c5c4) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x0c5313c5b17634332920f54081fd46464a5ce9399e507c8fece9df28bff19033) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x232ab86409f60c50fd5f04e879fbcbe60e358eb0337c5d0db1934277e1d8b1f2) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x1fda66dfb58273345f2471dff55c51b6856241460272e64b4cc67cde65231e89) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x024ccc0fcff3b515cdc97dde2fae5c516bf3c97207891801707142af02538a83) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x27827250d02b7b67d084bfc52b26c722f33f75ae5098c109573bfe92b782e559) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x2b11866ecb5b6a1e891e14792b0c04fdb6484cc1f910410459d3a9dfcaa75435) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x1422d413ccda11d6519d87f9379c29ef515ff37bdd366ab1aa7fc2029263cfe7) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x255d46db0711edd487412cba825c76c98f793128a8ce85c35133541e4e02abb2) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x160fad047875fd62e546eae73bce930febf8477c87769a5b1063eae02638c560) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x01548254d6a72ea1faeef8e50364e8103b527e4df00944d0a0b53d231a6df578) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x040430723dc214dcc9d20d0ed95d24d62a5f935e6243eec6302917c1b5d2b38f) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x1bde6129bc64b0816a88717b9b5b2904c85e7b88c73ac1b0ba75d371ecf235fc) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x0f7e40cedb79332543fd5c040a0eb687b1124869437414aee4497e9d66a6ecb9) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x25fbe717194485caaf7607cd9043bc528100b7e757f957680865edf2d7794616) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x119e345a421dcd7c7c9d8ffd14843eeb1536f640d777340dc4321e59941a3ef7) // vk.ID4.y
            mstore(add(_vk, 0x640), 0x00) // vk.contains_recursive_proof
            mstore(add(_vk, 0x660), 0) // vk.recursive_proof_public_input_indices
            mstore(add(_vk, 0x680), 0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1) // vk.g2_x.X.c1
            mstore(add(_vk, 0x6a0), 0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0) // vk.g2_x.X.c0
            mstore(add(_vk, 0x6c0), 0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4) // vk.g2_x.Y.c1
            mstore(add(_vk, 0x6e0), 0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55) // vk.g2_x.Y.c0
            mstore(_omegaInverseLoc, 0x0b5d56b77fe704e8e92338c0082f37e091126414c830e4c6922d5ac802d842d4) // vk.work_root_inverse
        }
    }
}
