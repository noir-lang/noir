// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.21;

import {Honk} from "../HonkTypes.sol";

uint256 constant N = 32768;
uint256 constant LOG_N = 15;
uint256 constant NUMBER_OF_PUBLIC_INPUTS = 4;

library BlakeHonkVerificationKey {
    function loadVerificationKey() internal pure returns (Honk.VerificationKey memory) {
        Honk.VerificationKey memory vk = Honk.VerificationKey({
            circuitSize: uint256(32768),
            logCircuitSize: uint256(15),
            publicInputsSize: uint256(4),
            ql: Honk.G1Point({
                x: uint256(0x2e5f133c25f7e05bd6660196c892121f7fa686cb9a8717a5deea6cd0881e618e),
                y: uint256(0x1189bba9eeea96ba8935052434f4b0a60b0a481e3464dd81dfcd89e23def001b)
            }),
            qr: Honk.G1Point({
                x: uint256(0x2a93ffb34002da94f5b156ba5a212ac3616c848bd9c44c9821bbdd64cfd848af),
                y: uint256(0x015699dcc0b28766d45f5ddce8258393e84c40619d26034e76f778460a1e4d89)
            }),
            qo: Honk.G1Point({
                x: uint256(0x2057928e8c5eb539c32c3025007b7be1e1663c358f59540c6f949794c274f886),
                y: uint256(0x12bf0b15c3aa92792330f58b04512714c4a902e537fe87cc438293e1805eaabf)
            }),
            q4: Honk.G1Point({
                x: uint256(0x304f47a08d4687afa0e2502a9782c32c458bf873ef50c169b732a367e567aaf3),
                y: uint256(0x0bb37044594e7de200408a4db6bc46adf7790b06f17dce6f38b7deed480aa9f0)
            }),
            qm: Honk.G1Point({
                x: uint256(0x0aea5b04332ad8781411f7edde21266857ffe11e93af334b14a2b948429afaa4),
                y: uint256(0x2bd2e3884d486b387122effa12e8698daef82e9b99d7d25b7d5df91a9d738495)
            }),
            qc: Honk.G1Point({
                x: uint256(0x0e3b418ea1924b4514d5009cd983b5a8074fa95cd1fb200f019fdebe944e4225),
                y: uint256(0x1e6ef5bde7a9727f1c1d07c91461ae1b40524650b35fdd92ac7a129f263b1beb)
            }),
            qArith: Honk.G1Point({
                x: uint256(0x096841bfa8ec2295a5af5bf69ec539c31a05b221c84ed1d24c702e31ce1cbc95),
                y: uint256(0x10b14cca7e9ff05fcf1e3084f4fc9ab098cf379864b2e2e2e0d33fc5df9d9a50)
            }),
            qDeltaRange: Honk.G1Point({
                x: uint256(0x2d27fd1a30a0ab04a05144c27ac41187d5cf89a6022e47b263d1ccb93b3cbea5),
                y: uint256(0x238eb233e9aebc81285a2647f2598bab00a4367da47b12c2b0476afc2d94ab1d)
            }),
            qElliptic: Honk.G1Point({
                x: uint256(0x1c6fc8e14453adf64e6d9643ef9f1fb55e3a307ac1ec84f86cd736fc866e05ab),
                y: uint256(0x1bf8619b1704b99ab8820ed94dd760da2945e8e1c771c0bdeadbe40aa5700cdd)
            }),
            qAux: Honk.G1Point({
                x: uint256(0x023fe0703623b99c93358348d76eb620f26ceafa58df018e3a8f1d599a61e76f),
                y: uint256(0x2ceb9c4c4ca12ea769157ef10cde9644f9f0549805e48d5fd5d73a634d2cdcb5)
            }),
            qLookup: Honk.G1Point({
                x: uint256(0x1375bbfbf5ed31b38460f46a43ac14e2cda93a3bc5cfd6e8a93cca356694a346),
                y: uint256(0x204c5173892c19a97a04b5f8419898063df5136489809ddb9f7eabb58d6858ab)
            }),
            s1: Honk.G1Point({
                x: uint256(0x07205fa20c0a8ff2aab958f4dfc4fa14449aad08f66de2341412a70ca6cb972b),
                y: uint256(0x1341c9e5d1300703ae7ab1689c2d83a843ec898e5120a2b52b4323e2acf94f30)
            }),
            s2: Honk.G1Point({
                x: uint256(0x09821cc06874047f1204cf5c13d5d24951757e727368690715de0e8160f50eeb),
                y: uint256(0x01675621d2a4202874b596d732f983f06524dbcc855a08dadd9925e29c02c7e3)
            }),
            s3: Honk.G1Point({
                x: uint256(0x2567d6126bd4dfa15eecbda777e9c86566fc5817b87bdd479e558919cd5aa96b),
                y: uint256(0x2ac31be3d7a2f45625b98c022106a076c0c4730299c18d48c7739d423e53bd94)
            }),
            s4: Honk.G1Point({
                x: uint256(0x15d68f126e0adc08580957fad3db16973c5c7f47a64d04646204c2b14ec14482),
                y: uint256(0x3048e354794e4dbcbb670dc137409427a954eb899bdba8bbaeec5ba7f3a79170)
            }),
            t1: Honk.G1Point({
                x: uint256(0x1ec1b607634e31421b5047dc99d7674d6505fed978df0f42a3504f9771a8a7fa),
                y: uint256(0x1da802c6dc2fe6cffc6f9ae983080c66690ceee40c181b4d51fdba6c5c360297)
            }),
            t2: Honk.G1Point({
                x: uint256(0x1e38a0a482b7174f429a3bef25fb0a7656abc88cfd215b8e8404132601620784),
                y: uint256(0x2e9ea07a995fa6d589e37fba2715f3f1fa338652ddf84d4e2e4f33dccadb9156)
            }),
            t3: Honk.G1Point({
                x: uint256(0x211a0833bb3c6f9ae6c94519b6878ed6157c4a080df786a053d9a19796b9a7f8),
                y: uint256(0x1a3a450e1a272aa1fe9f097acf359502ff69df617de4918b37a497def94db2b5)
            }),
            t4: Honk.G1Point({
                x: uint256(0x281a984aef14716cd5d8fc2759eb8ea2464909b5c57d97b6bc50e4dad74d92d3),
                y: uint256(0x169160e1505685aabd5bd60e994bac45162c6868235cc4252c8b87d0f12603fd)
            }),
            id1: Honk.G1Point({
                x: uint256(0x01c082a85908fea4c69c4e51436fba7d965e1d88e485da16e35d8f4e8af3b8bd),
                y: uint256(0x11b0ada021468b059aa6c27f4d4950ef65b98d4d8808ae21718bd8b90f9bb365)
            }),
            id2: Honk.G1Point({
                x: uint256(0x0b8667619755bd09c7970defeae2c920df2b17b41608303ae1d7393615dd04e4),
                y: uint256(0x1c5419cd435c5516ac566a9d1dfecdb4e10190c63f2dbd8a1932785caf022e2c)
            }),
            id3: Honk.G1Point({
                x: uint256(0x110aee72793c4b4ede92c1375f058b4170fcf01bf18f8f1ee934f7ae0fa26da5),
                y: uint256(0x15c4f6a01ff04ef6b5225c896dfb7060a7a2c320395bda410e756d6b507b7eb8)
            }),
            id4: Honk.G1Point({
                x: uint256(0x2472aba130e7ed2aefad128109415ec2bdeb56e81e3cbeacc93e00c95f203579),
                y: uint256(0x0c867d0f8e2f9c861574383b89020980358d898497f80c198a6c17c2f4daf9a4)
            }),
            lagrangeFirst: Honk.G1Point({
                x: uint256(0x0000000000000000000000000000000000000000000000000000000000000001),
                y: uint256(0x0000000000000000000000000000000000000000000000000000000000000002)
            }),
            lagrangeLast: Honk.G1Point({
                x: uint256(0x13b825e996cc8d600f363dca4481a54d6dd3da85900cd9f0a61fa02600851998),
                y: uint256(0x151cb86205f2dc38a5651840c1a4b4928f3f3c98f77c2abd08336562986dc404)
            })
        });
        return vk;
    }
}
