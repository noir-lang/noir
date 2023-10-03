// Verification Key Hash: ab0e7eca8953a659e04b83b3e1eb0525036ab76b5c6c53b090c8e3e568df3912
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library BlakeUltraVerificationKey {
    function verificationKeyHash() internal pure returns(bytes32) {
        return 0xab0e7eca8953a659e04b83b3e1eb0525036ab76b5c6c53b090c8e3e568df3912;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000008000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000004) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x2d1ba66f5941dc91017171fa69ec2bd0022a2a2d4115a009a93458fd4e26ecfb) // vk.work_root
            mstore(add(_vk, 0x60), 0x3063edaa444bddc677fcd515f614555a777997e0a9287d1e62bf6dd004d82001) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x1a80f3623f0778b08837c863a65a1947e56828b26496c5737e25041052f7009c) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x2a3cacc92240779ca4f320c576190d0d6ec1266368357178a9a8260cfa235455) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x2acda97834aca4418d41ed03d801e2dd48a1d0b5c41d76ad369a8f7af50e0661) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x1b69f6a121b1cb789d87cbb3236233b7a4a480ee9bdde5b7bd47e62a227dc5fc) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x0ac1dea6a1af1f8971d1341ab89d8a04f441306474df59985c648f96d1de246c) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x0d41764f9161b19571da0200f8efa730ac7c2025e09a2decd3e4889261792e8b) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x26c4dcb136c91c0acb772bba73db577c1f0f3c09cf4dd7cf4a2bcecf4aa3057a) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x0a5072484a9fce7eef8d097a377375142da6f88bb9616d5ca1f2aad4bb092129) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x2c294f4909e331bfdaf0b65ac1b7197251217d346f0b990471359d419a632662) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x0b3d987ce70e2cff4a2fd93156efd896e0f4f9f671ccb1ab8c3493217b3d9b07) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x1dfbace7f5f42d72d8df9c6c40a118a8c05d340796095cdb30d5ae76807ff4c3) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x1a8d4a2ec9837b684fa5b688e36f48288c1450591c7e807bbb86ec125d7b3dd4) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x0a51756abf4c062b5f4d76b29e1dbba021b670d13442aee1fef9e3018eb4aca4) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x0ad4318714e1493aae41463ebd941fd2d7fb5357c55ae9eb6491c7c3ccb8399d) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x0d7d8284681025c0926d0bfda9a6887098f151a49b27160e76c0cbe5e081fcb8) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x083b2abe0a5c29769ba8f427e7440a3dcb1ffcd86cb1b106f0aa27b6903655ba) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x21959276775cd4749236c8bf773a9b2403cecb45fbf70e6439f73d75442e8850) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x017714509f01d1a9ee7ebaf4d50745e33a14150b4fe9850a27e44de56d88cb14) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x2e76c4474fcb457db84fb273ccc10a4647a1a37444369f2f275bb74540f5e2d0) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x209035caddd02a78acd0ed617a85d782533bd142c6cad8e3338f3142b919c3a4) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x12922936896c92f4773be96aa6eced49c9e6973d091baca38bcd2ce7c2432b13) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x0195e866ae7531344bc2cec037b002d99caace033bdbdf98b4db64b1e23be236) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x136128a1e6bc7bc733fcb9343fe23760d8d8a08d4ba4cdfc8970429696720a85) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x222d19d2afae1cc8c5b1b802be90b5d4d648f5e89f4b32df381fb92135497a2c) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x0dea033b8db2e2948559604ffac72f411931477ca58240ce0a3007c9a82d1b0a) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x1fd05a799d3dfaba1bfed3145b27e4768be0bc0893b24962de96fdbce7cdc319) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x07f75bc93d92e6dccd6ba833a23aa5d1ba7c36cd2e2d6c2e6406c62480e19119) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x253f74ab6ef1778beba646f245c7dcd5e60cdaebcc37d2c5db22caf03ee7091f) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x06c5d3c2a64587cf9dc278c6892854fc8f1aba4183115224cb2eda4c1aab64b8) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x132622df9222e04fa9c4cf2895212a49556038d4fdc6d0d7a15b1067bb446efa) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x2dbc1ac72b2f0c530b3bdbef307395e6059f82ce9f3beea34ff6c3a04ca112bc) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x23e9676a2c36926b3e10b1102f06aa3a9828d1422ae9e6ea77203025cd18ada0) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x298b6eb4baf5c75d4542a2089226886cc3ef984af332cae76356af6da70820fe) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x1bb16a4d3b60d47e572e02fac8bf861df5ba5f96942054e0896c7d4d602dc5c7) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x1f5976fc145f0524228ca90c221a21228ff9be92d487b56890a39c3bc0d22bf2) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x0f43d83a0d9eb36476e05c8d1280df98ec46ce93ae238597a687a4937ebec6cc) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x0b140556df3e8e29980eeae95a724d3c06c830707ce655b6bee64acc9be9e9c2) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x04de5162de4a6bea5e1e3b1386d87d41adef6c98203f749c98d2c2113ba12e9b) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x2e9d323290bebf84302163b43af960fde161663907a5e344ab62e3d0db9dc2a4) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x282cbb41af3a18746f880f35caf350a0e653cc9ff266380f9dad6c8145fc8532) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x29c73bc4fb55d00bb6b91bff171aaa98e3ab1d64829603b29a5364e510f692d7) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x1011dea47912a0daae13bf8f0e4b6df1f706252a27b69d34724cd05a3bbb5542) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x040261783eb93ad94557cb1c9c798d8468953eb65b00b3e0059fc7af5555ac5f) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x0e05422e3bc394ae55128d83ed0087f1d18810c951f4d03b2f0eff6d5b2e74a2) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x1b21ff32cadfbe3b43171ee3d0b14d1eed1c29e3719320d3e61f0e71c6aaf6d8) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x04f57e846a88c4a0254841cf7b6226e878e7a4ea49c34c3732870f1d8c4f6c18) // vk.ID4.y
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
