// Verification Key Hash: 7370a14d9a35deb926608bdc13693b06292d2f66052be3dd6d13d35441270318
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library BlakeUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0x7370a14d9a35deb926608bdc13693b06292d2f66052be3dd6d13d35441270318;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000008000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000004) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x2d1ba66f5941dc91017171fa69ec2bd0022a2a2d4115a009a93458fd4e26ecfb) // vk.work_root
            mstore(add(_vk, 0x60), 0x3063edaa444bddc677fcd515f614555a777997e0a9287d1e62bf6dd004d82001) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x260b63c5c404eba05c531b64a63c4f8752eb48b02d8d53910bbc22e7648b672b) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x0a84e4198292ba82e9ec6e130ebf86ff8513b8133501e8b7c625c322451cc17a) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x066bf08a2edd0ed02a5b2b4c72f512585b1816d943d06f4822219d37d28b88f7) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x18e0525744e594592f2ba6832df668c1424c920ebf1e2ec358558075df1fc906) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x25880f31af07e4d48de7be715bc3b63495b1ce16c3ce6233ad5ba832cf3330a5) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x190ab6675593f90dc404518e02369c97f0d736010033237073dfc5611cb4e0cc) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x1afd5ebe896054ba2812a94f5903a17aa5de0ffc7f1915259b4d9e01a24ceb44) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x03b445d3e75bd9ecf05703d2301157ccb3795adf7ddb3c0d03cbcc691772288e) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x00f612887aad4e61796d7948533fff40184bd1d00ba52e9201fea5b9b5a8258a) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x25bd9c5fec68e480ccf127be8b4bf7810c737a38f4d6a4379b3817d4d157a3f5) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x27e6361916a9c2a4f81501df8f6588c394f7ba0010e565fe9162e1456acb64fe) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x232d6f8f7582fc930a95c1d97e1cbe471935642ef95ac1457f953b92601a7f36) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x1344134626051322a90942b7cbd3a98227e7e192c8597604dea27f5eb49a1332) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x2c4782c37eb3e19589fc42e465f89cb3dd47ddcfce1a3a5f7e0e6423e9290f53) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x2ea84c6aebfa0d7b78e6f8344086d9a4ceabf599cdc3c8b8efaf937f78fa89f8) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x00fc4dc0688832477ed1b999b886307775590a5155ccfbe5e4a686cab3684fd9) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x21959276775cd4749236c8bf773a9b2403cecb45fbf70e6439f73d75442e8850) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x017714509f01d1a9ee7ebaf4d50745e33a14150b4fe9850a27e44de56d88cb14) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x2e76c4474fcb457db84fb273ccc10a4647a1a37444369f2f275bb74540f5e2d0) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x209035caddd02a78acd0ed617a85d782533bd142c6cad8e3338f3142b919c3a4) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x16a04bedbbced0858d1cb768d5dee65d7a9e5eda5840f041a6b0c2d9a05a47e9) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x0f295c2f65406bd8aa6844f7a8c797da1ec69b048441e5926a0d11e515056af4) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x219f3919df06c1843bdcf405c9c6304c9affb6b5b075e25d9213cb9ca4177ad8) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x2a5acc53d574ddef7c44bc0d578d9371cecc89ab42a4b0bc6017eaecc68ebeb0) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x270efbcff761d452b5a3024f5a1b13b2108bfd126610f7c6580acfc8a3eadc43) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x052330edc9afc72fdaa4d7c5df4617631634c46a7132ece7ec56286647f66a77) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x1a07bea503dbfd8375d3cd35b79187516326c0a96af71418b8004e863d2126d7) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x1601a5ddea012bc53cf9633d99e704caa30b017e3e81e935a8d791030be559c3) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x06c5d3c2a64587cf9dc278c6892854fc8f1aba4183115224cb2eda4c1aab64b8) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x132622df9222e04fa9c4cf2895212a49556038d4fdc6d0d7a15b1067bb446efa) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x2dbc1ac72b2f0c530b3bdbef307395e6059f82ce9f3beea34ff6c3a04ca112bc) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x23e9676a2c36926b3e10b1102f06aa3a9828d1422ae9e6ea77203025cd18ada0) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x298b6eb4baf5c75d4542a2089226886cc3ef984af332cae76356af6da70820fe) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x1bb16a4d3b60d47e572e02fac8bf861df5ba5f96942054e0896c7d4d602dc5c7) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x1f5976fc145f0524228ca90c221a21228ff9be92d487b56890a39c3bc0d22bf2) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x0f43d83a0d9eb36476e05c8d1280df98ec46ce93ae238597a687a4937ebec6cc) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x239c09880dcbafee7caf9fb8460d1ca62e86b42b8724e350f01a12ddd4f08add) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x14dd9f4bba78075eb5bdb7d401281b01d9e88f8827fab15f6b718bfed5b6a598) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x0059fbc7f6f474f8a602db54e7aeb9b7072081bfb31d4831562003e8c5804177) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x01e2a561adf9c7843fd4e9acab18137656dbef06f22c9d2f05a68eae8576bd6b) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x205ed43983566317600b8324e02262240b23d6caa751e53360fe9410deb876b3) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x29ca9b6ba6da40ef21d62321d81594b449185bde9f071a3619731444a3cc30a2) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x0e633257c8c686bbe65fbf5792b8c944747838fd385d3b02eb7900dad50f6f4c) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x1bfda6b7d38472e9418a8eb55f4c1d372642b5819fde074d4fe62c29f843b566) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x1dab0d03d72afa6328933a39b05c764bc713f67606fa016ebf532deb2b4bc105) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x24bef3bbfed9cfcedabed6d61d289ae44ce360aa38fd022886fd22bc75fd5980) // vk.ID4.y
            mstore(add(_vk, 0x640), 0x00) // vk.contains_recursive_proof
            mstore(add(_vk, 0x660), 0) // vk.recursive_proof_public_input_indices
            mstore(add(_vk, 0x680), 0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1) // vk.g2_x.X.c1
            mstore(add(_vk, 0x6a0), 0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0) // vk.g2_x.X.c0
            mstore(add(_vk, 0x6c0), 0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4) // vk.g2_x.Y.c1
            mstore(add(_vk, 0x6e0), 0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55) // vk.g2_x.Y.c0
            mstore(_omegaInverseLoc, 0x05d33766e4590b3722701b6f2fa43d0dc3f028424d384e68c92a742fb2dbc0b4) // vk.work_root_inverse
        }
    }
}
