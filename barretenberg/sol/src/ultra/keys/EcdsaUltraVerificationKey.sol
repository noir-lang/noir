// Verification Key Hash: e30e949f5160482ce231cd52882ea6e3146c3ef53d7a2e7a1ead236a058d5a78
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library EcdsaUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0xe30e949f5160482ce231cd52882ea6e3146c3ef53d7a2e7a1ead236a058d5a78;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000010000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000006) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x00eeb2cb5981ed45649abebde081dcff16c8601de4347e7dd1628ba2daac43b7) // vk.work_root
            mstore(add(_vk, 0x60), 0x30641e0e92bebef818268d663bcad6dbcfd6c0149170f6d7d350b1b1fa6c1001) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x21e3003d8b8316c4ce71fb5419a9ae911da948bb43cf79b6f77db537b680c442) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x3005267a5059be6fde8f88bbd95321c8ddea179df2a11dc1df2e77f740ca8f1f) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x024526d9500f6edf685a057c97b8cff30a2ed489a002f7f35d1856da3ac42e01) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x016b7763c0fba3bb1e445895adfb07bf26cbd99e202b0d8f93a8f7004a306bbc) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x2ff3496186b9586a042939286d9d83596cdd074f6df473613951ce5229fa31bc) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x28d41b097e1c8a0863f5593d6cc4a804e5f748b38443cb7909c6cbe2f9d1ab84) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x19a9e44ac41213223362bbedd7b6d61adf942a0a5880be9c7c53943b4e280640) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x1fa921169505614db374a60d1bbea0a08815b4804542bcbf3309506b8b0507f0) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x2035284dbb7544e5a0c1f5b56084267b165b23f541c038979555b82e0f2607ea) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x02ff6b514c7fa2c20ba949ea94c08c5f5e146e09336b65de3d5fccfcaaf56b96) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x098709b45abc886af6902f48ba0ae3587c9ebd298ceab3183e0aeb6068e96dcb) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x2767a84ab297d7757b53fb08492d0a5f657d44b47c944a080c7c42ab890732f1) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x0e5bc77185f9a211ce4dba62562d072d88600ba25fbd13028829cb916af11030) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x2039506dca5969fc929e7ed9ab162f17294f7e0d7c56959508562d9556357a6e) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x1d2501b2fa086e00afea57bf974ab9df0e259dc30aebf0021c17b8189c42d50c) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x13cdd55a7fa83f59db568fdc67f4b07e7adddd403b975edcb7b4bf2ad2dcf453) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x21245d6c0a4d2ff12b21a825f39f30e8f8cf9b259448d111183e975828539576) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x16a409532c8a1693536e93b6ce9920bfc2e6796e8dfe404675a0cdf6ee77ee7a) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x2a5e88249e7a11c5011cdca34920359e9442a5ae3aae68e56e8cdfc4062c8b52) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x12065cc874d23213d1ccbef7087359fa4c75a87d9a25df50a05dce8e635073d5) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x2e5b641397b2265450974ac68cdd0f151bd66d9c9854ab6f64e84671f3e0c267) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x0fe1186570db78e9d0c24fa2f5582c43d1ca518567caa9e033007612c8245873) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x0d5bbea26d87e9fadc9c3b860123ca849abba8e0e41c4e55998022b51d95d8a1) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x1b503ef8d777b251c32e63d17a76e9ebd7efb586063bc52d6f75c86d3722efe9) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x07c2dae78ad3943d62867423792d1cdaa68df83ebc974863ae3be3f90c490aae) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x15301e4d92461354d18ac6208dcfea4967feac437435efa613183fb84007c6ae) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x272d645226dbce24fcbaf241b18ebcd6c745f5f462f4f17b1b0d05deb1b342f0) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x2a9f2b440f257f3e4619df6f3cb0c0cf1e8026061b4071249ae178447eb51b9e) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x18f7cf965339d9c9d190296fa92f915767b0a8da455975f3e03fa98439fd7110) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x0eecc02f9d44125407adbf00d56b086afd1adc5de536450afe05de382761b32f) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x0bdfe662ea9f40f125ca5f7e99a8c6ba09b87ba8313864316745df862946c5c4) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x0c5313c5b17634332920f54081fd46464a5ce9399e507c8fece9df28bff19033) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x232ab86409f60c50fd5f04e879fbcbe60e358eb0337c5d0db1934277e1d8b1f2) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x1fda66dfb58273345f2471dff55c51b6856241460272e64b4cc67cde65231e89) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x024ccc0fcff3b515cdc97dde2fae5c516bf3c97207891801707142af02538a83) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x27827250d02b7b67d084bfc52b26c722f33f75ae5098c109573bfe92b782e559) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x1ae2687fae0bfbb8b923aee57fd70697da8239d170e3dd9e903c5d2141073acc) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x2bc2419b9c6badd0755da06b3f73fba761bf5fc6708b1d9ebf8024ba7f95a2f6) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x1006cbbc3a187f1d286337a2a5851481ad736ddc9708de146b0c16af67af55f5) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x11cc4086b8c85a1c1cb633e148743dd35026dcb5d78b2d95c1d82235b4aa3f55) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x1280e0e41489a0689eca740eb87c2a956d7e5e01490d4ab8bed22cf702b868f5) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x1ee6fba2609e79fd8c183a090740d594b31ff64d1fa417d5b257e073d158dded) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x2b9204f05f51933d2aee0d9b4d6abb90fc8b647d2867191259b6b1180081d75e) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x0ae99a82bb35dbde21c2930925f571f81f41e1fb7afe103a52c01b830c042449) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x12dcf5c41e156844037bb35b2eb4b0b0c0e40c75b8374a1800387dbf399b4bc9) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x2c1e133ab13d64c88e3ac0dd7e4c29f4cb517ed81f84053a824608cc5fc3c3b0) // vk.ID4.y
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
